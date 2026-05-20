use std::path::PathBuf;

use eidetic_core::contracts::{
    CommandEnvelope, ProjectionEnvelope, ScriptDocumentProjection, SetObjectFieldCommand,
    SetScriptBlockCommand, SetScriptLockCommand,
};
use serde::Serialize;

use crate::backend_error::BackendError;
use crate::history_store::{self, HistoryStoreError, RecordChangeOutcome};
use crate::object_field_command::{self, ObjectFieldCommandError};
use crate::revision_projection::ObjectFieldProjection;
use crate::script_document_command::{self, ScriptDocumentCommandError};
use crate::state::{AppState, ServerEvent};

#[derive(Debug, Serialize)]
pub struct ObjectFieldCommandResponse {
    outcome: RecordChangeOutcome,
    projection: ProjectionEnvelope<ObjectFieldProjection>,
}

#[derive(Debug, Serialize)]
pub struct ScriptDocumentCommandResponse {
    outcome: RecordChangeOutcome,
    projection: ProjectionEnvelope<ScriptDocumentProjection>,
}

pub async fn set_object_field(
    state: &AppState,
    command: CommandEnvelope<SetObjectFieldCommand>,
) -> Result<ObjectFieldCommandResponse, BackendError> {
    let path = active_project_path(state)?;
    let response = tokio::task::spawn_blocking(move || apply_object_field_at_path(path, command))
        .await
        .map_err(|error| {
            BackendError::internal(format!("object field command task failed: {error}"))
        })??;

    let _ = state.events_tx.send(ServerEvent::BibleChanged);
    Ok(response)
}

pub async fn set_script_block(
    state: &AppState,
    command: CommandEnvelope<SetScriptBlockCommand>,
) -> Result<ScriptDocumentCommandResponse, BackendError> {
    let path = active_project_path(state)?;
    let response = tokio::task::spawn_blocking(move || set_script_block_at_path(path, command))
        .await
        .map_err(|error| {
            BackendError::internal(format!("script block command task failed: {error}"))
        })??;

    let _ = state.events_tx.send(ServerEvent::ScriptChanged);
    Ok(response)
}

pub async fn set_script_lock(
    state: &AppState,
    command: CommandEnvelope<SetScriptLockCommand>,
) -> Result<ScriptDocumentCommandResponse, BackendError> {
    let path = active_project_path(state)?;
    let response = tokio::task::spawn_blocking(move || set_script_lock_at_path(path, command))
        .await
        .map_err(|error| {
            BackendError::internal(format!("script lock command task failed: {error}"))
        })??;

    let _ = state.events_tx.send(ServerEvent::ScriptChanged);
    Ok(response)
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

fn apply_object_field_at_path(
    path: PathBuf,
    command: CommandEnvelope<SetObjectFieldCommand>,
) -> Result<ObjectFieldCommandResponse, BackendError> {
    let mut conn = crate::sqlite::open_write_connection(&path)
        .map_err(|e| BackendError::internal(e.to_string()))?;
    history_store::create_schema(&conn).map_err(map_history_error)?;
    let (outcome, projection) =
        object_field_command::apply_set_object_field(&mut conn, &command, 0)
            .map_err(map_object_field_error)?;
    let object_kind = projection.object_kind.clone();
    let object_id = projection.object_id.clone();
    let projection = crate::revision_projection::load_object_field_projection_envelope(
        &conn,
        object_kind,
        &object_id,
    )
    .map_err(map_history_error)?;

    Ok(ObjectFieldCommandResponse {
        outcome,
        projection,
    })
}

fn set_script_block_at_path(
    path: PathBuf,
    command: CommandEnvelope<SetScriptBlockCommand>,
) -> Result<ScriptDocumentCommandResponse, BackendError> {
    let mut conn = crate::sqlite::open_write_connection(&path)
        .map_err(|e| BackendError::internal(e.to_string()))?;
    let (outcome, projection) =
        script_document_command::apply_set_script_block(&mut conn, &command, 0)
            .map_err(map_script_document_error)?;

    Ok(ScriptDocumentCommandResponse {
        outcome,
        projection,
    })
}

fn set_script_lock_at_path(
    path: PathBuf,
    command: CommandEnvelope<SetScriptLockCommand>,
) -> Result<ScriptDocumentCommandResponse, BackendError> {
    let mut conn = crate::sqlite::open_write_connection(&path)
        .map_err(|e| BackendError::internal(e.to_string()))?;
    let (outcome, projection) =
        script_document_command::apply_set_script_lock(&mut conn, &command, 0)
            .map_err(map_script_document_error)?;

    Ok(ScriptDocumentCommandResponse {
        outcome,
        projection,
    })
}

fn map_object_field_error(error: ObjectFieldCommandError) -> BackendError {
    match error {
        ObjectFieldCommandError::InvalidCommand(message) => BackendError::bad_request(message),
        ObjectFieldCommandError::History(error) => map_history_error(error),
    }
}

fn map_script_document_error(error: ScriptDocumentCommandError) -> BackendError {
    match error {
        ScriptDocumentCommandError::InvalidCommand(message) => BackendError::bad_request(message),
        ScriptDocumentCommandError::Store(error) => map_history_error(error),
    }
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
