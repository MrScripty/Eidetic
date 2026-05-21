use eidetic_core::contracts::{
    ProjectionEnvelope, SelectedNodeEditorProjection, TimelineRenderProjection,
};
use eidetic_server::projection_service::{self, SelectedNodeEditorProjectionRequest};
use eidetic_server::state::AppState;
use tauri::Manager;

use crate::error::CommandError;

#[tauri::command]
pub async fn projection_timeline_render(
    app: tauri::AppHandle,
) -> Result<ProjectionEnvelope<TimelineRenderProjection>, CommandError> {
    let state = app.state::<AppState>().inner().clone();
    projection_service::timeline_render_projection(&state)
        .await
        .map_err(CommandError::from)
}

#[tauri::command]
pub async fn projection_selected_node(
    app: tauri::AppHandle,
    query: SelectedNodeEditorProjectionRequest,
) -> Result<ProjectionEnvelope<SelectedNodeEditorProjection>, CommandError> {
    let state = app.state::<AppState>().inner().clone();
    projection_service::selected_node_editor_projection(&state, query)
        .await
        .map_err(CommandError::from)
}
