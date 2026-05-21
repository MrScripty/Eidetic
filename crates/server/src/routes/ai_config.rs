use axum::Json;
use axum::extract::State;

use crate::ai_service::{self, AiConfigUpdate};
use crate::state::AppState;

pub(super) async fn status(State(state): State<AppState>) -> Json<serde_json::Value> {
    Json(
        serde_json::to_value(ai_service::get_ai_status(&state).await)
            .expect("AI status serializes"),
    )
}

pub(super) async fn config(
    State(state): State<AppState>,
    Json(body): Json<AiConfigUpdate>,
) -> Json<serde_json::Value> {
    Json(
        serde_json::to_value(ai_service::update_ai_config(&state, body))
            .expect("AI config serializes"),
    )
}
