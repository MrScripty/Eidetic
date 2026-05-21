use axum::extract::{Query, State};
use axum::http::StatusCode;
use axum::routing::get;
use axum::{Json, Router};

use crate::model_service::{self, MODEL_LIBRARY_UNCONFIGURED, ModelListRequest, ModelListResponse};
use crate::state::AppState;

pub fn router() -> Router<AppState> {
    Router::new().route("/models", get(list_models))
}

async fn list_models(
    State(state): State<AppState>,
    Query(params): Query<ModelListRequest>,
) -> Result<Json<ModelListResponse>, (StatusCode, Json<serde_json::Value>)> {
    model_service::list_models(&state, params)
        .await
        .map(Json)
        .map_err(|error| {
            let status = if error.message() == MODEL_LIBRARY_UNCONFIGURED {
                StatusCode::SERVICE_UNAVAILABLE
            } else {
                StatusCode::INTERNAL_SERVER_ERROR
            };
            (
                status,
                Json(serde_json::json!({ "error": error.message() })),
            )
        })
}
