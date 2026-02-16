use axum::extract::{Path, State};
use axum::routing::{get, post, put};
use axum::{Json, Router};
use serde::Deserialize;
use uuid::Uuid;

use eidetic_core::timeline::node::{ContentStatus, NodeId};

use crate::state::{AppState, ServerEvent};

pub fn router() -> Router<AppState> {
    Router::new()
        .route("/nodes/{id}/content", get(get_node_content))
        .route("/nodes/{id}/notes", put(update_notes))
        .route("/nodes/{id}/script", put(update_script))
        .route("/nodes/{id}/lock", post(lock_node))
        .route("/nodes/{id}/unlock", post(unlock_node))
}

async fn get_node_content(
    State(state): State<AppState>,
    Path(id): Path<Uuid>,
) -> Json<serde_json::Value> {
    let guard = state.project.lock();
    let Some(project) = guard.as_ref() else {
        return Json(serde_json::json!({ "error": "no project loaded" }));
    };

    match project.timeline.node(NodeId(id)) {
        Ok(node) => Json(serde_json::to_value(&node.content).unwrap()),
        Err(e) => Json(serde_json::json!({ "error": e.to_string() })),
    }
}

#[derive(Deserialize)]
struct UpdateNotesRequest {
    notes: String,
}

async fn update_notes(
    State(state): State<AppState>,
    Path(id): Path<Uuid>,
    Json(body): Json<UpdateNotesRequest>,
) -> Json<serde_json::Value> {
    state.snapshot_for_undo();
    let mut guard = state.project.lock();
    let Some(project) = guard.as_mut() else {
        return Json(serde_json::json!({ "error": "no project loaded" }));
    };

    match project.timeline.node_mut(NodeId(id)) {
        Ok(node) => {
            node.content.notes = body.notes;
            if node.content.status == ContentStatus::Empty {
                node.content.status = ContentStatus::NotesOnly;
            }
            let json = serde_json::to_value(&node.content).unwrap();
            let _ = state.events_tx.send(ServerEvent::NodeUpdated { node_id: id });
            state.trigger_save();
            Json(json)
        }
        Err(e) => Json(serde_json::json!({ "error": e.to_string() })),
    }
}

#[derive(Deserialize)]
struct UpdateScriptRequest {
    script: String,
}

async fn update_script(
    State(state): State<AppState>,
    Path(id): Path<Uuid>,
    Json(body): Json<UpdateScriptRequest>,
) -> Json<serde_json::Value> {
    state.snapshot_for_undo();
    let mut guard = state.project.lock();
    let Some(project) = guard.as_mut() else {
        return Json(serde_json::json!({ "error": "no project loaded" }));
    };

    match project.timeline.node_mut(NodeId(id)) {
        Ok(node) => {
            node.content.content = body.script;
            node.content.status = ContentStatus::HasContent;
            let json = serde_json::to_value(&node.content).unwrap();
            let _ = state.events_tx.send(ServerEvent::NodeUpdated { node_id: id });
            state.trigger_save();
            Json(json)
        }
        Err(e) => Json(serde_json::json!({ "error": e.to_string() })),
    }
}

async fn lock_node(
    State(state): State<AppState>,
    Path(id): Path<Uuid>,
) -> Json<serde_json::Value> {
    state.snapshot_for_undo();
    let mut guard = state.project.lock();
    let Some(project) = guard.as_mut() else {
        return Json(serde_json::json!({ "error": "no project loaded" }));
    };

    match project.timeline.node_mut(NodeId(id)) {
        Ok(node) => {
            node.locked = true;
            // Status unchanged — locking doesn't alter content state.
            let _ = state.events_tx.send(ServerEvent::NodeUpdated { node_id: id });
            state.trigger_save();
            Json(serde_json::json!({ "locked": true }))
        }
        Err(e) => Json(serde_json::json!({ "error": e.to_string() })),
    }
}

async fn unlock_node(
    State(state): State<AppState>,
    Path(id): Path<Uuid>,
) -> Json<serde_json::Value> {
    state.snapshot_for_undo();
    let mut guard = state.project.lock();
    let Some(project) = guard.as_mut() else {
        return Json(serde_json::json!({ "error": "no project loaded" }));
    };

    match project.timeline.node_mut(NodeId(id)) {
        Ok(node) => {
            node.locked = false;
            // Recompute status from current content state.
            node.content.status = if !node.content.content.is_empty() {
                ContentStatus::HasContent
            } else if !node.content.notes.is_empty() {
                ContentStatus::NotesOnly
            } else {
                ContentStatus::Empty
            };
            let _ = state.events_tx.send(ServerEvent::NodeUpdated { node_id: id });
            state.trigger_save();
            Json(serde_json::json!({ "locked": false }))
        }
        Err(e) => Json(serde_json::json!({ "error": e.to_string() })),
    }
}
