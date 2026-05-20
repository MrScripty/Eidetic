use std::path::PathBuf;

use eidetic_core::contracts::{
    ObjectKind, ProjectionEnvelope, ScriptDocumentId, ScriptDocumentProjection,
    StoryArcListProjection,
};
use serde::Deserialize;

use crate::backend_error::BackendError;
use crate::history_store::{self, HistoryStoreError};
use crate::script_store;
use crate::state::AppState;
use crate::story_arc_store;

#[derive(Debug, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct ObjectFieldProjectionRequest {
    pub object_kind: ObjectKind,
    pub object_id: String,
}

#[derive(Debug, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct ScriptDocumentProjectionRequest {
    pub document_id: ScriptDocumentId,
}

pub async fn object_field_projection(
    state: &AppState,
    request: ObjectFieldProjectionRequest,
) -> Result<serde_json::Value, BackendError> {
    if request.object_id.trim().is_empty() {
        return Err(BackendError::bad_request("object_id is required"));
    }

    let path = active_project_path(state)?;
    tokio::task::spawn_blocking(move || {
        load_object_field_projection_value_at_path(path, request.object_kind, request.object_id)
    })
    .await
    .map_err(|error| {
        BackendError::internal(format!("object field projection task failed: {error}"))
    })?
}

pub async fn script_document_projection(
    state: &AppState,
    request: ScriptDocumentProjectionRequest,
) -> Result<ProjectionEnvelope<ScriptDocumentProjection>, BackendError> {
    let path = active_project_path(state)?;
    tokio::task::spawn_blocking(move || {
        load_script_document_projection_at_path(path, request.document_id)
    })
    .await
    .map_err(|error| {
        BackendError::internal(format!("script document projection task failed: {error}"))
    })?
}

pub async fn story_arc_list_projection(
    state: &AppState,
) -> Result<ProjectionEnvelope<StoryArcListProjection>, BackendError> {
    let path = active_project_path(state)?;
    tokio::task::spawn_blocking(move || load_story_arc_list_projection_at_path(path))
        .await
        .map_err(|error| {
            BackendError::internal(format!("story arc list projection task failed: {error}"))
        })?
}

fn active_project_path(state: &AppState) -> Result<PathBuf, BackendError> {
    if state.project.lock().is_none() {
        return Err(BackendError::no_project());
    }
    state
        .project_database
        .active_path()
        .ok_or_else(BackendError::no_project)
}

fn load_object_field_projection_value_at_path(
    path: PathBuf,
    object_kind: ObjectKind,
    object_id: String,
) -> Result<serde_json::Value, BackendError> {
    let conn = crate::sqlite::open_write_connection(&path)
        .map_err(|e| BackendError::internal(e.to_string()))?;
    history_store::create_schema(&conn).map_err(map_history_error)?;
    let projection = crate::revision_projection::load_object_field_projection_envelope(
        &conn,
        object_kind,
        &object_id,
    )
    .map_err(map_history_error)?;
    serde_json::to_value(projection).map_err(|e| BackendError::internal(e.to_string()))
}

fn load_script_document_projection_at_path(
    path: PathBuf,
    document_id: ScriptDocumentId,
) -> Result<ProjectionEnvelope<ScriptDocumentProjection>, BackendError> {
    let conn = crate::sqlite::open_write_connection(&path)
        .map_err(|e| BackendError::internal(e.to_string()))?;
    script_store::create_schema(&conn).map_err(map_history_error)?;
    script_store::load_document_projection_envelope(&conn, &document_id)
        .map_err(map_history_error)?
        .ok_or_else(|| BackendError::not_found("script document not found"))
}

fn load_story_arc_list_projection_at_path(
    path: PathBuf,
) -> Result<ProjectionEnvelope<StoryArcListProjection>, BackendError> {
    let conn = crate::sqlite::open_write_connection(&path)
        .map_err(|e| BackendError::internal(e.to_string()))?;
    story_arc_store::create_schema(&conn).map_err(map_history_error)?;
    story_arc_store::load_arc_list_projection_envelope(&conn).map_err(map_history_error)
}

fn map_history_error(error: HistoryStoreError) -> BackendError {
    match error {
        HistoryStoreError::InvalidValue(message) => BackendError::conflict(message),
        HistoryStoreError::InvalidId(message) => BackendError::bad_request(message),
        HistoryStoreError::MissingColumn(message) => BackendError::internal(message),
        HistoryStoreError::Sqlite(error) => BackendError::internal(error.to_string()),
        HistoryStoreError::Json(error) => BackendError::bad_request(error.to_string()),
    }
}
