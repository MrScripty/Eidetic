use std::path::PathBuf;

use eidetic_core::contracts::CommandId;
use uuid::Uuid;

use crate::backend_error::BackendError;
use crate::history_store::HistoryStoreError;
use crate::state::AppState;

pub(crate) fn active_project_path(state: &AppState) -> Result<PathBuf, BackendError> {
    if state.project.lock().is_none() {
        return Err(BackendError::no_project());
    }
    state
        .project_database
        .active_path()
        .ok_or_else(BackendError::no_project)
}

pub(crate) fn derived_command_uuid(command_id: CommandId, role: &[u8]) -> Uuid {
    let mut bytes = *command_id.0.as_bytes();
    for (index, byte) in role.iter().enumerate() {
        let slot = index % bytes.len();
        bytes[slot] = bytes[slot].wrapping_add(*byte).rotate_left(1);
    }
    bytes[6] = (bytes[6] & 0x0f) | 0x40;
    bytes[8] = (bytes[8] & 0x3f) | 0x80;
    Uuid::from_bytes(bytes)
}

pub(crate) fn map_history_error(error: HistoryStoreError) -> BackendError {
    match error {
        HistoryStoreError::InvalidValue(message) => BackendError::conflict(message),
        HistoryStoreError::InvalidId(message) => BackendError::bad_request(message),
        HistoryStoreError::MissingColumn(message) => BackendError::internal(message),
        HistoryStoreError::Sqlite(error) => BackendError::internal(error.to_string()),
        HistoryStoreError::Json(error) => BackendError::bad_request(error.to_string()),
    }
}
