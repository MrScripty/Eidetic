use eidetic_core::contracts::{
    CommandEnvelope, EnsureCanonicalBibleRootsCommand, SetBibleGraphFieldCommand,
};
use eidetic_server::command_service;
use eidetic_server::state::AppState;
use tauri::Manager;

use crate::error::CommandError;

#[tauri::command]
pub async fn command_bible_graph_node(
    app: tauri::AppHandle,
    command: command_service::CreateBibleGraphNodeRequestCommand,
) -> Result<command_service::BibleGraphNodeCommandResponse, CommandError> {
    let state = app.state::<AppState>().inner().clone();
    command_service::create_bible_graph_node(&state, command)
        .await
        .map_err(CommandError::from)
}

#[tauri::command]
pub async fn command_bible_graph_field(
    app: tauri::AppHandle,
    command: CommandEnvelope<SetBibleGraphFieldCommand>,
) -> Result<command_service::BibleGraphNodeCommandResponse, CommandError> {
    let state = app.state::<AppState>().inner().clone();
    command_service::set_bible_graph_field(&state, command)
        .await
        .map_err(CommandError::from)
}

#[tauri::command]
pub async fn command_bible_graph_edge(
    app: tauri::AppHandle,
    command: command_service::SetBibleGraphEdgeRequestCommand,
) -> Result<command_service::BibleGraphNodeCommandResponse, CommandError> {
    let state = app.state::<AppState>().inner().clone();
    command_service::set_bible_graph_edge(&state, command)
        .await
        .map_err(CommandError::from)
}

#[tauri::command]
pub async fn command_bible_graph_snapshot_field(
    app: tauri::AppHandle,
    command: command_service::SetBibleGraphSnapshotFieldRequestCommand,
) -> Result<command_service::BibleGraphNodeCommandResponse, CommandError> {
    let state = app.state::<AppState>().inner().clone();
    command_service::set_bible_graph_snapshot_field(&state, command)
        .await
        .map_err(CommandError::from)
}

#[tauri::command]
pub async fn command_bible_graph_roots(
    app: tauri::AppHandle,
    command: CommandEnvelope<EnsureCanonicalBibleRootsCommand>,
) -> Result<command_service::BibleGraphRootsCommandResponse, CommandError> {
    let state = app.state::<AppState>().inner().clone();
    command_service::ensure_canonical_bible_roots(&state, command)
        .await
        .map_err(CommandError::from)
}
