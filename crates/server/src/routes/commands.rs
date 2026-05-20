use axum::extract::State;
use axum::routing::post;
use axum::{Json, Router};
use eidetic_core::contracts::{
    CommandEnvelope, CommandId, CreateStoryArcCommand, DeleteStoryArcCommand, ProjectionEnvelope,
    ScriptDocumentProjection, SetObjectFieldCommand, SetScriptBlockCommand, SetScriptLockCommand,
    SetStoryArcMetadataCommand, StoryArcListProjection,
};
use eidetic_core::story::arc::ArcId;
use serde::{Deserialize, Serialize};
use uuid::Uuid;

use crate::error::{ApiError, ApiJson};
use crate::history_store::RecordChangeOutcome;
use crate::script_document_command::{self, ScriptDocumentCommandError};
use crate::state::{AppState, ServerEvent};
use crate::story_arc_command::{self, StoryArcCommandError};
use crate::story_arc_store;

use super::support::{active_project_path, map_history_error};

pub fn router() -> Router<AppState> {
    Router::new()
        .route("/commands/object-field", post(set_object_field))
        .route("/commands/script/block", post(set_script_block))
        .route("/commands/script/lock", post(set_script_lock))
        .route("/commands/story/create-arc", post(create_story_arc))
        .route("/commands/story/update-arc", post(update_story_arc))
        .route("/commands/story/delete-arc", post(delete_story_arc))
        .merge(super::commands_bible::router())
        .merge(super::commands_timeline::router())
}

#[derive(Debug, Serialize)]
struct ScriptDocumentCommandResponse {
    outcome: RecordChangeOutcome,
    projection: ProjectionEnvelope<ScriptDocumentProjection>,
}

#[derive(Debug, Serialize)]
struct StoryArcCommandResponse {
    outcome: RecordChangeOutcome,
    projection: ProjectionEnvelope<StoryArcListProjection>,
}

#[derive(Debug, Deserialize)]
#[serde(deny_unknown_fields)]
struct CreateStoryArcRouteCommand {
    id: CommandId,
    payload: CreateStoryArcRoutePayload,
}

#[derive(Debug, Deserialize)]
#[serde(deny_unknown_fields)]
struct CreateStoryArcRoutePayload {
    #[serde(default)]
    arc_id: Option<ArcId>,
    #[serde(default)]
    parent_arc_id: Option<ArcId>,
    name: String,
    #[serde(default)]
    description: String,
    arc_type: eidetic_core::story::arc::ArcType,
    color: eidetic_core::story::arc::Color,
}

impl CreateStoryArcRouteCommand {
    fn into_core_command(self) -> CommandEnvelope<CreateStoryArcCommand> {
        CommandEnvelope {
            id: self.id,
            payload: CreateStoryArcCommand {
                arc_id: self
                    .payload
                    .arc_id
                    .unwrap_or_else(|| ArcId(derived_command_uuid(self.id, b"story.arc"))),
                parent_arc_id: self.payload.parent_arc_id,
                name: self.payload.name,
                description: self.payload.description,
                arc_type: self.payload.arc_type,
                color: self.payload.color,
            },
        }
    }
}

fn derived_command_uuid(command_id: CommandId, role: &[u8]) -> Uuid {
    let mut bytes = *command_id.0.as_bytes();
    for (index, byte) in role.iter().enumerate() {
        let slot = index % bytes.len();
        bytes[slot] = bytes[slot].wrapping_add(*byte).rotate_left(1);
    }
    bytes[6] = (bytes[6] & 0x0f) | 0x40;
    bytes[8] = (bytes[8] & 0x3f) | 0x80;
    Uuid::from_bytes(bytes)
}

async fn set_object_field(
    State(state): State<AppState>,
    Json(command): Json<CommandEnvelope<SetObjectFieldCommand>>,
) -> ApiJson {
    let response = crate::command_service::set_object_field(&state, command)
        .await
        .map_err(ApiError::from)?;
    crate::error::json_value(response)
}

async fn set_script_block(
    State(state): State<AppState>,
    Json(command): Json<CommandEnvelope<SetScriptBlockCommand>>,
) -> ApiJson {
    let path = active_project_path(&state)?;
    let response = tokio::task::spawn_blocking(move || set_script_block_at_path(path, command))
        .await
        .map_err(|e| ApiError::internal(format!("script block command task failed: {e}")))??;

    let _ = state.events_tx.send(ServerEvent::ScriptChanged);
    crate::error::json_value(response)
}

async fn set_script_lock(
    State(state): State<AppState>,
    Json(command): Json<CommandEnvelope<SetScriptLockCommand>>,
) -> ApiJson {
    let path = active_project_path(&state)?;
    let response = tokio::task::spawn_blocking(move || set_script_lock_at_path(path, command))
        .await
        .map_err(|e| ApiError::internal(format!("script lock command task failed: {e}")))??;

    let _ = state.events_tx.send(ServerEvent::ScriptChanged);
    crate::error::json_value(response)
}

async fn create_story_arc(
    State(state): State<AppState>,
    Json(command): Json<CreateStoryArcRouteCommand>,
) -> ApiJson {
    let command = command.into_core_command();
    let path = active_project_path(&state)?;
    if state.project.lock().is_none() {
        return Err(ApiError::no_project());
    }
    let response = {
        let mut conn = crate::sqlite::open_write_connection(&path)
            .map_err(|e| ApiError::internal(e.to_string()))?;
        story_arc_store::create_schema(&conn).map_err(map_history_error)?;
        let outcome = story_arc_command::record_create_story_arc_history(&mut conn, &command, 0)
            .map_err(map_story_arc_command_error)?;
        let projection =
            story_arc_store::load_arc_list_projection_envelope(&conn).map_err(map_history_error)?;
        StoryArcCommandResponse {
            outcome,
            projection,
        }
    };

    if response.outcome == RecordChangeOutcome::Recorded {
        let _ = state.events_tx.send(ServerEvent::StoryChanged);
    }
    crate::error::json_value(response)
}

async fn update_story_arc(
    State(state): State<AppState>,
    Json(command): Json<CommandEnvelope<SetStoryArcMetadataCommand>>,
) -> ApiJson {
    let path = active_project_path(&state)?;
    if state.project.lock().is_none() {
        return Err(ApiError::no_project());
    }
    let response = {
        let mut conn = crate::sqlite::open_write_connection(&path)
            .map_err(|e| ApiError::internal(e.to_string()))?;
        story_arc_store::create_schema(&conn).map_err(map_history_error)?;
        let outcome =
            story_arc_command::record_set_story_arc_metadata_history(&mut conn, &command, 0)
                .map_err(map_story_arc_command_error)?;
        let projection =
            story_arc_store::load_arc_list_projection_envelope(&conn).map_err(map_history_error)?;
        StoryArcCommandResponse {
            outcome,
            projection,
        }
    };

    if response.outcome == RecordChangeOutcome::Recorded {
        let _ = state.events_tx.send(ServerEvent::StoryChanged);
    }
    crate::error::json_value(response)
}

async fn delete_story_arc(
    State(state): State<AppState>,
    Json(command): Json<CommandEnvelope<DeleteStoryArcCommand>>,
) -> ApiJson {
    let path = active_project_path(&state)?;
    if state.project.lock().is_none() {
        return Err(ApiError::no_project());
    }
    let response = {
        let mut conn = crate::sqlite::open_write_connection(&path)
            .map_err(|e| ApiError::internal(e.to_string()))?;
        story_arc_store::create_schema(&conn).map_err(map_history_error)?;
        let outcome = story_arc_command::record_delete_story_arc_history(&mut conn, &command, 0)
            .map_err(map_story_arc_command_error)?;
        let projection =
            story_arc_store::load_arc_list_projection_envelope(&conn).map_err(map_history_error)?;
        StoryArcCommandResponse {
            outcome,
            projection,
        }
    };

    if response.outcome == RecordChangeOutcome::Recorded {
        let _ = state.events_tx.send(ServerEvent::StoryChanged);
    }
    crate::error::json_value(response)
}

fn set_script_block_at_path(
    path: std::path::PathBuf,
    command: CommandEnvelope<SetScriptBlockCommand>,
) -> Result<ScriptDocumentCommandResponse, ApiError> {
    let mut conn = crate::sqlite::open_write_connection(&path)
        .map_err(|e| ApiError::internal(e.to_string()))?;
    let (outcome, projection) =
        script_document_command::apply_set_script_block(&mut conn, &command, 0)
            .map_err(map_script_document_error)?;

    Ok(ScriptDocumentCommandResponse {
        outcome,
        projection,
    })
}

fn set_script_lock_at_path(
    path: std::path::PathBuf,
    command: CommandEnvelope<SetScriptLockCommand>,
) -> Result<ScriptDocumentCommandResponse, ApiError> {
    let mut conn = crate::sqlite::open_write_connection(&path)
        .map_err(|e| ApiError::internal(e.to_string()))?;
    let (outcome, projection) =
        script_document_command::apply_set_script_lock(&mut conn, &command, 0)
            .map_err(map_script_document_error)?;

    Ok(ScriptDocumentCommandResponse {
        outcome,
        projection,
    })
}

fn map_script_document_error(error: ScriptDocumentCommandError) -> ApiError {
    match error {
        ScriptDocumentCommandError::InvalidCommand(message) => ApiError::bad_request(message),
        ScriptDocumentCommandError::Store(error) => map_history_error(error),
    }
}

fn map_story_arc_command_error(error: StoryArcCommandError) -> ApiError {
    match error {
        StoryArcCommandError::InvalidCommand(message) => ApiError::bad_request(message),
        StoryArcCommandError::NotFound(message) => ApiError::not_found(message),
        StoryArcCommandError::History(error) => map_history_error(error),
    }
}

#[cfg(test)]
#[path = "commands_object_story_tests.rs"]
mod object_story_tests;

#[cfg(test)]
#[path = "commands_script_tests.rs"]
mod script_tests;
