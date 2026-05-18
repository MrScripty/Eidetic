use axum::Router;
use axum::extract::{Query, State};
use axum::routing::get;
use eidetic_core::contracts::ProjectionEnvelope;
use eidetic_core::contracts::{BibleGraphNodeId, BibleNodeDetailProjection, ObjectKind};
use serde::Deserialize;

use crate::bible_graph_store;
use crate::error::{ApiError, ApiJson};
use crate::history_store;
use crate::revision_projection::ObjectFieldProjection;
use crate::state::AppState;

use super::support::{active_project_path, map_history_error};

pub fn router() -> Router<AppState> {
    Router::new()
        .route(
            "/projections/object-field",
            get(get_object_field_projection),
        )
        .route(
            "/projections/bible-graph/node",
            get(get_bible_graph_node_projection),
        )
}

#[derive(Debug, Deserialize)]
struct ObjectFieldProjectionQuery {
    object_kind: ObjectKind,
    object_id: String,
}

#[derive(Debug, Deserialize)]
struct BibleGraphNodeProjectionQuery {
    node_id: BibleGraphNodeId,
}

async fn get_object_field_projection(
    State(state): State<AppState>,
    Query(query): Query<ObjectFieldProjectionQuery>,
) -> ApiJson {
    if query.object_id.trim().is_empty() {
        return Err(ApiError::bad_request("object_id is required"));
    }

    let path = active_project_path(&state)?;
    let projection = tokio::task::spawn_blocking(move || {
        load_projection_at_path(path, query.object_kind, query.object_id)
    })
    .await
    .map_err(|e| ApiError::internal(format!("object field projection task failed: {e}")))??;

    crate::error::json_value(projection)
}

async fn get_bible_graph_node_projection(
    State(state): State<AppState>,
    Query(query): Query<BibleGraphNodeProjectionQuery>,
) -> ApiJson {
    let path = active_project_path(&state)?;
    let projection = tokio::task::spawn_blocking(move || {
        load_bible_node_projection_at_path(path, query.node_id)
    })
    .await
    .map_err(|e| ApiError::internal(format!("bible graph projection task failed: {e}")))??;

    crate::error::json_value(projection)
}

fn load_projection_at_path(
    path: std::path::PathBuf,
    object_kind: ObjectKind,
    object_id: String,
) -> Result<ProjectionEnvelope<ObjectFieldProjection>, ApiError> {
    let conn = crate::sqlite::open_write_connection(&path)
        .map_err(|e| ApiError::internal(e.to_string()))?;
    history_store::create_schema(&conn).map_err(map_history_error)?;
    crate::revision_projection::load_object_field_projection_envelope(
        &conn,
        object_kind,
        &object_id,
    )
    .map_err(map_history_error)
}

fn load_bible_node_projection_at_path(
    path: std::path::PathBuf,
    node_id: BibleGraphNodeId,
) -> Result<ProjectionEnvelope<BibleNodeDetailProjection>, ApiError> {
    let conn = crate::sqlite::open_write_connection(&path)
        .map_err(|e| ApiError::internal(e.to_string()))?;
    bible_graph_store::create_schema(&conn).map_err(map_history_error)?;
    bible_graph_store::load_node_detail_projection_envelope(&conn, &node_id)
        .map_err(map_history_error)?
        .ok_or_else(|| ApiError::not_found("bible graph node not found"))
}

#[cfg(test)]
#[path = "projections_tests.rs"]
mod tests;
