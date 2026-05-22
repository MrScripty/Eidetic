use eidetic_core::contracts::{
    ContextInfluenceProjection, ContextInfluenceProjectionRequest, ProjectionEnvelope,
};
use eidetic_server::context_influence_service;
use eidetic_server::state::AppState;
use tauri::Manager;

use crate::error::CommandError;

#[tauri::command]
pub async fn projection_context_influence(
    app: tauri::AppHandle,
    query: ContextInfluenceProjectionRequest,
) -> Result<ProjectionEnvelope<ContextInfluenceProjection>, CommandError> {
    let state = app.state::<AppState>().inner().clone();
    context_influence_service::context_influence_projection(&state, query)
        .await
        .map_err(CommandError::from)
}
