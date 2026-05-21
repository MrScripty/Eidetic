use axum::extract::{Path, State};
use axum::routing::{delete, get, post};
use axum::{Json, Router};
use uuid::Uuid;

use crate::error::{ApiError, ApiJson, json_value};
use crate::reference_service::{self, UploadReferenceRequest};
use crate::state::AppState;

pub fn router() -> Router<AppState> {
    Router::new()
        .route("/references", get(list_references))
        .route("/references", post(upload_reference))
        .route("/references/{id}", delete(delete_reference))
}

async fn list_references(State(state): State<AppState>) -> ApiJson {
    reference_service::list_references(&state)
        .map_err(ApiError::from)
        .and_then(json_value)
}

async fn upload_reference(
    State(state): State<AppState>,
    Json(body): Json<UploadReferenceRequest>,
) -> ApiJson {
    reference_service::upload_reference(&state, body)
        .map_err(ApiError::from)
        .and_then(json_value)
}

async fn delete_reference(State(state): State<AppState>, Path(id): Path<Uuid>) -> ApiJson {
    reference_service::delete_reference(&state, id)
        .map_err(ApiError::from)
        .and_then(json_value)
}
