use eidetic_core::contracts::{
    CommandEnvelope, DeleteStoryArcCommand, SetObjectFieldCommand, SetScriptBlockCommand,
    SetScriptLockCommand, SetStoryArcMetadataCommand,
};
use eidetic_server::command_service::{self, CreateStoryArcRequestCommand};
use eidetic_server::state::AppState;
use tauri::Manager;

use crate::error::CommandError;

#[tauri::command]
pub async fn command_object_field(
    app: tauri::AppHandle,
    command: CommandEnvelope<SetObjectFieldCommand>,
) -> Result<command_service::ObjectFieldCommandResponse, CommandError> {
    let state = app.state::<AppState>().inner().clone();
    command_service::set_object_field(&state, command)
        .await
        .map_err(CommandError::from)
}

#[tauri::command]
pub async fn command_script_block(
    app: tauri::AppHandle,
    command: CommandEnvelope<SetScriptBlockCommand>,
) -> Result<command_service::ScriptDocumentCommandResponse, CommandError> {
    let state = app.state::<AppState>().inner().clone();
    command_service::set_script_block(&state, command)
        .await
        .map_err(CommandError::from)
}

#[tauri::command]
pub async fn command_script_lock(
    app: tauri::AppHandle,
    command: CommandEnvelope<SetScriptLockCommand>,
) -> Result<command_service::ScriptDocumentCommandResponse, CommandError> {
    let state = app.state::<AppState>().inner().clone();
    command_service::set_script_lock(&state, command)
        .await
        .map_err(CommandError::from)
}

#[tauri::command]
pub async fn command_story_create(
    app: tauri::AppHandle,
    command: CreateStoryArcRequestCommand,
) -> Result<command_service::StoryArcCommandResponse, CommandError> {
    let state = app.state::<AppState>().inner().clone();
    command_service::create_story_arc(&state, command)
        .await
        .map_err(CommandError::from)
}

#[tauri::command]
pub async fn command_story_update(
    app: tauri::AppHandle,
    command: CommandEnvelope<SetStoryArcMetadataCommand>,
) -> Result<command_service::StoryArcCommandResponse, CommandError> {
    let state = app.state::<AppState>().inner().clone();
    command_service::update_story_arc(&state, command)
        .await
        .map_err(CommandError::from)
}

#[tauri::command]
pub async fn command_story_delete(
    app: tauri::AppHandle,
    command: CommandEnvelope<DeleteStoryArcCommand>,
) -> Result<command_service::StoryArcCommandResponse, CommandError> {
    let state = app.state::<AppState>().inner().clone();
    command_service::delete_story_arc(&state, command)
        .await
        .map_err(CommandError::from)
}
