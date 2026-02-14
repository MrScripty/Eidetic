use axum::extract::State;
use axum::routing::{get, post, put};
use axum::{Json, Router};
use futures::StreamExt;
use serde::Deserialize;
use uuid::Uuid;

use crate::ai_backends::Backend;
use crate::prompt_format::build_chat_prompt;
use crate::state::{AppState, BackendType, ServerEvent};
use eidetic_core::ai::prompt::build_generate_request;
use eidetic_core::timeline::clip::{ClipId, ContentStatus};

pub fn router() -> Router<AppState> {
    Router::new()
        .route("/ai/generate", post(generate))
        .route("/ai/react", post(react))
        .route("/ai/summarize", post(summarize))
        .route("/ai/status", get(status))
        .route("/ai/config", put(config))
}

#[derive(Deserialize)]
struct GenerateBody {
    clip_id: Uuid,
}

async fn generate(
    State(state): State<AppState>,
    Json(body): Json<GenerateBody>,
) -> Json<serde_json::Value> {
    let clip_id = ClipId(body.clip_id);

    // Validate and build the request while holding the project lock briefly.
    let request = {
        let project_guard = state.project.lock();
        let Some(project) = project_guard.as_ref() else {
            return Json(serde_json::json!({ "error": "no project loaded" }));
        };

        // Check clip exists.
        let clip = match project.timeline.clip(clip_id) {
            Ok(c) => c,
            Err(_) => {
                return Json(
                    serde_json::json!({ "error": format!("clip not found: {}", body.clip_id) }),
                );
            }
        };

        // Check clip is not locked.
        if clip.locked {
            return Json(serde_json::json!({ "error": "clip is locked" }));
        }

        // Check clip has beat notes.
        if clip.content.beat_notes.trim().is_empty() {
            return Json(serde_json::json!({ "error": "clip has no beat notes" }));
        }

        // Check not already generating.
        if state.generating.lock().contains(&body.clip_id) {
            return Json(serde_json::json!({ "error": "generation already in progress" }));
        }

        match build_generate_request(project, clip_id) {
            Ok(req) => req,
            Err(e) => {
                return Json(serde_json::json!({ "error": e.to_string() }));
            }
        }
    };

    // Mark as generating.
    state.generating.lock().insert(body.clip_id);

    // Update clip status to Generating.
    {
        let mut project_guard = state.project.lock();
        if let Some(project) = project_guard.as_mut() {
            if let Ok(clip) = project.timeline.clip_mut(clip_id) {
                clip.content.status = ContentStatus::Generating;
            }
        }
    }
    let _ = state
        .events_tx
        .send(ServerEvent::BeatUpdated { clip_id: body.clip_id });

    // Spawn the generation task.
    let state_clone = state.clone();
    let clip_uuid = body.clip_id;
    tokio::spawn(async move {
        run_generation(state_clone, clip_uuid, request).await;
    });

    Json(serde_json::json!({
        "status": "started",
        "clip_id": body.clip_id.to_string(),
    }))
}

async fn run_generation(
    state: AppState,
    clip_id: Uuid,
    request: eidetic_core::ai::backend::GenerateRequest,
) {
    let config = state.ai_config.lock().clone();
    let backend = Backend::from_config(&config);
    let prompt = build_chat_prompt(&request);

    let stream_result = backend.generate(&prompt, &config).await;

    let mut stream = match stream_result {
        Ok(s) => s,
        Err(e) => {
            tracing::error!("AI generation failed for clip {clip_id}: {e}");
            // Revert clip status.
            {
                let mut project_guard = state.project.lock();
                if let Some(project) = project_guard.as_mut() {
                    if let Ok(clip) = project.timeline.clip_mut(ClipId(clip_id)) {
                        clip.content.status = ContentStatus::NotesOnly;
                    }
                }
            }
            let _ = state.events_tx.send(ServerEvent::GenerationError {
                clip_id,
                error: e.to_string(),
            });
            state.generating.lock().remove(&clip_id);
            return;
        }
    };

    let mut full_text = String::new();
    let mut tokens_generated: usize = 0;

    while let Some(item) = stream.next().await {
        match item {
            Ok(token) => {
                full_text.push_str(&token);
                tokens_generated += 1;
                let _ = state.events_tx.send(ServerEvent::GenerationProgress {
                    clip_id,
                    token,
                    tokens_generated,
                });
            }
            Err(e) => {
                tracing::warn!("Stream error during generation for clip {clip_id}: {e}");
                break;
            }
        }
    }

    // Store the result.
    if full_text.is_empty() {
        // No output produced — revert status.
        {
            let mut project_guard = state.project.lock();
            if let Some(project) = project_guard.as_mut() {
                if let Ok(clip) = project.timeline.clip_mut(ClipId(clip_id)) {
                    clip.content.status = ContentStatus::NotesOnly;
                }
            }
        }
        let _ = state.events_tx.send(ServerEvent::GenerationError {
            clip_id,
            error: "AI produced no output".into(),
        });
    } else {
        // Store generated script and update status.
        {
            let mut project_guard = state.project.lock();
            if let Some(project) = project_guard.as_mut() {
                if let Ok(clip) = project.timeline.clip_mut(ClipId(clip_id)) {
                    clip.content.generated_script = Some(full_text);
                    clip.content.status = ContentStatus::Generated;
                }
            }
        }
        let _ = state
            .events_tx
            .send(ServerEvent::GenerationComplete { clip_id });
        let _ = state
            .events_tx
            .send(ServerEvent::BeatUpdated { clip_id });
    }

    state.generating.lock().remove(&clip_id);
}

async fn status(State(state): State<AppState>) -> Json<serde_json::Value> {
    let config = state.ai_config.lock().clone();
    let backend = Backend::from_config(&config);

    match backend.health_check().await {
        Ok(status) => Json(serde_json::json!({
            "backend": config.backend_type,
            "model": config.model,
            "connected": status.connected,
            "message": status.message,
        })),
        Err(e) => Json(serde_json::json!({
            "backend": config.backend_type,
            "model": config.model,
            "connected": false,
            "error": e.to_string(),
        })),
    }
}

#[derive(Deserialize)]
struct ConfigUpdate {
    backend_type: Option<BackendType>,
    model: Option<String>,
    temperature: Option<f32>,
    max_tokens: Option<usize>,
    base_url: Option<String>,
    api_key: Option<String>,
}

async fn config(
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
        config.api_key = Some(key);
    }
    Json(serde_json::to_value(&*config).unwrap())
}

/// Stub — Sprint 4 will implement the edit reaction pipeline.
async fn react() -> Json<serde_json::Value> {
    Json(serde_json::json!({
        "error": "AI edit reaction not yet implemented (Sprint 4)"
    }))
}

/// Stub — generation handles this inline for now.
async fn summarize() -> Json<serde_json::Value> {
    Json(serde_json::json!({
        "error": "AI summarization not yet implemented"
    }))
}
