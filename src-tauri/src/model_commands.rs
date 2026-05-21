use eidetic_server::model_service::{self, ModelListRequest, ModelListResponse};
use eidetic_server::state::AppState;
use tauri::Manager;

use crate::error::CommandError;

#[tauri::command]
pub async fn model_list(
    app: tauri::AppHandle,
    params: ModelListRequest,
) -> Result<ModelListResponse, CommandError> {
    let state = app.state::<AppState>().inner().clone();
    model_service::list_models(&state, params)
        .await
        .map_err(CommandError::from)
}
