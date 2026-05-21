use eidetic_core::contracts::{
    CommandEnvelope, DeleteTimelineNodeCommand, DeleteTimelineRelationshipCommand,
    SetTimelineNodeLockCommand, SetTimelineNodeNotesCommand, SetTimelineNodeRangeCommand,
};
use eidetic_server::command_service;
use eidetic_server::state::AppState;
use tauri::Manager;

use crate::error::CommandError;

#[tauri::command]
pub async fn command_timeline_create_node(
    app: tauri::AppHandle,
    command: command_service::CreateTimelineNodeRequestCommand,
) -> Result<command_service::TimelineCommandResponse, CommandError> {
    let state = app.state::<AppState>().inner().clone();
    command_service::create_timeline_node(&state, command)
        .await
        .map_err(CommandError::from)
}

#[tauri::command]
pub async fn command_timeline_node_range(
    app: tauri::AppHandle,
    command: CommandEnvelope<SetTimelineNodeRangeCommand>,
) -> Result<command_service::TimelineCommandResponse, CommandError> {
    let state = app.state::<AppState>().inner().clone();
    command_service::set_timeline_node_range(&state, command)
        .await
        .map_err(CommandError::from)
}

#[tauri::command]
pub async fn command_timeline_node_lock(
    app: tauri::AppHandle,
    command: CommandEnvelope<SetTimelineNodeLockCommand>,
) -> Result<command_service::TimelineCommandResponse, CommandError> {
    let state = app.state::<AppState>().inner().clone();
    command_service::set_timeline_node_lock(&state, command)
        .await
        .map_err(CommandError::from)
}

#[tauri::command]
pub async fn command_timeline_node_notes(
    app: tauri::AppHandle,
    command: CommandEnvelope<SetTimelineNodeNotesCommand>,
) -> Result<command_service::TimelineCommandResponse, CommandError> {
    let state = app.state::<AppState>().inner().clone();
    command_service::set_timeline_node_notes(&state, command)
        .await
        .map_err(CommandError::from)
}

#[tauri::command]
pub async fn command_timeline_delete_node(
    app: tauri::AppHandle,
    command: CommandEnvelope<DeleteTimelineNodeCommand>,
) -> Result<command_service::TimelineCommandResponse, CommandError> {
    let state = app.state::<AppState>().inner().clone();
    command_service::delete_timeline_node(&state, command)
        .await
        .map_err(CommandError::from)
}

#[tauri::command]
pub async fn command_timeline_delete_relationship(
    app: tauri::AppHandle,
    command: CommandEnvelope<DeleteTimelineRelationshipCommand>,
) -> Result<command_service::TimelineCommandResponse, CommandError> {
    let state = app.state::<AppState>().inner().clone();
    command_service::delete_timeline_relationship(&state, command)
        .await
        .map_err(CommandError::from)
}

#[tauri::command]
pub async fn command_timeline_split_node(
    app: tauri::AppHandle,
    command: command_service::SplitTimelineNodeRequestCommand,
) -> Result<command_service::TimelineCommandResponse, CommandError> {
    let state = app.state::<AppState>().inner().clone();
    command_service::split_timeline_node(&state, command)
        .await
        .map_err(CommandError::from)
}
