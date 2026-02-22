//! API routes for browsing the Pumas model library.
//!
//! Provides endpoints for listing and searching local models so the UI
//! can offer a model picker instead of requiring manual path entry.

use axum::extract::{Query, State};
use axum::http::StatusCode;
use axum::routing::get;
use axum::{Json, Router};
use serde::{Deserialize, Serialize};

use crate::state::AppState;

pub fn router() -> Router<AppState> {
    Router::new().route("/models", get(list_models))
}

#[derive(Deserialize)]
struct ListParams {
    /// Full-text search query. Empty string returns all models.
    #[serde(default)]
    q: String,
    /// Filter by model type (e.g. "llm", "diffusion", "embedding").
    #[serde(default)]
    model_type: Option<String>,
    /// Maximum results to return (default 100).
    #[serde(default = "default_limit")]
    limit: usize,
    /// Pagination offset (default 0).
    #[serde(default)]
    offset: usize,
}

fn default_limit() -> usize {
    100
}

/// Compact model info returned to the UI.
#[derive(Serialize)]
struct ModelEntry {
    id: String,
    name: String,
    path: String,
    model_type: String,
    size_bytes: Option<u64>,
    tags: Vec<String>,
}

#[derive(Serialize)]
struct ListResponse {
    models: Vec<ModelEntry>,
    total_count: usize,
}

async fn list_models(
    State(state): State<AppState>,
    Query(params): Query<ListParams>,
) -> Result<Json<ListResponse>, (StatusCode, Json<serde_json::Value>)> {
    let library = state.model_library.as_ref().ok_or_else(|| {
        (
            StatusCode::SERVICE_UNAVAILABLE,
            Json(serde_json::json!({
                "error": "Model library not configured. Set PUMAS_MODELS_DIR env var."
            })),
        )
    })?;

    let result = library
        .search_models_filtered(
            &params.q,
            params.limit,
            params.offset,
            params.model_type.as_deref(),
            None,
        )
        .await
        .map_err(|e| {
            (
                StatusCode::INTERNAL_SERVER_ERROR,
                Json(serde_json::json!({ "error": e.to_string() })),
            )
        })?;

    let models = result
        .models
        .into_iter()
        .map(|r| {
            let size_bytes = r
                .metadata
                .get("size_bytes")
                .or_else(|| r.metadata.get("sizeBytes"))
                .and_then(|v| v.as_u64());

            ModelEntry {
                id: r.id,
                name: r.official_name,
                path: r.path,
                model_type: r.model_type,
                size_bytes,
                tags: r.tags,
            }
        })
        .collect();

    Ok(Json(ListResponse {
        models,
        total_count: result.total_count,
    }))
}
