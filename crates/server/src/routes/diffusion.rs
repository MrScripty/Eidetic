//! API routes for the diffusion LLM engine.
//!
//! Provides endpoints for loading/unloading the model, checking status,
//! and running diffusion-based text infilling on story nodes.

use axum::extract::State;
use axum::http::StatusCode;
use axum::routing::{get, post};
use axum::{Json, Router};
use serde::Deserialize;
use uuid::Uuid;

use crate::diffusion::{DiffuseCmd, DiffusionError};
use crate::state::{AppState, ServerEvent};
use crate::ydoc::{self, ContentField, DocCommand};
use eidetic_core::timeline::node::NodeId;

pub fn router() -> Router<AppState> {
    Router::new()
        .route("/ai/diffuse", post(diffuse))
        .route("/ai/diffusion/load", post(load_model))
        .route("/ai/diffusion/unload", post(unload_model))
        .route("/ai/diffusion/status", get(status))
}

// ──────────────────────────────────────────────
// Load / Unload / Status
// ──────────────────────────────────────────────

#[derive(Deserialize)]
struct LoadBody {
    model_path: String,
    #[serde(default = "default_device")]
    device: String,
}

fn default_device() -> String {
    "cuda".into()
}

async fn load_model(
    State(state): State<AppState>,
    Json(body): Json<LoadBody>,
) -> Result<Json<serde_json::Value>, (StatusCode, Json<serde_json::Value>)> {
    let (reply_tx, reply_rx) = tokio::sync::oneshot::channel();

    state
        .diffuse_tx
        .send(DiffuseCmd::LoadModel {
            model_path: body.model_path.clone(),
            device: body.device.clone(),
            reply: reply_tx,
        })
        .await
        .map_err(|_| channel_closed_error())?;

    let result = reply_rx.await.map_err(|_| channel_closed_error())?;

    match result {
        Ok(()) => Ok(Json(serde_json::json!({
            "status": "loaded",
            "model_path": body.model_path,
            "device": body.device,
        }))),
        Err(e) => Err((
            StatusCode::INTERNAL_SERVER_ERROR,
            Json(serde_json::json!({ "error": e.to_string() })),
        )),
    }
}

async fn unload_model(
    State(state): State<AppState>,
) -> Result<Json<serde_json::Value>, (StatusCode, Json<serde_json::Value>)> {
    let (reply_tx, reply_rx) = tokio::sync::oneshot::channel();

    state
        .diffuse_tx
        .send(DiffuseCmd::UnloadModel { reply: reply_tx })
        .await
        .map_err(|_| channel_closed_error())?;

    let result = reply_rx.await.map_err(|_| channel_closed_error())?;

    match result {
        Ok(()) => Ok(Json(serde_json::json!({ "status": "unloaded" }))),
        Err(e) => Err((
            StatusCode::INTERNAL_SERVER_ERROR,
            Json(serde_json::json!({ "error": e.to_string() })),
        )),
    }
}

async fn status(
    State(state): State<AppState>,
) -> Result<Json<serde_json::Value>, (StatusCode, Json<serde_json::Value>)> {
    let (reply_tx, reply_rx) = tokio::sync::oneshot::channel();

    state
        .diffuse_tx
        .send(DiffuseCmd::Status { reply: reply_tx })
        .await
        .map_err(|_| channel_closed_error())?;

    let status = reply_rx.await.map_err(|_| channel_closed_error())?;

    Ok(Json(serde_json::to_value(&status).unwrap()))
}

// ──────────────────────────────────────────────
// Diffusion infilling
// ──────────────────────────────────────────────

#[derive(Deserialize)]
struct DiffuseBody {
    node_id: Uuid,
    /// Character ranges in the current script to preserve (user's anchored edits).
    anchor_ranges: Vec<CharRange>,
    /// Number of mask tokens to allocate for regenerated regions.
    mask_budget: usize,
}

#[derive(Deserialize)]
struct CharRange {
    start: usize,
    end: usize,
}

async fn diffuse(
    State(state): State<AppState>,
    Json(body): Json<DiffuseBody>,
) -> Result<Json<serde_json::Value>, (StatusCode, Json<serde_json::Value>)> {
    let node_id = NodeId(body.node_id);

    // ── PHASE 1: GATHER ──
    let script = {
        let snapshot = ydoc::read_content(&state.doc_tx, node_id).await;
        match snapshot {
            Some(s) => s.content,
            None => {
                return Err((
                    StatusCode::NOT_FOUND,
                    Json(serde_json::json!({ "error": "node not found in Y.Doc" })),
                ));
            }
        }
    };

    // ── PHASE 2: VALIDATE ──
    if body.mask_budget == 0 {
        return Err((
            StatusCode::BAD_REQUEST,
            Json(serde_json::json!({ "error": "mask_budget must be > 0" })),
        ));
    }

    if body.anchor_ranges.is_empty() {
        return Err((
            StatusCode::BAD_REQUEST,
            Json(serde_json::json!({ "error": "anchor_ranges must not be empty" })),
        ));
    }

    for range in &body.anchor_ranges {
        if range.start > range.end || range.end > script.len() {
            return Err((
                StatusCode::BAD_REQUEST,
                Json(serde_json::json!({
                    "error": format!(
                        "anchor range [{}, {}) out of bounds for script length {}",
                        range.start, range.end, script.len()
                    )
                })),
            ));
        }
    }

    if !state.diffusing.lock().insert(body.node_id) {
        return Err((
            StatusCode::CONFLICT,
            Json(serde_json::json!({ "error": "diffusion already in progress for this node" })),
        ));
    }

    // ── PHASE 3: EXECUTE ──
    // Build prefix and suffix from anchor ranges.
    // For simplicity: use the first anchor's start as the prefix boundary
    // and the last anchor's end as the suffix boundary.
    let first_anchor_start = body
        .anchor_ranges
        .iter()
        .map(|r| r.start)
        .min()
        .unwrap_or(0);
    let last_anchor_end = body
        .anchor_ranges
        .iter()
        .map(|r| r.end)
        .max()
        .unwrap_or(script.len());

    let prefix = script[..first_anchor_start].to_string();
    let suffix = script[last_anchor_end..].to_string();

    let config = state.ai_config.lock().clone();
    let mask_budget = body.mask_budget;
    let node_uuid = body.node_id;

    let state_clone = state.clone();
    tokio::spawn(async move {
        run_diffusion(
            state_clone,
            node_uuid,
            prefix,
            suffix,
            mask_budget,
            first_anchor_start,
            last_anchor_end,
            &config,
        )
        .await;
    });

    Ok(Json(serde_json::json!({
        "status": "started",
        "node_id": body.node_id.to_string(),
    })))
}

async fn run_diffusion(
    state: AppState,
    node_uuid: Uuid,
    prefix: String,
    suffix: String,
    mask_budget: usize,
    rewrite_start: usize,
    rewrite_end: usize,
    config: &crate::state::AiConfig,
) {
    let node_id = NodeId(node_uuid);

    // Subscribe to diffusion progress updates and re-broadcast as ServerEvents.
    let mut update_rx = state.diffuse_update_tx.subscribe();
    let events_tx = state.events_tx.clone();
    let progress_node_uuid = node_uuid;
    let progress_task = tokio::spawn(async move {
        while let Ok(update) = update_rx.recv().await {
            let _ = events_tx.send(ServerEvent::DiffusionProgress {
                node_id: progress_node_uuid,
                step: update.step,
                total_steps: update.total_steps,
            });
        }
    });

    // Send the infill command.
    let (reply_tx, reply_rx) = tokio::sync::oneshot::channel();
    let send_result = state
        .diffuse_tx
        .send(DiffuseCmd::Infill {
            prefix,
            suffix,
            mask_count: mask_budget,
            steps_per_block: config.diffusion_steps_per_block,
            block_length: config.diffusion_block_length,
            temperature: config.diffusion_temperature,
            dynamic_threshold: config.diffusion_dynamic_threshold,
            reply: reply_tx,
        })
        .await;

    if send_result.is_err() {
        let _ = state.events_tx.send(ServerEvent::DiffusionError {
            node_id: node_uuid,
            error: "diffusion manager channel closed".into(),
        });
        state.diffusing.lock().remove(&node_uuid);
        progress_task.abort();
        return;
    }

    // Await the result.
    match reply_rx.await {
        Ok(Ok(generated_text)) => {
            // ── PHASE 4: UPDATE ──
            // Write the generated text into the Y.Doc, replacing the region
            // between the anchors.
            let _ = state
                .doc_tx
                .send(DocCommand::RewriteRegion {
                    node_id,
                    field: ContentField::Content,
                    start: rewrite_start,
                    end: rewrite_end,
                    new_text: generated_text,
                    author: format!("ai:diffuse-{node_uuid}"),
                })
                .await;

            let _ = state.events_tx.send(ServerEvent::DiffusionComplete {
                node_id: node_uuid,
            });
            let _ = state.events_tx.send(ServerEvent::NodeUpdated {
                node_id: node_uuid,
            });
            state.trigger_save();
        }
        Ok(Err(DiffusionError::ModelNotLoaded)) => {
            let _ = state.events_tx.send(ServerEvent::DiffusionError {
                node_id: node_uuid,
                error: DiffusionError::ModelNotLoaded.to_string(),
            });
        }
        Ok(Err(e)) => {
            tracing::error!("Diffusion infill failed for node {node_uuid}: {e}");
            let _ = state.events_tx.send(ServerEvent::DiffusionError {
                node_id: node_uuid,
                error: e.to_string(),
            });
        }
        Err(_) => {
            let _ = state.events_tx.send(ServerEvent::DiffusionError {
                node_id: node_uuid,
                error: "diffusion manager reply dropped".into(),
            });
        }
    }

    state.diffusing.lock().remove(&node_uuid);
    progress_task.abort();
}

fn channel_closed_error() -> (StatusCode, Json<serde_json::Value>) {
    (
        StatusCode::SERVICE_UNAVAILABLE,
        Json(serde_json::json!({ "error": "diffusion manager unavailable" })),
    )
}
