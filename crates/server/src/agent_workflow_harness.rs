use eidetic_core::contracts::{
    AgentRun, AgentRunId, AgentRunStatus, AgentToolCall, AgentToolCallId, AgentToolCallStatus,
    AgentToolRequest, AgentToolResult, AgentToolResultPayload, AgentToolResultStatus,
    AgentWorkflowDefinition, CommandEnvelope,
};
use rusqlite::Connection;

use crate::agent_workflow_service::AgentRunHistoryProjection;
use crate::agent_workflow_store;
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

    let mut completed_calls = Vec::new();
    let mut completed_results = Vec::new();
    for sequence in 1..=workflow.budget.max_tool_calls {
        let turn = AgentProviderTurn {
            workflow: &workflow,
            run: &run,
            completed_calls: &completed_calls,
            completed_results: &completed_results,
        };
        let Some(request) = provider.next_tool_request(turn)? else {
            run.status = AgentRunStatus::Completed;
            run.completed_at_ms = Some(clock.tick());
            record_run(conn, run.clone())?;
            return Ok(load_history(conn, run.id)?);
        };
        workflow
            .manifest
            .validate_call(&request, &workflow.budget)?;

        let payload = execute_tool(conn, &request)?;
        let call = AgentToolCall {
            id: AgentToolCallId::new(),
            run_id: run.id,
            sequence,
            request,
            status: AgentToolCallStatus::Completed,
            created_at_ms: clock.tick(),
        };
        let result = AgentToolResult {
            call_id: call.id,
            status: AgentToolResultStatus::Succeeded,
            payload,
            completed_at_ms: clock.tick(),
        };
        record_tool_call(conn, call.clone())?;
        record_tool_result(conn, result.clone())?;
        completed_calls.push(call);
        completed_results.push(result);
    }

    run.status = AgentRunStatus::Failed;
    run.completed_at_ms = Some(clock.tick());
    run.error = Some("agent workflow exceeded max tool calls".to_string());
    record_run(conn, run)?;
    Err(AgentHarnessError::BudgetExceeded {
        max_tool_calls: workflow.budget.max_tool_calls,
    })
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
}

impl From<HistoryStoreError> for AgentHarnessError {
    fn from(value: HistoryStoreError) -> Self {
        Self::Store(value.to_string())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use eidetic_core::contracts::{
        AgentToolArguments, AgentToolBudget, AgentToolDefinition, AgentToolKind, AgentToolManifest,
        AgentToolName, AgentWorkflowId, AgentWorkflowIntent, AgentWorkflowPolicy, BibleGraphNodeId,
    };

    #[test]
    fn mock_provider_harness_records_validated_tool_history() {
        let mut conn = Connection::open_in_memory().unwrap();
        let tool_name = AgentToolName::new("read_bible_node").unwrap();
        let workflow = workflow_with_tool(tool_name.clone(), AgentToolKind::GraphRead, 4);
        let request = AgentToolRequest {
            tool_name,
            arguments: AgentToolArguments::ReadBibleNode {
                node_id: BibleGraphNodeId::new("node.character.ada").unwrap(),
            },
        };
        let mut provider = MockProvider {
            requests: vec![request],
        };
        let mut tools = MockTools;
        let mut clock = AgentHarnessClock::new(10);

        let history =
            run_mockable_agent_workflow(&mut conn, workflow, &mut provider, &mut tools, &mut clock)
                .unwrap();

        assert_eq!(history.run.status, AgentRunStatus::Completed);
        assert_eq!(history.calls.len(), 1);
        assert_eq!(history.calls[0].status, AgentToolCallStatus::Completed);
        assert_eq!(history.results.len(), 1);
        assert_eq!(history.results[0].status, AgentToolResultStatus::Succeeded);
        assert_eq!(
            history.results[0].payload,
            AgentToolResultPayload::Text {
                text: "mock tool result".to_string()
            }
        );
    }

    #[test]
    fn mock_provider_harness_rejects_disallowed_tool_calls_before_execution() {
        let mut conn = Connection::open_in_memory().unwrap();
        let workflow = workflow_with_tool(
            AgentToolName::new("read_bible_node").unwrap(),
            AgentToolKind::GraphRead,
            4,
        );
        let request = AgentToolRequest {
            tool_name: AgentToolName::new("propose_bible_node").unwrap(),
            arguments: AgentToolArguments::ProposeBibleNode {
                command_id: eidetic_core::contracts::CommandId::new(),
                parent_id: BibleGraphNodeId::new("canon.characters").unwrap(),
                schema_key: eidetic_core::contracts::BibleGraphSchemaKey::new(
                    "canonical.character",
                )
                .unwrap(),
                title: "Ada".to_string(),
                summary: "Premise character".to_string(),
            },
        };
        let mut provider = MockProvider {
            requests: vec![request],
        };
        let mut tools = MockTools;
        let mut clock = AgentHarnessClock::new(10);

        let error =
            run_mockable_agent_workflow(&mut conn, workflow, &mut provider, &mut tools, &mut clock)
                .unwrap_err();

        assert!(matches!(
            error,
            AgentHarnessError::Contract(
                eidetic_core::contracts::AgentWorkflowContractError::ToolNotAllowed { .. }
            )
        ));
    }

    #[test]
    fn mock_provider_harness_fails_closed_on_tool_budget_exhaustion() {
        let mut conn = Connection::open_in_memory().unwrap();
        let tool_name = AgentToolName::new("read_bible_node").unwrap();
        let workflow = workflow_with_tool(tool_name.clone(), AgentToolKind::GraphRead, 1);
        let request = AgentToolRequest {
            tool_name,
            arguments: AgentToolArguments::ReadBibleNode {
                node_id: BibleGraphNodeId::new("node.character.ada").unwrap(),
            },
        };
        let mut provider = MockProvider {
            requests: vec![request.clone(), request],
        };
        let mut tools = MockTools;
        let mut clock = AgentHarnessClock::new(10);

        let error =
            run_mockable_agent_workflow(&mut conn, workflow, &mut provider, &mut tools, &mut clock)
                .unwrap_err();

        assert!(matches!(
            error,
            AgentHarnessError::BudgetExceeded { max_tool_calls: 1 }
        ));
    }

    fn workflow_with_tool(
        tool_name: AgentToolName,
        kind: AgentToolKind,
        max_tool_calls: u32,
    ) -> AgentWorkflowDefinition {
        AgentWorkflowDefinition {
            id: AgentWorkflowId::new("workflow.premise.graph").unwrap(),
            label: "Premise graph".to_string(),
            intent: AgentWorkflowIntent::DevelopPremiseGraphContext,
            manifest: AgentToolManifest {
                tools: vec![AgentToolDefinition {
                    name: tool_name,
                    kind,
                    description: "Mock tool".to_string(),
                }],
            },
            budget: AgentToolBudget {
                max_tool_calls,
                ..AgentToolBudget::default()
            },
            policy: AgentWorkflowPolicy::default(),
        }
    }

    struct MockProvider {
        requests: Vec<AgentToolRequest>,
    }

    impl AgentWorkflowProvider for MockProvider {
        fn next_tool_request(
            &mut self,
            _turn: AgentProviderTurn<'_>,
        ) -> Result<Option<AgentToolRequest>, AgentHarnessError> {
            if self.requests.is_empty() {
                return Ok(None);
            }
            Ok(Some(self.requests.remove(0)))
        }
    }

    struct MockTools;

    impl AgentWorkflowToolExecutor for MockTools {
        fn execute_tool(
            &mut self,
            _request: &AgentToolRequest,
        ) -> Result<AgentToolResultPayload, AgentHarnessError> {
            Ok(AgentToolResultPayload::Text {
                text: "mock tool result".to_string(),
            })
        }
    }
}
