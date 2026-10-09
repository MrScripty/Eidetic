//! Host-neutral explicit inspection. This service never changes generation context.
use crate::{backend_error::BackendError, state::AppState};
use eidetic_core::contracts::{BibleRecallProjection, BibleRecallRequest, ProjectionEnvelope};

pub async fn recall(
    state: &AppState,
    request: BibleRecallRequest,
) -> Result<ProjectionEnvelope<BibleRecallProjection>, BackendError> {
    request.validate().map_err(BackendError::bad_request)?;
    let path = crate::projection_service::active_project_path(state)?;
    tokio::task::spawn_blocking(move || {
        let conn = crate::sqlite::open_write_connection(&path)
            .map_err(|error| BackendError::internal(error.to_string()))?;
        crate::bible_graph_store::create_schema(&conn)
            .map_err(|error| BackendError::internal(error.to_string()))?;
        crate::bible_recall_projection::load(&conn, &request)
            .map_err(crate::projection_service::map_history_error)
    })
    .await
    .map_err(|error| BackendError::internal(format!("Recall task failed: {error}")))?
}
