use eidetic_core::ai::backend::ChildPlan;
use eidetic_server::ai_service::{
    self, AiConfigUpdate, AiContextPreview, AiGenerateChildrenRequest, AiStatus,
};
use eidetic_server::state::{AiConfig, AppState};
use tauri::Manager;
use uuid::Uuid;

use crate::error::CommandError;

#[tauri::command]
pub async fn ai_status(app: tauri::AppHandle) -> AiStatus {
    let state = app.state::<AppState>().inner().clone();
    ai_service::get_ai_status(&state).await
}

#[tauri::command]
pub fn ai_config_update(app: tauri::AppHandle, updates: AiConfigUpdate) -> AiConfig {
    let state = app.state::<AppState>();
    ai_service::update_ai_config(&state, updates)
}

#[tauri::command]
pub async fn ai_context_preview(
    app: tauri::AppHandle,
    node_id: Uuid,
) -> Result<AiContextPreview, CommandError> {
    let state = app.state::<AppState>().inner().clone();
    ai_service::preview_ai_context(&state, node_id)
        .await
        .map_err(CommandError::from)
}

#[tauri::command]
pub async fn ai_generate_children(
    app: tauri::AppHandle,
    request: AiGenerateChildrenRequest,
) -> Result<ChildPlan, CommandError> {
    let state = app.state::<AppState>().inner().clone();
    ai_service::generate_children(&state, request)
        .await
        .map_err(CommandError::from)
}
