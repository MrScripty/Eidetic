use eidetic_server::ai_service::{self, AiConfigUpdate, AiStatus};
use eidetic_server::state::{AiConfig, AppState};
use tauri::Manager;

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
