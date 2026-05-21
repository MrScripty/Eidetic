use eidetic_server::export_service;
use eidetic_server::state::AppState;
use tauri::Manager;

use crate::error::CommandError;

#[tauri::command]
pub async fn export_pdf(app: tauri::AppHandle) -> Result<Vec<u8>, CommandError> {
    let state = app.state::<AppState>().inner().clone();
    export_service::export_pdf(&state)
        .await
        .map_err(CommandError::from)
}
