use eidetic_core::ai::backend::ChildPlan;
use eidetic_server::ai_generation_service::{
    self, AiGenerateBatchRequest, AiGenerateBatchResponse, AiGenerateRequest, AiGenerateResponse,
};
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
    story_time_ms: Option<u64>,
) -> Result<AiContextPreview, CommandError> {
    let state = app.state::<AppState>().inner().clone();
    ai_service::preview_ai_context_at_story_time(&state, node_id, story_time_ms)
        .await
        .map_err(CommandError::from)
}

#[tauri::command]
pub async fn ai_generate_content(
    app: tauri::AppHandle,
    request: AiGenerateRequest,
) -> Result<AiGenerateResponse, CommandError> {
    let state = app.state::<AppState>().inner().clone();
    ai_generation_service::start_generation(&state, request)
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

#[tauri::command]
pub async fn ai_generate_batch(
    app: tauri::AppHandle,
    request: AiGenerateBatchRequest,
) -> Result<AiGenerateBatchResponse, CommandError> {
    let state = app.state::<AppState>().inner().clone();
    ai_generation_service::start_generation_batch(&state, request)
        .await
        .map_err(CommandError::from)
}

#[tauri::command]
pub async fn pumas_load_model(
    endpoint: String,
    request: eidetic_server::pumas_inference::ServeModelRequest,
) -> Result<serde_json::Value, CommandError> {
    eidetic_server::pumas_inference::load_model(&endpoint, request)
        .await
        .map_err(|e| CommandError::from(eidetic_server::backend_error::BackendError::internal(e)))
}

#[tauri::command]
pub async fn pumas_unload_model(
    endpoint: String,
    request: eidetic_server::pumas_inference::UnserveModelRequest,
) -> Result<eidetic_server::pumas_inference::UnserveModelResponse, CommandError> {
    eidetic_server::pumas_inference::unload_model(&endpoint, request)
        .await
        .map_err(|e| CommandError::from(eidetic_server::backend_error::BackendError::internal(e)))
}

#[tauri::command]
pub async fn pumas_catalog(endpoint: String) -> Result<serde_json::Value, CommandError> {
    eidetic_server::pumas_inference::catalog(&endpoint)
        .await
        .map_err(|e| CommandError::from(eidetic_server::backend_error::BackendError::internal(e)))
}

#[tauri::command]
pub fn ai_config_get(app: tauri::AppHandle) -> AiConfig {
    app.state::<AppState>().ai_config.lock().clone()
}
