use axum::Json;
use axum::extract::State;

use crate::ai_generation_service::{self, AiGenerateBatchRequest, AiGenerateRequest};
use crate::error::json_value;
use crate::state::AppState;

pub(super) async fn generate(
    State(state): State<AppState>,
    Json(body): Json<AiGenerateRequest>,
) -> Json<serde_json::Value> {
    match ai_generation_service::start_generation(&state, body).await {
        Ok(response) => json_value(response).expect("generation responses serialize to JSON"),
        Err(error) => Json(serde_json::json!({ "error": error.message() })),
    }
}

pub(super) async fn generate_batch(
    State(state): State<AppState>,
    Json(body): Json<AiGenerateBatchRequest>,
) -> Json<serde_json::Value> {
    match ai_generation_service::start_generation_batch(&state, body).await {
        Ok(response) => json_value(response).expect("batch generation responses serialize to JSON"),
        Err(error) => Json(serde_json::json!({ "error": error.message() })),
    }
}
