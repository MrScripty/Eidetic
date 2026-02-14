use axum::extract::{Path, State};
use axum::routing::{get, post, put};
use axum::{Json, Router};
use serde::Deserialize;
use uuid::Uuid;

use eidetic_core::timeline::clip::{ClipId, ContentStatus};

use crate::state::{AppState, ServerEvent};

pub fn router() -> Router<AppState> {
    Router::new()
        .route("/beats/{id}", get(get_beat))
        .route("/beats/{id}/notes", put(update_notes))
        .route("/beats/{id}/script", put(update_script))
        .route("/beats/{id}/lock", post(lock_beat))
        .route("/beats/{id}/unlock", post(unlock_beat))
}

async fn get_beat(
    State(state): State<AppState>,
    Path(id): Path<Uuid>,
) -> Json<serde_json::Value> {
    let guard = state.project.lock();
    let Some(project) = guard.as_ref() else {
        return Json(serde_json::json!({ "error": "no project loaded" }));
    };

    match project.timeline.clip(ClipId(id)) {
        Ok(clip) => Json(serde_json::to_value(&clip.content).unwrap()),
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
    let mut guard = state.project.lock();
    let Some(project) = guard.as_mut() else {
        return Json(serde_json::json!({ "error": "no project loaded" }));
    };

    match project.timeline.clip_mut(ClipId(id)) {
        Ok(clip) => {
            clip.content.beat_notes = body.notes;
            if clip.content.status == ContentStatus::Empty {
                clip.content.status = ContentStatus::NotesOnly;
            }
            let json = serde_json::to_value(&clip.content).unwrap();
            let _ = state.events_tx.send(ServerEvent::BeatUpdated { clip_id: id });
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
    let mut guard = state.project.lock();
    let Some(project) = guard.as_mut() else {
        return Json(serde_json::json!({ "error": "no project loaded" }));
    };

    match project.timeline.clip_mut(ClipId(id)) {
        Ok(clip) => {
            clip.content.user_refined_script = Some(body.script);
            clip.content.status = ContentStatus::UserRefined;
            let json = serde_json::to_value(&clip.content).unwrap();
            let _ = state.events_tx.send(ServerEvent::BeatUpdated { clip_id: id });
            Json(json)
        }
        Err(e) => Json(serde_json::json!({ "error": e.to_string() })),
    }
}

async fn lock_beat(
    State(state): State<AppState>,
    Path(id): Path<Uuid>,
) -> Json<serde_json::Value> {
    let mut guard = state.project.lock();
    let Some(project) = guard.as_mut() else {
        return Json(serde_json::json!({ "error": "no project loaded" }));
    };

    match project.timeline.clip_mut(ClipId(id)) {
        Ok(clip) => {
            clip.locked = true;
            clip.content.status = ContentStatus::UserWritten;
            let _ = state.events_tx.send(ServerEvent::BeatUpdated { clip_id: id });
            Json(serde_json::json!({ "locked": true }))
        }
        Err(e) => Json(serde_json::json!({ "error": e.to_string() })),
    }
}

async fn unlock_beat(
    State(state): State<AppState>,
    Path(id): Path<Uuid>,
) -> Json<serde_json::Value> {
    let mut guard = state.project.lock();
    let Some(project) = guard.as_mut() else {
        return Json(serde_json::json!({ "error": "no project loaded" }));
    };

    match project.timeline.clip_mut(ClipId(id)) {
        Ok(clip) => {
            clip.locked = false;
            // Revert status based on what content exists.
            clip.content.status = if clip.content.user_refined_script.is_some() {
                ContentStatus::UserRefined
            } else if clip.content.generated_script.is_some() {
                ContentStatus::Generated
            } else if !clip.content.beat_notes.is_empty() {
                ContentStatus::NotesOnly
            } else {
                ContentStatus::Empty
            };
            let _ = state.events_tx.send(ServerEvent::BeatUpdated { clip_id: id });
            Json(serde_json::json!({ "locked": false }))
        }
        Err(e) => Json(serde_json::json!({ "error": e.to_string() })),
    }
}
