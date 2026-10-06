use eidetic_core::contracts::{
    AgentRun, AgentRunId, AgentRunStatus, AgentToolCall, AgentToolCallId, AgentToolCallStatus,
    AgentToolRequest, AgentToolResult, AgentToolResultPayload, AgentToolResultStatus,
    AgentWorkflowDefinition, CommandEnvelope,
};
use rusqlite::Connection;

use crate::agent_workflow_store;
use crate::agent_workflow_store::AgentRunHistoryProjection;
use crate::history_store::HistoryStoreError;

pub trait AgentWorkflowProvider {
    fn next_tool_request(
        &mut self,
        turn: AgentProviderTurn<'_>,
    ) -> Result<Option<AgentToolRequest>, AgentHarnessError>;
}

pub trait AgentWorkflowToolExecutor {
    fn execute_tool(
        &mut self,
        request: &AgentToolRequest,
    ) -> Result<AgentToolResultPayload, AgentHarnessError>;
}

pub struct AgentHarnessClock {
    next_ms: u64,
}

impl AgentHarnessClock {
    pub fn new(start_ms: u64) -> Self {
        Self { next_ms: start_ms }
    }

    fn tick(&mut self) -> u64 {
        let value = self.next_ms;
        self.next_ms = self.next_ms.saturating_add(1);
        value
    }
}

pub struct AgentProviderTurn<'a> {
    pub workflow: &'a AgentWorkflowDefinition,
    pub run: &'a AgentRun,
    pub completed_calls: &'a [AgentToolCall],
    pub completed_results: &'a [AgentToolResult],
}

pub fn run_mockable_agent_workflow<P, T>(
    conn: &mut Connection,
    workflow: AgentWorkflowDefinition,
    provider: &mut P,
    tools: &mut T,
    clock: &mut AgentHarnessClock,
) -> Result<AgentRunHistoryProjection, AgentHarnessError>
where
    P: AgentWorkflowProvider,
    T: AgentWorkflowToolExecutor,
{
    run_agent_workflow_with_connection_tools(
        conn,
        workflow,
        provider,
        |_, request| tools.execute_tool(request),
        clock,
    )
}

pub fn run_agent_workflow_with_connection_tools<P, F>(
    conn: &mut Connection,
    workflow: AgentWorkflowDefinition,
    provider: &mut P,
    mut execute_tool: F,
    clock: &mut AgentHarnessClock,
) -> Result<AgentRunHistoryProjection, AgentHarnessError>
where
    P: AgentWorkflowProvider,
    F: FnMut(
        &mut Connection,
        &AgentToolRequest,
    ) -> Result<AgentToolResultPayload, AgentHarnessError>,
{
    workflow.validate()?;
    let mut run = AgentRun {
        id: AgentRunId::new(),
        workflow_id: workflow.id.clone(),
        status: AgentRunStatus::Running,
        intent: workflow.intent.clone(),
        created_at_ms: clock.tick(),
        completed_at_ms: None,
        error: None,
    };
    record_run(conn, run.clone())?;

    let outcome = execute_run(conn, &workflow, &run, provider, &mut execute_tool, clock);
    run.completed_at_ms = Some(clock.tick());
    match &outcome {
        Ok(()) => run.status = AgentRunStatus::Completed,
        Err(error) if error.is_cancelled() => {
            run.status = AgentRunStatus::Cancelled;
            run.error = Some(error.to_string());
        }
        Err(error) => {
            run.status = AgentRunStatus::Failed;
            run.error = Some(error.to_string());
        }
    }
    let run_id = run.id;
    if let Err(error) = record_run(conn, run) {
        return Err(AgentHarnessError::Finalization {
            execution_error: outcome.err().map(Box::new),
            persistence_error: error.to_string(),
        });
    }
    outcome?;
    load_history(conn, run_id)
}

fn execute_run<P, F>(
    conn: &mut Connection,
    workflow: &AgentWorkflowDefinition,
    run: &AgentRun,
    provider: &mut P,
    execute_tool: &mut F,
    clock: &mut AgentHarnessClock,
) -> Result<(), AgentHarnessError>
where
    P: AgentWorkflowProvider,
    F: FnMut(
        &mut Connection,
        &AgentToolRequest,
    ) -> Result<AgentToolResultPayload, AgentHarnessError>,
{
    let mut completed_calls = Vec::new();
    let mut completed_results = Vec::new();
    // A provider may finish after using exactly its tool budget. The final
    // provider turn may declare completion but may not execute another tool.
    for completed in 0..=workflow.budget.max_tool_calls {
        let turn = AgentProviderTurn {
            workflow,
            run,
            completed_calls: &completed_calls,
            completed_results: &completed_results,
        };
        let Some(request) = provider.next_tool_request(turn)? else {
            return Ok(());
        };
        if completed == workflow.budget.max_tool_calls {
            return Err(AgentHarnessError::BudgetExceeded {
                max_tool_calls: workflow.budget.max_tool_calls,
            });
        }
        let mut call = AgentToolCall {
            id: AgentToolCallId::new(),
            run_id: run.id,
            sequence: completed + 1,
            request,
            status: AgentToolCallStatus::Running,
            created_at_ms: clock.tick(),
        };
        if let Err(error) = workflow
            .manifest
            .validate_call(&call.request, &workflow.budget)
        {
            call.status = AgentToolCallStatus::Rejected;
            let result = AgentToolResult {
                call_id: call.id,
                status: AgentToolResultStatus::Rejected,
                payload: AgentToolResultPayload::Rejection {
                    reason: error.to_string(),
                },
                completed_at_ms: clock.tick(),
            };
            return persist_tool_outcome(conn, &call, &result, Err(error.into()));
        }
        // Persist intent before entering an executor that may change the story.
        // Do not automatically retry a failed/incomplete tool attempt.
        record_tool_call(conn, call.clone())?;
        let outcome = execute_tool(conn, &call.request);
        let (status, payload) = match &outcome {
            Ok(payload) => {
                call.status = AgentToolCallStatus::Completed;
                (AgentToolResultStatus::Succeeded, payload.clone())
            }
            Err(error) => {
                call.status = AgentToolCallStatus::Failed;
                (
                    AgentToolResultStatus::Failed,
                    AgentToolResultPayload::Error {
                        message: error.to_string(),
                    },
                )
            }
        };
        let result = AgentToolResult {
            call_id: call.id,
            status,
            payload,
            completed_at_ms: clock.tick(),
        };
        persist_tool_outcome(conn, &call, &result, outcome.map(|_| ()))?;
        completed_calls.push(call);
        completed_results.push(result);
    }
    unreachable!("last provider turn either completes or exceeds the tool budget")
}

// Keep execution and history-write failures distinct. In particular, a failed
// terminal history write must not turn cooperative cancellation into failure.
fn persist_tool_outcome(
    conn: &mut Connection,
    call: &AgentToolCall,
    result: &AgentToolResult,
    outcome: Result<(), AgentHarnessError>,
) -> Result<(), AgentHarnessError> {
    let persistence = record_tool_call(conn, call.clone())
        .and_then(|()| record_tool_result(conn, result.clone()));
    match persistence {
        Ok(()) => outcome,
        Err(error) => Err(AgentHarnessError::Finalization {
            execution_error: outcome.err().map(Box::new),
            persistence_error: error.to_string(),
        }),
    }
}

fn record_run(conn: &mut Connection, run: AgentRun) -> Result<(), AgentHarnessError> {
    agent_workflow_store::record_agent_run(conn, &CommandEnvelope::new(run))?;
    Ok(())
}

fn record_tool_call(conn: &mut Connection, call: AgentToolCall) -> Result<(), AgentHarnessError> {
    agent_workflow_store::record_agent_tool_call(conn, &CommandEnvelope::new(call))?;
    Ok(())
}

fn record_tool_result(
    conn: &mut Connection,
    result: AgentToolResult,
) -> Result<(), AgentHarnessError> {
    agent_workflow_store::record_agent_tool_result(conn, &CommandEnvelope::new(result))?;
    Ok(())
}

fn load_history(
    conn: &Connection,
    run_id: AgentRunId,
) -> Result<AgentRunHistoryProjection, AgentHarnessError> {
    let run = agent_workflow_store::load_agent_run(conn, run_id)?
        .ok_or(AgentHarnessError::MissingRun { run_id })?;
    let calls = agent_workflow_store::load_agent_tool_calls(conn, run_id)?;
    let mut results = Vec::new();
    for call in &calls {
        if let Some(result) = agent_workflow_store::load_agent_tool_result(conn, call.id)? {
            results.push(result);
        }
    }
    Ok(AgentRunHistoryProjection {
        run,
        calls,
        results,
    })
}

#[derive(Debug, thiserror::Error)]
pub enum AgentHarnessError {
    #[error("{0}")]
    Store(String),
    #[error(transparent)]
    Contract(#[from] eidetic_core::contracts::AgentWorkflowContractError),
    #[error("agent workflow exceeded max tool calls {max_tool_calls}")]
    BudgetExceeded { max_tool_calls: u32 },
    #[error("agent run {run_id:?} was not recorded")]
    MissingRun { run_id: AgentRunId },
    #[error("{0}")]
    Tool(String),
    #[error("{0}")]
    Provider(String),
    /// Cooperative cancellation from the provider or executor. It never
    /// implies rollback of a tool that has already committed a command.
    #[error("agent workflow cancelled")]
    Cancelled,
    #[error(
        "failed to persist terminal agent history: {persistence_error}; execution error: {execution_error:?}"
    )]
    Finalization {
        execution_error: Option<Box<AgentHarnessError>>,
        persistence_error: String,
    },
}

impl AgentHarnessError {
    fn is_cancelled(&self) -> bool {
        match self {
            Self::Cancelled => true,
            Self::Finalization {
                execution_error: Some(error),
                ..
            } => error.is_cancelled(),
            _ => false,
        }
    }
}

impl From<HistoryStoreError> for AgentHarnessError {
    fn from(value: HistoryStoreError) -> Self {
        Self::Store(value.to_string())
    }
}

#[cfg(test)]
#[path = "agent_workflow_harness_tests.rs"]
mod tests;
