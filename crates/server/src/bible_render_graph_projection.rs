use std::path::PathBuf;

use eidetic_core::contracts::{
    BibleRenderGraphProjection, BibleRenderGraphProjectionRequest, ProjectionEnvelope,
};

use crate::backend_error::BackendError;
use crate::bible_graph_store;
use crate::history_store::HistoryStoreError;
use crate::state::AppState;

pub async fn bible_render_graph_projection(
    state: &AppState,
    request: BibleRenderGraphProjectionRequest,
) -> Result<ProjectionEnvelope<BibleRenderGraphProjection>, BackendError> {
    let path = active_project_path(state)?;
    tokio::task::spawn_blocking(move || load_bible_render_graph_projection_at_path(path, request))
        .await
        .map_err(|error| {
            BackendError::internal(format!("bible render graph task failed: {error}"))
        })?
}

fn active_project_path(state: &AppState) -> Result<PathBuf, BackendError> {
    if state.project.lock().is_none() {
        return Err(BackendError::no_project());
    }
    state
        .project_database
        .active_path()
        .ok_or_else(BackendError::no_project)
}

fn load_bible_render_graph_projection_at_path(
    path: PathBuf,
    request: BibleRenderGraphProjectionRequest,
) -> Result<ProjectionEnvelope<BibleRenderGraphProjection>, BackendError> {
    let conn = crate::sqlite::open_write_connection(&path)
        .map_err(|e| BackendError::internal(e.to_string()))?;
    bible_graph_store::create_schema(&conn).map_err(map_history_error)?;
    bible_graph_store::load_render_graph_projection_envelope(&conn, &request)
        .map_err(map_history_error)
}

fn map_history_error(error: HistoryStoreError) -> BackendError {
    match error {
        HistoryStoreError::InvalidValue(message) => BackendError::conflict(message),
        HistoryStoreError::InvalidId(message) => BackendError::bad_request(message),
        HistoryStoreError::MissingColumn(message) => BackendError::internal(message),
        HistoryStoreError::Sqlite(error) => BackendError::internal(error.to_string()),
        HistoryStoreError::Json(error) => BackendError::bad_request(error.to_string()),
    }
}
