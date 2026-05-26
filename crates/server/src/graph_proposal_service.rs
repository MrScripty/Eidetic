use eidetic_core::contracts::{
    CommandEnvelope, CreateGraphProposalCommand, GraphProposalListProjection, ProjectionEnvelope,
};
use serde::Serialize;

use crate::backend_error::BackendError;
use crate::command_service_support::{active_project_path, map_history_error};
use crate::graph_proposal_store;
use crate::history_store::RecordChangeOutcome;
use crate::state::AppState;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize)]
#[serde(rename_all = "snake_case")]
pub enum GraphProposalCommandOutcome {
    Recorded,
    AlreadyRecorded,
}

impl From<RecordChangeOutcome> for GraphProposalCommandOutcome {
    fn from(value: RecordChangeOutcome) -> Self {
        match value {
            RecordChangeOutcome::Recorded => Self::Recorded,
            RecordChangeOutcome::AlreadyRecorded => Self::AlreadyRecorded,
        }
    }
}

#[derive(Debug, Serialize)]
pub struct GraphProposalCommandResponse {
    pub outcome: GraphProposalCommandOutcome,
    pub projection: ProjectionEnvelope<GraphProposalListProjection>,
}

pub async fn create_graph_proposal(
    state: &AppState,
    command: CommandEnvelope<CreateGraphProposalCommand>,
) -> Result<GraphProposalCommandResponse, BackendError> {
    let path = active_project_path(state)?;
    tokio::task::spawn_blocking(move || create_graph_proposal_at_path(path, command))
        .await
        .map_err(|error| BackendError::internal(format!("graph proposal task failed: {error}")))?
}

pub async fn graph_proposal_list_projection(
    state: &AppState,
) -> Result<ProjectionEnvelope<GraphProposalListProjection>, BackendError> {
    let path = active_project_path(state)?;
    tokio::task::spawn_blocking(move || load_graph_proposal_list_projection_at_path(path))
        .await
        .map_err(|error| {
            BackendError::internal(format!("graph proposal projection task failed: {error}"))
        })?
}

fn create_graph_proposal_at_path(
    path: std::path::PathBuf,
    command: CommandEnvelope<CreateGraphProposalCommand>,
) -> Result<GraphProposalCommandResponse, BackendError> {
    let mut conn = crate::sqlite::open_write_connection(&path)
        .map_err(|error| BackendError::internal(error.to_string()))?;
    let outcome = graph_proposal_store::record_create_graph_proposal(&mut conn, &command, 0)
        .map_err(map_history_error)?;
    let projection = graph_proposal_store::load_graph_proposal_list_projection(&conn)
        .map_err(map_history_error)?;
    Ok(GraphProposalCommandResponse {
        outcome: outcome.into(),
        projection,
    })
}

fn load_graph_proposal_list_projection_at_path(
    path: std::path::PathBuf,
) -> Result<ProjectionEnvelope<GraphProposalListProjection>, BackendError> {
    let conn = crate::sqlite::open_write_connection(&path)
        .map_err(|error| BackendError::internal(error.to_string()))?;
    graph_proposal_store::load_graph_proposal_list_projection(&conn).map_err(map_history_error)
}

#[cfg(test)]
mod tests {
    use super::*;
    use eidetic_core::contracts::{
        BibleGraphNodeId, BibleGraphSchemaKey, CommandEnvelope, GraphProposalAction,
        GraphProposalId, GraphProposalTarget,
    };

    #[test]
    fn graph_proposal_service_records_and_loads_projection() {
        let path = std::env::temp_dir().join(format!(
            "eidetic-graph-proposal-service-{}.sqlite",
            uuid::Uuid::new_v4()
        ));
        let command = CommandEnvelope::new(CreateGraphProposalCommand {
            proposal_id: GraphProposalId::new("proposal.graph.ada").unwrap(),
            action: GraphProposalAction::CreateBibleNode,
            target: GraphProposalTarget::BibleNode {
                node_id: BibleGraphNodeId::new("node.character.ada").unwrap(),
                parent_id: BibleGraphNodeId::new("canonical.characters").unwrap(),
                schema_key: BibleGraphSchemaKey::new("canonical.character").unwrap(),
                title: "Ada".to_string(),
            },
            summary: "Create Ada".to_string(),
            proposed_value: None,
            rationale: Some("Premise names Ada.".to_string()),
            source_agent_run_id: None,
            source_tool_call_id: None,
        });

        let response = create_graph_proposal_at_path(path.clone(), command).unwrap();
        let projection = load_graph_proposal_list_projection_at_path(path.clone()).unwrap();

        assert_eq!(response.outcome, GraphProposalCommandOutcome::Recorded);
        assert_eq!(response.projection.payload.proposals.len(), 1);
        assert_eq!(projection.payload.proposals.len(), 1);
        let _ = std::fs::remove_file(path);
    }
}
