use axum::extract::{Path, State};
use axum::routing::{delete, get, post, put};
use axum::{Json, Router};
use serde::Deserialize;
use uuid::Uuid;

use eidetic_core::story::arc::ArcId;
use eidetic_core::timeline::clip::{BeatClip, BeatType, ClipId};
use eidetic_core::timeline::relationship::{Relationship, RelationshipId, RelationshipType};
use eidetic_core::timeline::timing::TimeRange;
use eidetic_core::timeline::track::{ArcTrack, TrackId};

use crate::state::{AppState, ServerEvent};

pub fn router() -> Router<AppState> {
    Router::new()
        .route("/timeline", get(get_timeline))
        .route("/timeline/tracks", post(add_track))
        .route("/timeline/tracks/{id}", delete(remove_track))
        .route("/timeline/clips", post(create_clip))
        .route("/timeline/clips/{id}", put(update_clip))
        .route("/timeline/clips/{id}", delete(delete_clip))
        .route("/timeline/clips/{id}/split", post(split_clip))
        .route("/timeline/relationships", post(create_relationship))
        .route("/timeline/relationships/{id}", delete(delete_relationship))
        .route("/scenes", get(get_scenes))
        .route("/timeline/tracks/{id}/close-gap", post(close_gap))
        .route("/timeline/tracks/{id}/close-all-gaps", post(close_all_gaps))
        .route("/timeline/gaps", get(get_gaps))
        .route("/timeline/gaps/fill", post(fill_gap))
}

async fn get_timeline(State(state): State<AppState>) -> Json<serde_json::Value> {
    let guard = state.project.lock();
    match guard.as_ref() {
        Some(p) => Json(serde_json::to_value(&p.timeline).unwrap()),
        None => Json(serde_json::json!({ "error": "no project loaded" })),
    }
}

#[derive(Deserialize)]
struct AddTrackRequest {
    arc_id: Uuid,
}

async fn add_track(
    State(state): State<AppState>,
    Json(body): Json<AddTrackRequest>,
) -> Json<serde_json::Value> {
    state.snapshot_for_undo();
    let mut guard = state.project.lock();
    let Some(project) = guard.as_mut() else {
        return Json(serde_json::json!({ "error": "no project loaded" }));
    };

    let track = ArcTrack::new(ArcId(body.arc_id));
    let json = serde_json::to_value(&track).unwrap();
    match project.timeline.add_track(track) {
        Ok(()) => {
            let _ = state.events_tx.send(ServerEvent::TimelineChanged);
            let _ = state.events_tx.send(ServerEvent::ScenesChanged);
            state.trigger_save();
            Json(json)
        }
        Err(e) => Json(serde_json::json!({ "error": e.to_string() })),
    }
}

async fn remove_track(
    State(state): State<AppState>,
    Path(id): Path<Uuid>,
) -> Json<serde_json::Value> {
    state.snapshot_for_undo();
    let mut guard = state.project.lock();
    let Some(project) = guard.as_mut() else {
        return Json(serde_json::json!({ "error": "no project loaded" }));
    };

    match project.timeline.remove_track(TrackId(id)) {
        Ok(track) => {
            let _ = state.events_tx.send(ServerEvent::TimelineChanged);
            let _ = state.events_tx.send(ServerEvent::ScenesChanged);
            state.trigger_save();
            Json(serde_json::to_value(&track).unwrap())
        }
        Err(e) => Json(serde_json::json!({ "error": e.to_string() })),
    }
}

#[derive(Deserialize)]
struct CreateClipRequest {
    track_id: Uuid,
    name: String,
    beat_type: String,
    start_ms: u64,
    end_ms: u64,
}

async fn create_clip(
    State(state): State<AppState>,
    Json(body): Json<CreateClipRequest>,
) -> Json<serde_json::Value> {
    state.snapshot_for_undo();
    let mut guard = state.project.lock();
    let Some(project) = guard.as_mut() else {
        return Json(serde_json::json!({ "error": "no project loaded" }));
    };

    let beat_type = parse_beat_type(&body.beat_type);
    let time_range = match TimeRange::new(body.start_ms, body.end_ms) {
        Ok(r) => r,
        Err(e) => return Json(serde_json::json!({ "error": e.to_string() })),
    };

    let clip = BeatClip::new(body.name, beat_type, time_range);
    let json = serde_json::to_value(&clip).unwrap();

    match project.timeline.add_clip(TrackId(body.track_id), clip) {
        Ok(()) => {
            let _ = state.events_tx.send(ServerEvent::TimelineChanged);
            let _ = state.events_tx.send(ServerEvent::ScenesChanged);
            state.trigger_save();
            Json(json)
        }
        Err(e) => Json(serde_json::json!({ "error": e.to_string() })),
    }
}

#[derive(Deserialize)]
struct UpdateClipRequest {
    name: Option<String>,
    start_ms: Option<u64>,
    end_ms: Option<u64>,
}

async fn update_clip(
    State(state): State<AppState>,
    Path(id): Path<Uuid>,
    Json(body): Json<UpdateClipRequest>,
) -> Json<serde_json::Value> {
    state.snapshot_for_undo();
    let mut guard = state.project.lock();
    let Some(project) = guard.as_mut() else {
        return Json(serde_json::json!({ "error": "no project loaded" }));
    };

    // If position changed, use move_clip for validation.
    if let (Some(start), Some(end)) = (body.start_ms, body.end_ms) {
        let range = match TimeRange::new(start, end) {
            Ok(r) => r,
            Err(e) => return Json(serde_json::json!({ "error": e.to_string() })),
        };
        if let Err(e) = project.timeline.move_clip(ClipId(id), range) {
            return Json(serde_json::json!({ "error": e.to_string() }));
        }
    }

    match project.timeline.clip_mut(ClipId(id)) {
        Ok(clip) => {
            if let Some(name) = body.name {
                clip.name = name;
            }
            let json = serde_json::to_value(&*clip).unwrap();
            let _ = state.events_tx.send(ServerEvent::TimelineChanged);
            let _ = state.events_tx.send(ServerEvent::ScenesChanged);
            state.trigger_save();
            Json(json)
        }
        Err(e) => Json(serde_json::json!({ "error": e.to_string() })),
    }
}

async fn delete_clip(
    State(state): State<AppState>,
    Path(id): Path<Uuid>,
) -> Json<serde_json::Value> {
    state.snapshot_for_undo();
    let mut guard = state.project.lock();
    let Some(project) = guard.as_mut() else {
        return Json(serde_json::json!({ "error": "no project loaded" }));
    };

    match project.timeline.remove_clip(ClipId(id)) {
        Ok(clip) => {
            let _ = state.events_tx.send(ServerEvent::TimelineChanged);
            let _ = state.events_tx.send(ServerEvent::ScenesChanged);
            state.trigger_save();
            Json(serde_json::to_value(&clip).unwrap())
        }
        Err(e) => Json(serde_json::json!({ "error": e.to_string() })),
    }
}

#[derive(Deserialize)]
struct SplitClipRequest {
    at_ms: u64,
}

async fn split_clip(
    State(state): State<AppState>,
    Path(id): Path<Uuid>,
    Json(body): Json<SplitClipRequest>,
) -> Json<serde_json::Value> {
    state.snapshot_for_undo();
    let mut guard = state.project.lock();
    let Some(project) = guard.as_mut() else {
        return Json(serde_json::json!({ "error": "no project loaded" }));
    };

    match project.timeline.split_clip(ClipId(id), body.at_ms) {
        Ok((left, right)) => {
            let _ = state.events_tx.send(ServerEvent::TimelineChanged);
            let _ = state.events_tx.send(ServerEvent::ScenesChanged);
            state.trigger_save();
            Json(serde_json::json!({ "left_id": left.0, "right_id": right.0 }))
        }
        Err(e) => Json(serde_json::json!({ "error": e.to_string() })),
    }
}

#[derive(Deserialize)]
struct CreateRelationshipRequest {
    from_clip: Uuid,
    to_clip: Uuid,
    relationship_type: String,
}

async fn create_relationship(
    State(state): State<AppState>,
    Json(body): Json<CreateRelationshipRequest>,
) -> Json<serde_json::Value> {
    state.snapshot_for_undo();
    let mut guard = state.project.lock();
    let Some(project) = guard.as_mut() else {
        return Json(serde_json::json!({ "error": "no project loaded" }));
    };

    let rel_type = match body.relationship_type.as_str() {
        "convergence" => RelationshipType::Convergence { arc_ids: vec![] },
        "thematic" => RelationshipType::Thematic,
        _ => RelationshipType::Causal,
    };

    let rel = Relationship::new(ClipId(body.from_clip), ClipId(body.to_clip), rel_type);
    let json = serde_json::to_value(&rel).unwrap();

    match project.timeline.add_relationship(rel) {
        Ok(()) => {
            let _ = state.events_tx.send(ServerEvent::TimelineChanged);
            state.trigger_save();
            Json(json)
        }
        Err(e) => Json(serde_json::json!({ "error": e.to_string() })),
    }
}

async fn delete_relationship(
    State(state): State<AppState>,
    Path(id): Path<Uuid>,
) -> Json<serde_json::Value> {
    state.snapshot_for_undo();
    let mut guard = state.project.lock();
    let Some(project) = guard.as_mut() else {
        return Json(serde_json::json!({ "error": "no project loaded" }));
    };

    match project.timeline.remove_relationship(RelationshipId(id)) {
        Ok(rel) => {
            let _ = state.events_tx.send(ServerEvent::TimelineChanged);
            state.trigger_save();
            Json(serde_json::to_value(&rel).unwrap())
        }
        Err(e) => Json(serde_json::json!({ "error": e.to_string() })),
    }
}

async fn get_scenes(State(state): State<AppState>) -> Json<serde_json::Value> {
    let guard = state.project.lock();
    match guard.as_ref() {
        Some(p) => {
            let scenes = p.timeline.infer_scenes();
            Json(serde_json::to_value(&scenes).unwrap())
        }
        None => Json(serde_json::json!([])),
    }
}

async fn get_gaps(State(state): State<AppState>) -> Json<serde_json::Value> {
    let guard = state.project.lock();
    match guard.as_ref() {
        Some(p) => {
            let gaps = p.timeline.find_gaps(crate::state::constants::GAP_THRESHOLD_MS);
            Json(serde_json::to_value(&gaps).unwrap())
        }
        None => Json(serde_json::json!([])),
    }
}

#[derive(Deserialize)]
struct FillGapRequest {
    track_id: Uuid,
    start_ms: u64,
    end_ms: u64,
}

async fn fill_gap(
    State(state): State<AppState>,
    Json(body): Json<FillGapRequest>,
) -> Json<serde_json::Value> {
    state.snapshot_for_undo();
    let mut guard = state.project.lock();
    let Some(project) = guard.as_mut() else {
        return Json(serde_json::json!({ "error": "no project loaded" }));
    };

    let time_range = match TimeRange::new(body.start_ms, body.end_ms) {
        Ok(r) => r,
        Err(e) => return Json(serde_json::json!({ "error": e.to_string() })),
    };

    let clip = BeatClip::new("Bridge".to_string(), BeatType::Custom("bridge".to_string()), time_range);
    let json = serde_json::to_value(&clip).unwrap();

    match project.timeline.add_clip(TrackId(body.track_id), clip) {
        Ok(()) => {
            let _ = state.events_tx.send(ServerEvent::TimelineChanged);
            let _ = state.events_tx.send(ServerEvent::ScenesChanged);
            state.trigger_save();
            Json(json)
        }
        Err(e) => Json(serde_json::json!({ "error": e.to_string() })),
    }
}

#[derive(Deserialize)]
struct CloseGapRequest {
    gap_end_ms: u64,
}

async fn close_gap(
    State(state): State<AppState>,
    Path(id): Path<Uuid>,
    Json(body): Json<CloseGapRequest>,
) -> Json<serde_json::Value> {
    state.snapshot_for_undo();
    let mut guard = state.project.lock();
    let Some(project) = guard.as_mut() else {
        return Json(serde_json::json!({ "error": "no project loaded" }));
    };

    match project.timeline.close_gap(TrackId(id), body.gap_end_ms) {
        Ok(()) => {
            let _ = state.events_tx.send(ServerEvent::TimelineChanged);
            let _ = state.events_tx.send(ServerEvent::ScenesChanged);
            state.trigger_save();
            Json(serde_json::to_value(&project.timeline).unwrap())
        }
        Err(e) => Json(serde_json::json!({ "error": e.to_string() })),
    }
}

async fn close_all_gaps(
    State(state): State<AppState>,
    Path(id): Path<Uuid>,
) -> Json<serde_json::Value> {
    state.snapshot_for_undo();
    let mut guard = state.project.lock();
    let Some(project) = guard.as_mut() else {
        return Json(serde_json::json!({ "error": "no project loaded" }));
    };

    match project.timeline.close_all_gaps(TrackId(id)) {
        Ok(()) => {
            let _ = state.events_tx.send(ServerEvent::TimelineChanged);
            let _ = state.events_tx.send(ServerEvent::ScenesChanged);
            state.trigger_save();
            Json(serde_json::to_value(&project.timeline).unwrap())
        }
        Err(e) => Json(serde_json::json!({ "error": e.to_string() })),
    }
}

fn parse_beat_type(s: &str) -> BeatType {
    match s {
        "setup" => BeatType::Setup,
        "complication" => BeatType::Complication,
        "escalation" => BeatType::Escalation,
        "climax" => BeatType::Climax,
        "resolution" => BeatType::Resolution,
        "payoff" => BeatType::Payoff,
        "callback" => BeatType::Callback,
        other => BeatType::Custom(other.to_string()),
    }
}
