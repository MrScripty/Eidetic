use eidetic_core::contracts::{
    CommandEnvelope, ContextInfluenceProjection, ContextInfluenceProjectionRequest,
    ContextStackProjection, ContextStackProjectionRequest, ObjectKind, ProjectionEnvelope,
    ProjectionVersion, RecordContextEvaluationCommand,
};

use crate::backend_error::BackendError;
use crate::command_service_support::{active_project_path, map_history_error};
use crate::context_influence_store;
use crate::history_store;
use crate::state::AppState;
use crate::timeline_node_store;

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

pub async fn context_stack_projection(
    state: &AppState,
    request: ContextStackProjectionRequest,
) -> Result<ProjectionEnvelope<ContextStackProjection>, BackendError> {
    let path = active_project_path(state)?;
    tokio::task::spawn_blocking(move || load_context_stack_projection_at_path(path, request))
        .await
        .map_err(|error| {
            BackendError::internal(format!("context stack projection task failed: {error}"))
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

fn load_context_stack_projection_at_path(
    path: std::path::PathBuf,
    request: ContextStackProjectionRequest,
) -> Result<ProjectionEnvelope<ContextStackProjection>, BackendError> {
    let conn = crate::sqlite::open_write_connection(&path)
        .map_err(|e| BackendError::internal(e.to_string()))?;
    let nodes = timeline_node_store::load_node_ancestor_stack(&conn, request.target_node_id)
        .map_err(map_history_error)?;
    let projection = ContextStackProjection::from_nodes(&nodes, request.target_node_id)
        .ok_or_else(|| BackendError::not_found("context stack target node not found"))?;
    let summary = history_store::load_revision_summary_for_kind(&conn, ObjectKind::TimelineNode)
        .map_err(map_history_error)?;

    match summary.latest_change_event_id {
        Some(change_event_id) => Ok(ProjectionEnvelope::from_event(
            ProjectionVersion(summary.revision_count + 1),
            change_event_id,
            projection,
        )),
        None => Ok(ProjectionEnvelope::initial(projection)),
    }
}
