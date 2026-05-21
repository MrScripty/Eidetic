use eidetic_core::reference::ReferenceDocument;
use eidetic_server::reference_service::{self, DeleteReferenceResponse, UploadReferenceRequest};
use eidetic_server::state::AppState;
use tauri::Manager;
use uuid::Uuid;

use crate::error::CommandError;

#[tauri::command]
pub fn reference_list(app: tauri::AppHandle) -> Result<Vec<ReferenceDocument>, CommandError> {
    let state = app.state::<AppState>();
    reference_service::list_references(&state).map_err(CommandError::from)
}

#[tauri::command]
pub fn reference_upload(
    app: tauri::AppHandle,
    request: UploadReferenceRequest,
) -> Result<ReferenceDocument, CommandError> {
    let state = app.state::<AppState>();
    reference_service::upload_reference(&state, request).map_err(CommandError::from)
}

#[tauri::command]
pub fn reference_delete(
    app: tauri::AppHandle,
    id: Uuid,
) -> Result<DeleteReferenceResponse, CommandError> {
    let state = app.state::<AppState>();
    reference_service::delete_reference(&state, id).map_err(CommandError::from)
}
