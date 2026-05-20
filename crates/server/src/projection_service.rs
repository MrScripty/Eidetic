use std::path::PathBuf;

use eidetic_core::contracts::ObjectKind;
use serde::Deserialize;

use crate::backend_error::BackendError;
use crate::history_store::{self, HistoryStoreError};
use crate::state::AppState;

#[derive(Debug, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct ObjectFieldProjectionRequest {
    pub object_kind: ObjectKind,
    pub object_id: String,
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

fn map_history_error(error: HistoryStoreError) -> BackendError {
    match error {
        HistoryStoreError::InvalidValue(message) => BackendError::conflict(message),
        HistoryStoreError::InvalidId(message) => BackendError::bad_request(message),
        HistoryStoreError::MissingColumn(message) => BackendError::internal(message),
        HistoryStoreError::Sqlite(error) => BackendError::internal(error.to_string()),
        HistoryStoreError::Json(error) => BackendError::bad_request(error.to_string()),
    }
}
