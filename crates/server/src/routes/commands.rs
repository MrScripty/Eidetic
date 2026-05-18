use std::path::PathBuf;

use axum::extract::State;
use axum::routing::post;
use axum::{Json, Router};
use eidetic_core::contracts::{CommandEnvelope, ProjectionEnvelope, SetObjectFieldCommand};
use rusqlite::Connection;
use serde::Serialize;

use crate::error::{ApiError, ApiJson};
use crate::history_store::{self, HistoryStoreError, RecordChangeOutcome};
use crate::object_field_command::{self, ObjectFieldCommandError};
use crate::revision_projection::ObjectFieldProjection;
use crate::state::{AppState, ServerEvent};

pub fn router() -> Router<AppState> {
    Router::new().route("/commands/object-field", post(set_object_field))
}

#[derive(Debug, Serialize)]
struct ObjectFieldCommandResponse {
    outcome: RecordChangeOutcome,
    projection: ProjectionEnvelope<ObjectFieldProjection>,
}

async fn set_object_field(
    State(state): State<AppState>,
    Json(command): Json<CommandEnvelope<SetObjectFieldCommand>>,
) -> ApiJson {
    let path = active_project_path(&state)?;
    let response = tokio::task::spawn_blocking(move || apply_command_at_path(path, command))
        .await
        .map_err(|e| ApiError::internal(format!("object field command task failed: {e}")))??;

    let _ = state.events_tx.send(ServerEvent::BibleChanged);
    crate::error::json_value(response)
}

fn active_project_path(state: &AppState) -> Result<PathBuf, ApiError> {
    if state.project.lock().is_none() {
        return Err(ApiError::no_project());
    }
    state
        .project_path
        .lock()
        .clone()
        .ok_or_else(ApiError::no_project)
}

fn apply_command_at_path(
    path: PathBuf,
    command: CommandEnvelope<SetObjectFieldCommand>,
) -> Result<ObjectFieldCommandResponse, ApiError> {
    let mut conn = Connection::open(path).map_err(|e| ApiError::internal(e.to_string()))?;
    conn.execute_batch(
        "PRAGMA journal_mode = WAL;
         PRAGMA synchronous = NORMAL;
         PRAGMA foreign_keys = ON;",
    )
    .map_err(|e| ApiError::internal(e.to_string()))?;
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

fn map_object_field_error(error: ObjectFieldCommandError) -> ApiError {
    match error {
        ObjectFieldCommandError::InvalidCommand(message) => ApiError::bad_request(message),
        ObjectFieldCommandError::History(error) => map_history_error(error),
    }
}

fn map_history_error(error: HistoryStoreError) -> ApiError {
    match error {
        HistoryStoreError::InvalidValue(message) => ApiError::conflict(message),
        HistoryStoreError::InvalidId(message) => ApiError::bad_request(message),
        HistoryStoreError::MissingColumn(message) => ApiError::internal(message),
        HistoryStoreError::Sqlite(error) => ApiError::internal(error.to_string()),
        HistoryStoreError::Json(error) => ApiError::bad_request(error.to_string()),
    }
}

#[cfg(test)]
#[path = "commands_tests.rs"]
mod tests;
