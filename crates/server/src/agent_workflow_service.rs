use eidetic_core::contracts::{
    AgentRun, AgentRunId, AgentToolCall, AgentToolResult, CommandEnvelope,
};
use serde::Serialize;

use crate::agent_workflow_store;
use crate::backend_error::BackendError;
use crate::command_service_support::{active_project_path, map_history_error};
use crate::history_store::RecordChangeOutcome;
use crate::state::AppState;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize)]
#[serde(rename_all = "snake_case")]
pub enum AgentCommandOutcome {
    Recorded,
    AlreadyRecorded,
}

impl From<RecordChangeOutcome> for AgentCommandOutcome {
    fn from(value: RecordChangeOutcome) -> Self {
        match value {
            RecordChangeOutcome::Recorded => Self::Recorded,
            RecordChangeOutcome::AlreadyRecorded => Self::AlreadyRecorded,
        }
    }
}

#[derive(Debug, Serialize)]
pub struct AgentRunCommandResponse {
    pub outcome: AgentCommandOutcome,
    pub run: AgentRun,
}

#[derive(Debug, Serialize)]
pub struct AgentToolCallCommandResponse {
    pub outcome: AgentCommandOutcome,
    pub call: AgentToolCall,
}

#[derive(Debug, Serialize)]
pub struct AgentToolResultCommandResponse {
    pub outcome: AgentCommandOutcome,
    pub result: AgentToolResult,
}

#[derive(Debug, Serialize)]
pub struct AgentRunHistoryProjection {
    pub run: AgentRun,
    pub calls: Vec<AgentToolCall>,
    pub results: Vec<AgentToolResult>,
}

pub async fn record_agent_run(
    state: &AppState,
    command: CommandEnvelope<AgentRun>,
) -> Result<AgentRunCommandResponse, BackendError> {
    let path = active_project_path(state)?;
    tokio::task::spawn_blocking(move || record_agent_run_at_path(path, command))
        .await
        .map_err(|error| BackendError::internal(format!("agent run task failed: {error}")))?
}

pub async fn record_agent_tool_call(
    state: &AppState,
    command: CommandEnvelope<AgentToolCall>,
) -> Result<AgentToolCallCommandResponse, BackendError> {
    let path = active_project_path(state)?;
    tokio::task::spawn_blocking(move || record_agent_tool_call_at_path(path, command))
        .await
        .map_err(|error| BackendError::internal(format!("agent tool call task failed: {error}")))?
}

pub async fn record_agent_tool_result(
    state: &AppState,
    command: CommandEnvelope<AgentToolResult>,
) -> Result<AgentToolResultCommandResponse, BackendError> {
    let path = active_project_path(state)?;
    tokio::task::spawn_blocking(move || record_agent_tool_result_at_path(path, command))
        .await
        .map_err(|error| {
            BackendError::internal(format!("agent tool result task failed: {error}"))
        })?
}

pub async fn load_agent_run_history(
    state: &AppState,
    run_id: AgentRunId,
) -> Result<AgentRunHistoryProjection, BackendError> {
    let path = active_project_path(state)?;
    tokio::task::spawn_blocking(move || load_agent_run_history_at_path(path, run_id))
        .await
        .map_err(|error| BackendError::internal(format!("agent history task failed: {error}")))?
}

fn record_agent_run_at_path(
    path: std::path::PathBuf,
    command: CommandEnvelope<AgentRun>,
) -> Result<AgentRunCommandResponse, BackendError> {
    let mut conn = crate::sqlite::open_write_connection(&path)
        .map_err(|error| BackendError::internal(error.to_string()))?;
    let outcome =
        agent_workflow_store::record_agent_run(&mut conn, &command).map_err(map_history_error)?;
    let run = agent_workflow_store::load_agent_run(&conn, command.payload.id)
        .map_err(map_history_error)?
        .ok_or_else(|| BackendError::internal("missing recorded agent run"))?;
    Ok(AgentRunCommandResponse {
        outcome: outcome.into(),
        run,
    })
}

fn record_agent_tool_call_at_path(
    path: std::path::PathBuf,
    command: CommandEnvelope<AgentToolCall>,
) -> Result<AgentToolCallCommandResponse, BackendError> {
    let mut conn = crate::sqlite::open_write_connection(&path)
        .map_err(|error| BackendError::internal(error.to_string()))?;
    let outcome = agent_workflow_store::record_agent_tool_call(&mut conn, &command)
        .map_err(map_history_error)?;
    let call = agent_workflow_store::load_agent_tool_calls(&conn, command.payload.run_id)
        .map_err(map_history_error)?
        .into_iter()
        .find(|call| call.id == command.payload.id)
        .ok_or_else(|| BackendError::internal("missing recorded agent tool call"))?;
    Ok(AgentToolCallCommandResponse {
        outcome: outcome.into(),
        call,
    })
}

fn record_agent_tool_result_at_path(
    path: std::path::PathBuf,
    command: CommandEnvelope<AgentToolResult>,
) -> Result<AgentToolResultCommandResponse, BackendError> {
    let mut conn = crate::sqlite::open_write_connection(&path)
        .map_err(|error| BackendError::internal(error.to_string()))?;
    let outcome = agent_workflow_store::record_agent_tool_result(&mut conn, &command)
        .map_err(map_history_error)?;
    let result = agent_workflow_store::load_agent_tool_result(&conn, command.payload.call_id)
        .map_err(map_history_error)?
        .ok_or_else(|| BackendError::internal("missing recorded agent tool result"))?;
    Ok(AgentToolResultCommandResponse {
        outcome: outcome.into(),
        result,
    })
}

fn load_agent_run_history_at_path(
    path: std::path::PathBuf,
    run_id: AgentRunId,
) -> Result<AgentRunHistoryProjection, BackendError> {
    let conn = crate::sqlite::open_write_connection(&path)
        .map_err(|error| BackendError::internal(error.to_string()))?;
    let run = agent_workflow_store::load_agent_run(&conn, run_id)
        .map_err(map_history_error)?
        .ok_or_else(|| BackendError::not_found("agent run not found"))?;
    let calls =
        agent_workflow_store::load_agent_tool_calls(&conn, run_id).map_err(map_history_error)?;
    let mut results = Vec::new();
    for call in &calls {
        if let Some(result) = agent_workflow_store::load_agent_tool_result(&conn, call.id)
            .map_err(map_history_error)?
        {
            results.push(result);
        }
    }
    Ok(AgentRunHistoryProjection {
        run,
        calls,
        results,
    })
}

#[cfg(test)]
mod tests {
    use super::*;
    use eidetic_core::contracts::{
        AgentRunStatus, AgentToolArguments, AgentToolCallId, AgentToolCallStatus, AgentToolName,
        AgentToolRequest, AgentToolResultPayload, AgentToolResultStatus, AgentWorkflowId,
        AgentWorkflowIntent, BibleGraphNodeId,
    };

    #[test]
    fn agent_run_history_loads_calls_and_results() {
        let path = std::env::temp_dir().join(format!(
            "eidetic-agent-run-history-{}.sqlite",
            uuid::Uuid::new_v4()
        ));
        let run = agent_run();
        let call = agent_tool_call(run.id);
        let result = agent_tool_result(call.id);

        record_agent_run_at_path(path.clone(), CommandEnvelope::new(run.clone())).unwrap();
        record_agent_tool_call_at_path(path.clone(), CommandEnvelope::new(call.clone())).unwrap();
        record_agent_tool_result_at_path(path.clone(), CommandEnvelope::new(result.clone()))
            .unwrap();

        let projection = load_agent_run_history_at_path(path.clone(), run.id).unwrap();

        assert_eq!(projection.run, run);
        assert_eq!(projection.calls, vec![call]);
        assert_eq!(projection.results, vec![result]);
        let _ = std::fs::remove_file(path);
    }

    fn agent_run() -> AgentRun {
        AgentRun {
            id: AgentRunId::new(),
            workflow_id: AgentWorkflowId::new("workflow.premise.graph").unwrap(),
            status: AgentRunStatus::Running,
            intent: AgentWorkflowIntent::DevelopPremiseGraphContext,
            created_at_ms: 1,
            completed_at_ms: None,
            error: None,
        }
    }

    fn agent_tool_call(run_id: AgentRunId) -> AgentToolCall {
        AgentToolCall {
            id: AgentToolCallId::new(),
            run_id,
            sequence: 1,
            request: AgentToolRequest {
                tool_name: AgentToolName::new("read_bible_node").unwrap(),
                arguments: AgentToolArguments::ReadBibleNode {
                    node_id: BibleGraphNodeId::new("node.character.ada").unwrap(),
                },
            },
            status: AgentToolCallStatus::Completed,
            created_at_ms: 2,
        }
    }

    fn agent_tool_result(call_id: AgentToolCallId) -> AgentToolResult {
        AgentToolResult {
            call_id,
            status: AgentToolResultStatus::Succeeded,
            payload: AgentToolResultPayload::Text {
                text: "Ada is relevant.".to_string(),
            },
            completed_at_ms: 3,
        }
    }
}
