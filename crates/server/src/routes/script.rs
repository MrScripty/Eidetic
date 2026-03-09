use axum::extract::{Path, State};
use axum::routing::{get, post, put};
use axum::{Json, Router};
use serde::Deserialize;
use uuid::Uuid;

use eidetic_core::timeline::node::{ContentStatus, NodeId};

use crate::error::{json_value, ApiError, ApiJson};
use crate::state::{AppState, ServerEvent};
use crate::ydoc::{ContentField, DocCommand};

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
) -> ApiJson {
    let guard = state.project.lock();
    let Some(project) = guard.as_ref() else {
        return Err(ApiError::no_project());
    };

    match project.timeline.node(NodeId(id)) {
        Ok(node) => json_value(&node.content),
        Err(e) => Err(ApiError::bad_request(e.to_string())),
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
) -> ApiJson {
    state.snapshot_for_undo();
    let json = {
        let mut guard = state.project.lock();
        let Some(project) = guard.as_mut() else {
            return Err(ApiError::no_project());
        };

        match project.timeline.node_mut(NodeId(id)) {
            Ok(node) => {
                node.content.notes = body.notes.clone();
                if node.content.status == ContentStatus::Empty {
                    node.content.status = ContentStatus::NotesOnly;
                }
                serde_json::to_value(&node.content)
                    .map_err(|e| ApiError::internal(e.to_string()))?
            }
            Err(e) => return Err(ApiError::bad_request(e.to_string())),
        }
    };
    // Mirror to Y.Doc (fire-and-forget).
    let _ = state.doc_tx.try_send(DocCommand::WriteNodeContent {
        node_id: NodeId(id),
        field: ContentField::Notes,
        text: body.notes,
        author: "human:rest".into(),
    });
    let _ = state.events_tx.send(ServerEvent::NodeUpdated { node_id: id });
    state.trigger_save();
    Ok(Json(json))
}

#[derive(Deserialize)]
struct UpdateScriptRequest {
    script: String,
}

async fn update_script(
    State(state): State<AppState>,
    Path(id): Path<Uuid>,
    Json(body): Json<UpdateScriptRequest>,
) -> ApiJson {
    state.snapshot_for_undo();
    let json = {
        let mut guard = state.project.lock();
        let Some(project) = guard.as_mut() else {
            return Err(ApiError::no_project());
        };

        match project.timeline.node_mut(NodeId(id)) {
            Ok(node) => {
                node.content.content = body.script.clone();
                node.content.status = ContentStatus::HasContent;
                serde_json::to_value(&node.content)
                    .map_err(|e| ApiError::internal(e.to_string()))?
            }
            Err(e) => return Err(ApiError::bad_request(e.to_string())),
        }
    };
    // Mirror to Y.Doc (fire-and-forget).
    let _ = state.doc_tx.try_send(DocCommand::WriteNodeContent {
        node_id: NodeId(id),
        field: ContentField::Content,
        text: body.script,
        author: "human:rest".into(),
    });
    let _ = state.events_tx.send(ServerEvent::NodeUpdated { node_id: id });
    state.trigger_save();
    Ok(Json(json))
}

async fn lock_node(
    State(state): State<AppState>,
    Path(id): Path<Uuid>,
) -> ApiJson {
    state.snapshot_for_undo();
    let mut guard = state.project.lock();
    let Some(project) = guard.as_mut() else {
        return Err(ApiError::no_project());
    };

    match project.timeline.node_mut(NodeId(id)) {
        Ok(node) => {
            node.locked = true;
            // Status unchanged — locking doesn't alter content state.
            let _ = state.events_tx.send(ServerEvent::NodeUpdated { node_id: id });
            state.trigger_save();
            Ok(Json(serde_json::json!({ "locked": true })))
        }
        Err(e) => Err(ApiError::bad_request(e.to_string())),
    }
}

async fn unlock_node(
    State(state): State<AppState>,
    Path(id): Path<Uuid>,
) -> ApiJson {
    state.snapshot_for_undo();
    let mut guard = state.project.lock();
    let Some(project) = guard.as_mut() else {
        return Err(ApiError::no_project());
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
            Ok(Json(serde_json::json!({ "locked": false })))
        }
        Err(e) => Err(ApiError::bad_request(e.to_string())),
    }
}
