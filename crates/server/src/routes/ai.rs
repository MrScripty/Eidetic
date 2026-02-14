use axum::routing::{get, post, put};
use axum::{Json, Router};

use crate::state::AppState;

pub fn router() -> Router<AppState> {
    Router::new()
        .route("/ai/generate", post(generate))
        .route("/ai/react", post(react))
        .route("/ai/summarize", post(summarize))
        .route("/ai/status", get(status))
        .route("/ai/config", put(config))
}

/// Stub — Sprint 3 will implement the generation pipeline.
async fn generate() -> Json<serde_json::Value> {
    Json(serde_json::json!({
        "error": "AI generation not yet implemented (Sprint 3)"
    }))
}

/// Stub — Sprint 4 will implement the edit reaction pipeline.
async fn react() -> Json<serde_json::Value> {
    Json(serde_json::json!({
        "error": "AI edit reaction not yet implemented (Sprint 4)"
    }))
}

/// Stub — Sprint 3.
async fn summarize() -> Json<serde_json::Value> {
    Json(serde_json::json!({
        "error": "AI summarization not yet implemented (Sprint 3)"
    }))
}

async fn status() -> Json<serde_json::Value> {
    Json(serde_json::json!({
        "backend": "none",
        "status": "not_configured",
        "message": "AI backends will be available in Sprint 3"
    }))
}

async fn config() -> Json<serde_json::Value> {
    Json(serde_json::json!({
        "error": "AI configuration not yet implemented (Sprint 3)"
    }))
}
