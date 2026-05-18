use axum::Router;
use axum::extract::{Path, Query, State};
use axum::routing::get;
use serde::Deserialize;
use uuid::Uuid;

use eidetic_core::timeline::node::{NodeId, StoryLevel};

use crate::error::{ApiError, ApiJson, json_value};
use crate::state::AppState;

pub fn router() -> Router<AppState> {
    Router::new()
        .route("/timeline", get(get_timeline))
        .route("/timeline/nodes/{id}/children", get(get_children))
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
