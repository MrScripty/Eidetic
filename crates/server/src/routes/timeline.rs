use axum::extract::{Path, Query, State};
use axum::routing::{delete, get, post};
use axum::{Json, Router};
use serde::Deserialize;
use uuid::Uuid;

use eidetic_core::story::arc::ArcId;
use eidetic_core::timeline::node::{NodeId, StoryLevel};

use crate::error::{ApiError, ApiJson, json_value};
use crate::state::{AppState, ServerEvent};

pub fn router() -> Router<AppState> {
    Router::new()
        .route("/timeline", get(get_timeline))
        .route("/timeline/nodes/{id}/children", get(get_children))
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
