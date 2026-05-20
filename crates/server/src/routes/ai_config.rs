use axum::Json;
use axum::extract::State;
use serde::Deserialize;

use crate::ai_backends::Backend;
use crate::state::{AppState, BackendType};

pub(super) async fn status(State(state): State<AppState>) -> Json<serde_json::Value> {
    let config = state.ai_config.lock().clone();
    let backend = Backend::from_config(&config);

    match backend.health_check().await {
        Ok(s) => {
            let display_model =
                if config.model.eq_ignore_ascii_case("auto") || config.model.is_empty() {
                    if s.model.is_empty() {
                        "auto".to_string()
                    } else {
                        s.model.clone()
                    }
                } else {
                    config.model.clone()
                };
            Json(serde_json::json!({
                "backend": config.backend_type,
                "model": display_model,
                "connected": s.connected,
                "message": s.message,
            }))
        }
        Err(e) => Json(serde_json::json!({
            "backend": config.backend_type,
            "model": config.model,
            "connected": false,
            "error": e.to_string(),
        })),
    }
}

#[derive(Deserialize)]
pub(super) struct ConfigUpdate {
    backend_type: Option<BackendType>,
    model: Option<String>,
    temperature: Option<f32>,
    max_tokens: Option<usize>,
    base_url: Option<String>,
    api_key: Option<Option<String>>,
}

pub(super) async fn config(
    State(state): State<AppState>,
    Json(body): Json<ConfigUpdate>,
) -> Json<serde_json::Value> {
    let mut config = state.ai_config.lock();
    if let Some(bt) = body.backend_type {
        config.backend_type = bt;
    }
    if let Some(m) = body.model {
        config.model = m;
    }
    if let Some(t) = body.temperature {
        config.temperature = t;
    }
    if let Some(mt) = body.max_tokens {
        config.max_tokens = mt;
    }
    if let Some(url) = body.base_url {
        config.base_url = url;
    }
    if let Some(key) = body.api_key {
        config.api_key = key.filter(|value| !value.is_empty());
    }
    Json(serde_json::to_value(&*config).expect("AI config serializes to JSON"))
}
