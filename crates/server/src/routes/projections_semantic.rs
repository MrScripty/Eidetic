use axum::Router;
use axum::extract::State;
use axum::routing::get;
use eidetic_core::contracts::{BibleReferenceProposalListProjection, ProjectionEnvelope};

use crate::error::{ApiError, ApiJson};
use crate::semantic_proposal_store::{self, SemanticProposalStoreError};
use crate::state::AppState;

use super::support::{active_project_path, map_history_error};

pub fn router() -> Router<AppState> {
    Router::new().route(
        "/projections/semantic/bible-reference-proposals",
        get(get_bible_reference_proposal_list),
    )
}

async fn get_bible_reference_proposal_list(State(state): State<AppState>) -> ApiJson {
    let path = active_project_path(&state)?;
    let projection =
        tokio::task::spawn_blocking(move || load_bible_reference_proposal_list_at_path(path))
            .await
            .map_err(|e| {
                ApiError::internal(format!("semantic proposal projection task failed: {e}"))
            })??;

    crate::error::json_value(projection)
}

fn load_bible_reference_proposal_list_at_path(
    path: std::path::PathBuf,
) -> Result<ProjectionEnvelope<BibleReferenceProposalListProjection>, ApiError> {
    let conn = crate::sqlite::open_write_connection(&path)
        .map_err(|e| ApiError::internal(e.to_string()))?;
    semantic_proposal_store::load_bible_reference_proposal_list_projection(&conn)
        .map_err(map_semantic_proposal_error)
}

fn map_semantic_proposal_error(error: SemanticProposalStoreError) -> ApiError {
    match error {
        SemanticProposalStoreError::InvalidCommand(message) => ApiError::bad_request(message),
        SemanticProposalStoreError::NotFound(message) => ApiError::not_found(message),
        SemanticProposalStoreError::History(error) => map_history_error(error),
        SemanticProposalStoreError::Sqlite(error) => ApiError::internal(error.to_string()),
    }
}

#[cfg(test)]
#[path = "projections_semantic_tests.rs"]
mod tests;
