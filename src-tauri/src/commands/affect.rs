use eidetic_core::contracts::{
    AcceptAffectProposalCommand, AffectProjection, AffectProposalListProjection, CommandEnvelope,
    CreateAffectProposalCommand, ProjectionEnvelope, RejectAffectProposalCommand,
    SetAffectValueCommand,
};
use eidetic_server::affect_service;
use eidetic_server::state::AppState;
use tauri::Manager;

use crate::error::CommandError;

#[tauri::command]
pub async fn command_affect_set(
    app: tauri::AppHandle,
    command: CommandEnvelope<SetAffectValueCommand>,
) -> Result<ProjectionEnvelope<AffectProjection>, CommandError> {
    let state = app.state::<AppState>().inner().clone();
    affect_service::set_affect_value(&state, command)
        .await
        .map_err(CommandError::from)
}

#[tauri::command]
pub async fn command_affect_proposal_create(
    app: tauri::AppHandle,
    command: CommandEnvelope<CreateAffectProposalCommand>,
) -> Result<ProjectionEnvelope<AffectProposalListProjection>, CommandError> {
    let state = app.state::<AppState>().inner().clone();
    affect_service::create_affect_proposal(&state, command)
        .await
        .map_err(CommandError::from)
}

#[tauri::command]
pub async fn command_affect_proposal_reject(
    app: tauri::AppHandle,
    command: CommandEnvelope<RejectAffectProposalCommand>,
) -> Result<ProjectionEnvelope<AffectProposalListProjection>, CommandError> {
    let state = app.state::<AppState>().inner().clone();
    affect_service::reject_affect_proposal(&state, command)
        .await
        .map_err(CommandError::from)
}

#[tauri::command]
pub async fn command_affect_proposal_accept(
    app: tauri::AppHandle,
    command: CommandEnvelope<AcceptAffectProposalCommand>,
) -> Result<ProjectionEnvelope<AffectProposalListProjection>, CommandError> {
    let state = app.state::<AppState>().inner().clone();
    affect_service::accept_affect_proposal(&state, command)
        .await
        .map_err(CommandError::from)
}
