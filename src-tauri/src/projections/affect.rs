use eidetic_core::contracts::{
    AffectProjection, AffectProposalListProjection, AffectTarget, ProjectionEnvelope,
};
use eidetic_server::affect_service;
use eidetic_server::state::AppState;
use serde::Deserialize;
use tauri::Manager;

use crate::error::CommandError;

#[derive(Debug, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct AffectProjectionRequest {
    pub target: AffectTarget,
}

#[tauri::command]
pub async fn projection_affect(
    app: tauri::AppHandle,
    query: AffectProjectionRequest,
) -> Result<ProjectionEnvelope<AffectProjection>, CommandError> {
    let state = app.state::<AppState>().inner().clone();
    affect_service::affect_projection(&state, query.target)
        .await
        .map_err(CommandError::from)
}

#[tauri::command]
pub async fn projection_affect_proposals(
    app: tauri::AppHandle,
) -> Result<ProjectionEnvelope<AffectProposalListProjection>, CommandError> {
    let state = app.state::<AppState>().inner().clone();
    affect_service::affect_proposal_projection(&state)
        .await
        .map_err(CommandError::from)
}
