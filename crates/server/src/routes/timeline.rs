use axum::extract::{Path, Query, State};
use axum::routing::{delete, get, post, put};
use axum::{Json, Router};
use serde::Deserialize;
use uuid::Uuid;

use eidetic_core::story::arc::ArcId;
use eidetic_core::timeline::node::{NodeId, StoryLevel};
use eidetic_core::timeline::track::TrackId;

use crate::error::{ApiError, ApiJson, json_value};
use crate::state::{AppState, ServerEvent};
use crate::validation;

pub fn router() -> Router<AppState> {
    Router::new()
        .route("/timeline", get(get_timeline))
        .route("/timeline/nodes/{id}/children", get(get_children))
        .route("/timeline/tracks", post(create_track))
        .route("/timeline/tracks/{id}", put(update_track))
        .route("/timeline/tracks/{id}", delete(delete_track))
        .route("/timeline/node-arcs", post(tag_node_with_arc))
        .route(
            "/timeline/node-arcs/{node_id}/{arc_id}",
            delete(untag_node_from_arc),
        )
        .route("/timeline/gaps", get(get_gaps))
}

// ─── Timeline ──────────────────────────────────────────────────────

async fn get_timeline(State(state): State<AppState>) -> ApiJson {
    let guard = state.project.lock();
    match guard.as_ref() {
        Some(p) => json_value(&p.timeline),
        None => Err(ApiError::no_project()),
    }
}

// ─── Node Queries And Planning ─────────────────────────────────────

async fn get_children(State(state): State<AppState>, Path(id): Path<Uuid>) -> ApiJson {
    let guard = state.project.lock();
    match guard.as_ref() {
        Some(p) => {
            let children = p.timeline.children_of(NodeId(id));
            json_value(&children)
        }
        None => Err(ApiError::no_project()),
    }
}

// ─── Track CRUD ────────────────────────────────────────────────────

#[derive(Deserialize)]
struct CreateTrackRequest {
    level: String,
    label: Option<String>,
}

async fn create_track(
    State(state): State<AppState>,
    Json(body): Json<CreateTrackRequest>,
) -> ApiJson {
    use eidetic_core::timeline::track::Track;

    state.snapshot_for_undo();
    let mut guard = state.project.lock();
    let Some(project) = guard.as_mut() else {
        return Err(ApiError::no_project());
    };

    let level = match parse_story_level(&body.level) {
        Some(l) => l,
        None => return Err(ApiError::bad_request("invalid level")),
    };

    let mut track = Track::new(level);
    if let Some(label) = body.label {
        validation::validate_name(&label, "track label")?;
        track.label = label;
    }

    let json = serde_json::to_value(&track).map_err(|e| ApiError::internal(e.to_string()))?;
    project.timeline.tracks.push(track);
    let _ = state.events_tx.send(ServerEvent::TimelineChanged);
    state.trigger_save();
    Ok(Json(json))
}

#[derive(Deserialize)]
struct UpdateTrackRequest {
    label: Option<String>,
    collapsed: Option<bool>,
}

async fn update_track(
    State(state): State<AppState>,
    Path(id): Path<Uuid>,
    Json(body): Json<UpdateTrackRequest>,
) -> ApiJson {
    let mut guard = state.project.lock();
    let Some(project) = guard.as_mut() else {
        return Err(ApiError::no_project());
    };

    match project.timeline.track_mut(TrackId(id)) {
        Ok(track) => {
            if let Some(label) = body.label {
                validation::validate_name(&label, "track label")?;
                track.label = label;
            }
            if let Some(collapsed) = body.collapsed {
                track.collapsed = collapsed;
            }
            let json =
                serde_json::to_value(&*track).map_err(|e| ApiError::internal(e.to_string()))?;
            let _ = state.events_tx.send(ServerEvent::TimelineChanged);
            state.trigger_save();
            Ok(Json(json))
        }
        Err(e) => Err(ApiError::bad_request(e.to_string())),
    }
}

async fn delete_track(State(state): State<AppState>, Path(id): Path<Uuid>) -> ApiJson {
    state.snapshot_for_undo();
    let mut guard = state.project.lock();
    let Some(project) = guard.as_mut() else {
        return Err(ApiError::no_project());
    };

    let track_id = TrackId(id);
    let idx = project
        .timeline
        .tracks
        .iter()
        .position(|t| t.id == track_id);
    match idx {
        Some(i) => {
            let track = project.timeline.tracks.remove(i);
            let _ = state.events_tx.send(ServerEvent::TimelineChanged);
            state.trigger_save();
            json_value(&track)
        }
        None => Err(ApiError::not_found("track not found")),
    }
}

// ─── Node-Arc Tagging ──────────────────────────────────────────────

#[derive(Deserialize)]
struct TagNodeArcRequest {
    node_id: Uuid,
    arc_id: Uuid,
}

async fn tag_node_with_arc(
    State(state): State<AppState>,
    Json(body): Json<TagNodeArcRequest>,
) -> ApiJson {
    state.snapshot_for_undo();
    let mut guard = state.project.lock();
    let Some(project) = guard.as_mut() else {
        return Err(ApiError::no_project());
    };

    project
        .timeline
        .tag_node(NodeId(body.node_id), ArcId(body.arc_id));
    let _ = state.events_tx.send(ServerEvent::TimelineChanged);
    state.trigger_save();
    Ok(Json(serde_json::json!({ "ok": true })))
}

async fn untag_node_from_arc(
    State(state): State<AppState>,
    Path((node_id, arc_id)): Path<(Uuid, Uuid)>,
) -> ApiJson {
    state.snapshot_for_undo();
    let mut guard = state.project.lock();
    let Some(project) = guard.as_mut() else {
        return Err(ApiError::no_project());
    };

    project.timeline.untag_node(NodeId(node_id), ArcId(arc_id));
    let _ = state.events_tx.send(ServerEvent::TimelineChanged);
    state.trigger_save();
    Ok(Json(serde_json::json!({ "ok": true })))
}

// ─── Gaps ──────────────────────────────────────────────────────────

#[derive(Deserialize)]
struct GapQuery {
    level: Option<String>,
}

async fn get_gaps(State(state): State<AppState>, Query(query): Query<GapQuery>) -> ApiJson {
    let guard = state.project.lock();
    match guard.as_ref() {
        Some(p) => {
            let level = query
                .level
                .as_deref()
                .and_then(parse_story_level)
                .unwrap_or(StoryLevel::Scene);
            let gaps = p
                .timeline
                .find_gaps(level, crate::state::constants::GAP_THRESHOLD_MS);
            json_value(&gaps)
        }
        None => Err(ApiError::no_project()),
    }
}

// ─── Helpers ───────────────────────────────────────────────────────

fn parse_story_level(s: &str) -> Option<StoryLevel> {
    match s {
        "premise" | "Premise" => Some(StoryLevel::Premise),
        "act" | "Act" => Some(StoryLevel::Act),
        "sequence" | "Sequence" => Some(StoryLevel::Sequence),
        "scene" | "Scene" => Some(StoryLevel::Scene),
        "beat" | "Beat" => Some(StoryLevel::Beat),
        _ => None,
    }
}
