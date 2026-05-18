use std::path::PathBuf;

use crate::error::ApiError;
use crate::history_store::HistoryStoreError;
use crate::state::AppState;

pub(super) fn active_project_path(state: &AppState) -> Result<PathBuf, ApiError> {
    if state.project.lock().is_none() {
        return Err(ApiError::no_project());
    }
    state
        .project_path
        .lock()
        .clone()
        .ok_or_else(ApiError::no_project)
}

pub(super) fn map_history_error(error: HistoryStoreError) -> ApiError {
    match error {
        HistoryStoreError::InvalidValue(message) => ApiError::conflict(message),
        HistoryStoreError::InvalidId(message) => ApiError::bad_request(message),
        HistoryStoreError::MissingColumn(message) => ApiError::internal(message),
        HistoryStoreError::Sqlite(error) => ApiError::internal(error.to_string()),
        HistoryStoreError::Json(error) => ApiError::bad_request(error.to_string()),
    }
}
