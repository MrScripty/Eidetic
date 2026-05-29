use eidetic_core::contracts::{
    BibleGraphNodeId, CommandEnvelope, DeleteBibleGraphEdgeCommand, DeleteBibleGraphNodeCommand,
    EnsureCanonicalBibleRootsCommand, SetBibleGraphFieldCommand, SetBibleGraphNodeNameCommand,
    SetBibleGraphNodeTextCommand,
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
pub async fn command_bible_graph_connected_node(
    app: tauri::AppHandle,
    parent_id: BibleGraphNodeId,
) -> Result<command_service::BibleGraphNodeCommandResponse, CommandError> {
    let state = app.state::<AppState>().inner().clone();
    command_service::create_connected_bible_graph_node(&state, parent_id)
        .await
        .map_err(CommandError::from)
}

#[tauri::command]
pub async fn command_bible_graph_delete_node(
    app: tauri::AppHandle,
    command: CommandEnvelope<DeleteBibleGraphNodeCommand>,
) -> Result<command_service::BibleGraphNodeListCommandResponse, CommandError> {
    let state = app.state::<AppState>().inner().clone();
    command_service::delete_bible_graph_node(&state, command)
        .await
        .map_err(CommandError::from)
}

#[tauri::command]
pub async fn command_bible_graph_node_name(
    app: tauri::AppHandle,
    command: CommandEnvelope<SetBibleGraphNodeNameCommand>,
) -> Result<command_service::BibleGraphNodeCommandResponse, CommandError> {
    let state = app.state::<AppState>().inner().clone();
    command_service::set_bible_graph_node_name(&state, command)
        .await
        .map_err(CommandError::from)
}

#[tauri::command]
pub async fn command_bible_graph_node_text(
    app: tauri::AppHandle,
    command: CommandEnvelope<SetBibleGraphNodeTextCommand>,
) -> Result<command_service::BibleGraphNodeCommandResponse, CommandError> {
    let state = app.state::<AppState>().inner().clone();
    command_service::set_bible_graph_node_text(&state, command)
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
pub async fn command_bible_graph_delete_edge(
    app: tauri::AppHandle,
    command: CommandEnvelope<DeleteBibleGraphEdgeCommand>,
) -> Result<command_service::BibleGraphNodeCommandResponse, CommandError> {
    let state = app.state::<AppState>().inner().clone();
    command_service::delete_bible_graph_edge(&state, command)
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
