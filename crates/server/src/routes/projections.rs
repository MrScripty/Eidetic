use axum::Router;
use axum::extract::{Query, State};
use axum::routing::get;
use eidetic_core::contracts::ObjectKind;
use eidetic_core::contracts::ProjectionEnvelope;
use serde::Deserialize;

use crate::error::{ApiError, ApiJson};
use crate::history_store;
use crate::revision_projection::ObjectFieldProjection;
use crate::state::AppState;

use super::support::{active_project_path, map_history_error};

pub fn router() -> Router<AppState> {
    Router::new().route(
        "/projections/object-field",
        get(get_object_field_projection),
    )
}

#[derive(Debug, Deserialize)]
struct ObjectFieldProjectionQuery {
    object_kind: ObjectKind,
    object_id: String,
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

#[cfg(test)]
#[path = "projections_tests.rs"]
mod tests;
