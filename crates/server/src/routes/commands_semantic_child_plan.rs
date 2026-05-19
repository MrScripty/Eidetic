use axum::extract::State;
use axum::routing::post;
use axum::{Json, Router};
use eidetic_core::ai::backend::{ChildPlanListProjection, RejectChildPlanCommand};
use eidetic_core::contracts::{CommandEnvelope, ProjectionEnvelope};
use serde::Serialize;

use crate::child_plan_projection_store;
use crate::child_plan_review;
use crate::child_plan_store::ChildPlanStoreError;
use crate::error::{ApiError, ApiJson};
use crate::history_store::RecordChangeOutcome;
use crate::state::{AppState, ServerEvent};

use super::support::{active_project_path, map_history_error};

pub fn router() -> Router<AppState> {
    Router::new().route(
        "/commands/semantic/child-plan/reject",
        post(reject_child_plan),
    )
}

#[derive(Debug, Serialize)]
struct ChildPlanCommandResponse {
    outcome: RecordChangeOutcome,
    projection: ProjectionEnvelope<ChildPlanListProjection>,
}

async fn reject_child_plan(
    State(state): State<AppState>,
    Json(command): Json<CommandEnvelope<RejectChildPlanCommand>>,
) -> ApiJson {
    let path = active_project_path(&state)?;
    let response = tokio::task::spawn_blocking(move || reject_child_plan_at_path(path, command))
        .await
        .map_err(|e| ApiError::internal(format!("child plan reject task failed: {e}")))??;

    if response.outcome == RecordChangeOutcome::Recorded {
        let _ = state.events_tx.send(ServerEvent::SemanticProposalsChanged);
    }
    crate::error::json_value(response)
}

fn reject_child_plan_at_path(
    path: std::path::PathBuf,
    command: CommandEnvelope<RejectChildPlanCommand>,
) -> Result<ChildPlanCommandResponse, ApiError> {
    let mut conn = crate::sqlite::open_write_connection(&path)
        .map_err(|e| ApiError::internal(e.to_string()))?;
    let outcome = child_plan_review::record_reject_child_plan(&mut conn, &command, 0)
        .map_err(map_child_plan_error)?;
    let projection = child_plan_projection_store::load_child_plan_list_projection(&conn)
        .map_err(map_child_plan_error)?;

    Ok(ChildPlanCommandResponse {
        outcome,
        projection,
    })
}

fn map_child_plan_error(error: ChildPlanStoreError) -> ApiError {
    match error {
        ChildPlanStoreError::InvalidCommand(message) => ApiError::bad_request(message),
        ChildPlanStoreError::NotFound(message) => ApiError::not_found(message),
        ChildPlanStoreError::History(error) => map_history_error(error),
        ChildPlanStoreError::Sqlite(error) => ApiError::internal(error.to_string()),
    }
}

#[cfg(test)]
#[path = "commands_semantic_child_plan_tests.rs"]
mod tests;
