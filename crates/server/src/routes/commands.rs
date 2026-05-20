use axum::extract::State;
use axum::routing::post;
use axum::{Json, Router};
use eidetic_core::contracts::{
    CommandEnvelope, DeleteStoryArcCommand, SetObjectFieldCommand, SetScriptBlockCommand,
    SetScriptLockCommand, SetStoryArcMetadataCommand,
};

use crate::error::{ApiError, ApiJson};
use crate::state::AppState;

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
    let response = crate::command_service::set_script_block(&state, command)
        .await
        .map_err(ApiError::from)?;
    crate::error::json_value(response)
}

async fn set_script_lock(
    State(state): State<AppState>,
    Json(command): Json<CommandEnvelope<SetScriptLockCommand>>,
) -> ApiJson {
    let response = crate::command_service::set_script_lock(&state, command)
        .await
        .map_err(ApiError::from)?;
    crate::error::json_value(response)
}

async fn create_story_arc(
    State(state): State<AppState>,
    Json(command): Json<crate::command_service::CreateStoryArcRequestCommand>,
) -> ApiJson {
    let response = crate::command_service::create_story_arc(&state, command)
        .await
        .map_err(ApiError::from)?;
    crate::error::json_value(response)
}

async fn update_story_arc(
    State(state): State<AppState>,
    Json(command): Json<CommandEnvelope<SetStoryArcMetadataCommand>>,
) -> ApiJson {
    let response = crate::command_service::update_story_arc(&state, command)
        .await
        .map_err(ApiError::from)?;
    crate::error::json_value(response)
}

async fn delete_story_arc(
    State(state): State<AppState>,
    Json(command): Json<CommandEnvelope<DeleteStoryArcCommand>>,
) -> ApiJson {
    let response = crate::command_service::delete_story_arc(&state, command)
        .await
        .map_err(ApiError::from)?;
    crate::error::json_value(response)
}

#[cfg(test)]
#[path = "commands_object_story_tests.rs"]
mod object_story_tests;

#[cfg(test)]
#[path = "commands_script_tests.rs"]
mod script_tests;
