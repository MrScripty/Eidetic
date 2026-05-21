use axum::Json;
use axum::extract::State;

use crate::ai_service::{self, AiGenerateChildrenRequest};
use crate::error::{ApiError, json_value};
use crate::state::AppState;

/// AI-powered decomposition: analyzes a node's notes and returns
/// a structured child plan that the user can edit before applying.
pub(super) async fn generate_children(
    State(state): State<AppState>,
    Json(body): Json<AiGenerateChildrenRequest>,
) -> Json<serde_json::Value> {
    match ai_service::generate_children(&state, body).await {
        Ok(plan) => json_value(plan).expect("child plans serialize to JSON"),
        Err(error) => Json(serde_json::json!({ "error": ApiError::from(error).1 })),
    }
}
