use eidetic_core::contracts::{
    CommandEnvelope, ContextInfluenceProjection, ContextInfluenceProjectionRequest,
    ProjectionEnvelope, RecordContextEvaluationCommand,
};

use crate::backend_error::BackendError;
use crate::command_service_support::{active_project_path, map_history_error};
use crate::context_influence_store;
use crate::state::AppState;

pub async fn record_context_evaluation(
    state: &AppState,
    command: CommandEnvelope<RecordContextEvaluationCommand>,
) -> Result<ProjectionEnvelope<ContextInfluenceProjection>, BackendError> {
    let path = active_project_path(state)?;
    tokio::task::spawn_blocking(move || record_context_evaluation_at_path(path, command))
        .await
        .map_err(|error| {
            BackendError::internal(format!("context evaluation task failed: {error}"))
        })?
}

pub async fn context_influence_projection(
    state: &AppState,
    request: ContextInfluenceProjectionRequest,
) -> Result<ProjectionEnvelope<ContextInfluenceProjection>, BackendError> {
    let path = active_project_path(state)?;
    tokio::task::spawn_blocking(move || load_context_influence_projection_at_path(path, request))
        .await
        .map_err(|error| {
            BackendError::internal(format!("context influence projection task failed: {error}"))
        })?
}

fn record_context_evaluation_at_path(
    path: std::path::PathBuf,
    command: CommandEnvelope<RecordContextEvaluationCommand>,
) -> Result<ProjectionEnvelope<ContextInfluenceProjection>, BackendError> {
    let mut conn = crate::sqlite::open_write_connection(&path)
        .map_err(|e| BackendError::internal(e.to_string()))?;
    context_influence_store::record_context_evaluation(&mut conn, &command, 0)
        .map_err(map_history_error)?;
    let projection = context_influence_store::load_context_influence_projection(
        &conn,
        command.payload.evaluation.target_node_id,
    )
    .map_err(map_history_error)?;
    projection
        .ok_or_else(|| BackendError::internal("missing recorded context influence projection"))
}

fn load_context_influence_projection_at_path(
    path: std::path::PathBuf,
    request: ContextInfluenceProjectionRequest,
) -> Result<ProjectionEnvelope<ContextInfluenceProjection>, BackendError> {
    let conn = crate::sqlite::open_write_connection(&path)
        .map_err(|e| BackendError::internal(e.to_string()))?;
    context_influence_store::load_context_influence_projection(&conn, request.target_node_id)
        .map_err(map_history_error)?
        .ok_or_else(|| BackendError::not_found("context influence projection not found"))
}
