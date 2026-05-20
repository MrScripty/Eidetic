use axum::extract::State;
use axum::routing::post;
use axum::{Json, Router};
use eidetic_core::contracts::{
    CommandEnvelope, EnsureCanonicalBibleRootsCommand, SetBibleGraphFieldCommand,
};

use crate::error::{ApiError, ApiJson};
use crate::state::AppState;

pub fn router() -> Router<AppState> {
    Router::new()
        .route("/commands/bible-graph/node", post(create_bible_graph_node))
        .route("/commands/bible-graph/field", post(set_bible_graph_field))
        .route("/commands/bible-graph/edge", post(set_bible_graph_edge))
        .route(
            "/commands/bible-graph/snapshot-field",
            post(set_bible_graph_snapshot_field),
        )
        .route(
            "/commands/bible-graph/canonical-roots",
            post(ensure_canonical_bible_roots),
        )
}

async fn create_bible_graph_node(
    State(state): State<AppState>,
    Json(command): Json<crate::command_service::CreateBibleGraphNodeRequestCommand>,
) -> ApiJson {
    crate::command_service::create_bible_graph_node(&state, command)
        .await
        .map_err(ApiError::from)
        .and_then(crate::error::json_value)
}

async fn set_bible_graph_field(
    State(state): State<AppState>,
    Json(command): Json<CommandEnvelope<SetBibleGraphFieldCommand>>,
) -> ApiJson {
    crate::command_service::set_bible_graph_field(&state, command)
        .await
        .map_err(ApiError::from)
        .and_then(crate::error::json_value)
}

async fn set_bible_graph_edge(
    State(state): State<AppState>,
    Json(command): Json<crate::command_service::SetBibleGraphEdgeRequestCommand>,
) -> ApiJson {
    crate::command_service::set_bible_graph_edge(&state, command)
        .await
        .map_err(ApiError::from)
        .and_then(crate::error::json_value)
}

async fn set_bible_graph_snapshot_field(
    State(state): State<AppState>,
    Json(command): Json<crate::command_service::SetBibleGraphSnapshotFieldRequestCommand>,
) -> ApiJson {
    crate::command_service::set_bible_graph_snapshot_field(&state, command)
        .await
        .map_err(ApiError::from)
        .and_then(crate::error::json_value)
}

async fn ensure_canonical_bible_roots(
    State(state): State<AppState>,
    Json(command): Json<CommandEnvelope<EnsureCanonicalBibleRootsCommand>>,
) -> ApiJson {
    crate::command_service::ensure_canonical_bible_roots(&state, command)
        .await
        .map_err(ApiError::from)
        .and_then(crate::error::json_value)
}

#[cfg(test)]
#[path = "commands_bible_tests.rs"]
mod tests;

#[cfg(test)]
#[path = "commands_bible_snapshot_tests.rs"]
mod snapshot_tests;
