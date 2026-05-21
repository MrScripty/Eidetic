use eidetic_core::contracts::{
    AcceptBibleReferenceProposalCommand, AcceptPropagationProposalCommand, CommandEnvelope,
    CreateBibleReferenceProposalCommand, CreatePropagationProposalCommand,
    RejectBibleReferenceProposalCommand, RejectPropagationProposalCommand,
    UpdatePropagationProposalCommand,
};
use eidetic_server::command_service;
use eidetic_server::state::AppState;
use tauri::Manager;

use crate::error::CommandError;

#[tauri::command]
pub async fn command_bible_reference_proposal_create(
    app: tauri::AppHandle,
    command: CommandEnvelope<CreateBibleReferenceProposalCommand>,
) -> Result<command_service::BibleReferenceProposalCommandResponse, CommandError> {
    let state = app.state::<AppState>().inner().clone();
    command_service::create_bible_reference_proposal(&state, command)
        .await
        .map_err(CommandError::from)
}

#[tauri::command]
pub async fn command_bible_reference_proposal_reject(
    app: tauri::AppHandle,
    command: CommandEnvelope<RejectBibleReferenceProposalCommand>,
) -> Result<command_service::BibleReferenceProposalCommandResponse, CommandError> {
    let state = app.state::<AppState>().inner().clone();
    command_service::reject_bible_reference_proposal(&state, command)
        .await
        .map_err(CommandError::from)
}

#[tauri::command]
pub async fn command_bible_reference_proposal_accept(
    app: tauri::AppHandle,
    command: CommandEnvelope<AcceptBibleReferenceProposalCommand>,
) -> Result<command_service::BibleReferenceProposalCommandResponse, CommandError> {
    let state = app.state::<AppState>().inner().clone();
    command_service::accept_bible_reference_proposal(&state, command)
        .await
        .map_err(CommandError::from)
}

#[tauri::command]
pub async fn command_propagation_proposal_create(
    app: tauri::AppHandle,
    command: CommandEnvelope<CreatePropagationProposalCommand>,
) -> Result<command_service::PropagationProposalCommandResponse, CommandError> {
    let state = app.state::<AppState>().inner().clone();
    command_service::create_propagation_proposal(&state, command)
        .await
        .map_err(CommandError::from)
}

#[tauri::command]
pub async fn command_propagation_proposal_reject(
    app: tauri::AppHandle,
    command: CommandEnvelope<RejectPropagationProposalCommand>,
) -> Result<command_service::PropagationProposalCommandResponse, CommandError> {
    let state = app.state::<AppState>().inner().clone();
    command_service::reject_propagation_proposal(&state, command)
        .await
        .map_err(CommandError::from)
}

#[tauri::command]
pub async fn command_propagation_proposal_update(
    app: tauri::AppHandle,
    command: CommandEnvelope<UpdatePropagationProposalCommand>,
) -> Result<command_service::PropagationProposalCommandResponse, CommandError> {
    let state = app.state::<AppState>().inner().clone();
    command_service::update_propagation_proposal(&state, command)
        .await
        .map_err(CommandError::from)
}

#[tauri::command]
pub async fn command_propagation_proposal_accept(
    app: tauri::AppHandle,
    command: CommandEnvelope<AcceptPropagationProposalCommand>,
) -> Result<command_service::PropagationProposalCommandResponse, CommandError> {
    let state = app.state::<AppState>().inner().clone();
    command_service::accept_propagation_proposal(&state, command)
        .await
        .map_err(CommandError::from)
}
