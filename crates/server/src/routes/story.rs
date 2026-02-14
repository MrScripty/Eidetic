use axum::extract::{Path, State};
use axum::routing::{delete, get, post, put};
use axum::{Json, Router};
use serde::Deserialize;
use uuid::Uuid;

use eidetic_core::story::arc::{ArcType, Color, StoryArc};
use eidetic_core::story::character::Character;
use eidetic_core::story::progression::analyze_all_arcs;

use crate::state::{AppState, ServerEvent};

pub fn router() -> Router<AppState> {
    Router::new()
        .route("/arcs", get(list_arcs))
        .route("/arcs", post(create_arc))
        .route("/arcs/{id}", put(update_arc))
        .route("/arcs/{id}", delete(delete_arc))
        .route("/characters", get(list_characters))
        .route("/characters", post(create_character))
        .route("/characters/{id}", put(update_character))
        .route("/characters/{id}", delete(delete_character))
        .route("/arcs/progression", get(arc_progression))
}

// --- Arcs ---

async fn list_arcs(State(state): State<AppState>) -> Json<serde_json::Value> {
    let guard = state.project.lock();
    match guard.as_ref() {
        Some(p) => Json(serde_json::to_value(&p.arcs).unwrap()),
        None => Json(serde_json::json!([])),
    }
}

#[derive(Deserialize)]
struct CreateArcRequest {
    name: String,
    arc_type: String,
    color: Option<[u8; 3]>,
}

async fn create_arc(
    State(state): State<AppState>,
    Json(body): Json<CreateArcRequest>,
) -> Json<serde_json::Value> {
    state.snapshot_for_undo();
    let mut guard = state.project.lock();
    let Some(project) = guard.as_mut() else {
        return Json(serde_json::json!({ "error": "no project loaded" }));
    };

    let arc_type = match body.arc_type.as_str() {
        "a_plot" => ArcType::APlot,
        "b_plot" => ArcType::BPlot,
        "c_runner" => ArcType::CRunner,
        other => ArcType::Custom(other.to_string()),
    };

    let color = body
        .color
        .map(|[r, g, b]| Color::new(r, g, b))
        .unwrap_or(Color::new(180, 180, 180));

    let arc = StoryArc::new(body.name, arc_type, color);
    let json = serde_json::to_value(&arc).unwrap();
    project.arcs.push(arc);
    let _ = state.events_tx.send(ServerEvent::StoryChanged);
    state.trigger_save();
    Json(json)
}

async fn update_arc(
    State(state): State<AppState>,
    Path(id): Path<Uuid>,
    Json(body): Json<serde_json::Value>,
) -> Json<serde_json::Value> {
    state.snapshot_for_undo();
    let mut guard = state.project.lock();
    let Some(project) = guard.as_mut() else {
        return Json(serde_json::json!({ "error": "no project loaded" }));
    };

    if let Some(arc) = project.arcs.iter_mut().find(|a| a.id.0 == id) {
        if let Some(name) = body.get("name").and_then(|v| v.as_str()) {
            arc.name = name.to_string();
        }
        if let Some(desc) = body.get("description").and_then(|v| v.as_str()) {
            arc.description = desc.to_string();
        }
        if let Some(color_arr) = body.get("color").and_then(|v| v.as_array()) {
            if color_arr.len() == 3 {
                if let (Some(r), Some(g), Some(b)) = (
                    color_arr[0].as_u64(),
                    color_arr[1].as_u64(),
                    color_arr[2].as_u64(),
                ) {
                    arc.color = Color::new(r as u8, g as u8, b as u8);
                }
            }
        }
        let json = serde_json::to_value(&*arc).unwrap();
        let _ = state.events_tx.send(ServerEvent::StoryChanged);
        Json(json)
    } else {
        Json(serde_json::json!({ "error": "arc not found" }))
    }
}

async fn delete_arc(
    State(state): State<AppState>,
    Path(id): Path<Uuid>,
) -> Json<serde_json::Value> {
    state.snapshot_for_undo();
    let mut guard = state.project.lock();
    let Some(project) = guard.as_mut() else {
        return Json(serde_json::json!({ "error": "no project loaded" }));
    };

    let before = project.arcs.len();
    project.arcs.retain(|a| a.id.0 != id);
    let deleted = before != project.arcs.len();
    if deleted {
        let _ = state.events_tx.send(ServerEvent::StoryChanged);
        state.trigger_save();
    }
    Json(serde_json::json!({ "deleted": deleted }))
}

// --- Characters ---

async fn list_characters(State(state): State<AppState>) -> Json<serde_json::Value> {
    let guard = state.project.lock();
    match guard.as_ref() {
        Some(p) => Json(serde_json::to_value(&p.characters).unwrap()),
        None => Json(serde_json::json!([])),
    }
}

#[derive(Deserialize)]
struct CreateCharacterRequest {
    name: String,
    color: Option<[u8; 3]>,
}

async fn create_character(
    State(state): State<AppState>,
    Json(body): Json<CreateCharacterRequest>,
) -> Json<serde_json::Value> {
    state.snapshot_for_undo();
    let mut guard = state.project.lock();
    let Some(project) = guard.as_mut() else {
        return Json(serde_json::json!({ "error": "no project loaded" }));
    };

    let color = body
        .color
        .map(|[r, g, b]| Color::new(r, g, b))
        .unwrap_or(Color::new(200, 200, 200));

    let character = Character::new(body.name, color);
    let json = serde_json::to_value(&character).unwrap();
    project.characters.push(character);
    let _ = state.events_tx.send(ServerEvent::StoryChanged);
    state.trigger_save();
    Json(json)
}

async fn update_character(
    State(state): State<AppState>,
    Path(id): Path<Uuid>,
    Json(body): Json<serde_json::Value>,
) -> Json<serde_json::Value> {
    state.snapshot_for_undo();
    let mut guard = state.project.lock();
    let Some(project) = guard.as_mut() else {
        return Json(serde_json::json!({ "error": "no project loaded" }));
    };

    if let Some(ch) = project.characters.iter_mut().find(|c| c.id.0 == id) {
        if let Some(name) = body.get("name").and_then(|v| v.as_str()) {
            ch.name = name.to_string();
        }
        if let Some(desc) = body.get("description").and_then(|v| v.as_str()) {
            ch.description = desc.to_string();
        }
        if let Some(voice) = body.get("voice_notes").and_then(|v| v.as_str()) {
            ch.voice_notes = voice.to_string();
        }
        if let Some(color_arr) = body.get("color").and_then(|v| v.as_array()) {
            if color_arr.len() == 3 {
                if let (Some(r), Some(g), Some(b)) = (
                    color_arr[0].as_u64(),
                    color_arr[1].as_u64(),
                    color_arr[2].as_u64(),
                ) {
                    ch.color = Color::new(r as u8, g as u8, b as u8);
                }
            }
        }
        let json = serde_json::to_value(&*ch).unwrap();
        let _ = state.events_tx.send(ServerEvent::StoryChanged);
        Json(json)
    } else {
        Json(serde_json::json!({ "error": "character not found" }))
    }
}

async fn delete_character(
    State(state): State<AppState>,
    Path(id): Path<Uuid>,
) -> Json<serde_json::Value> {
    state.snapshot_for_undo();
    let mut guard = state.project.lock();
    let Some(project) = guard.as_mut() else {
        return Json(serde_json::json!({ "error": "no project loaded" }));
    };

    let before = project.characters.len();
    project.characters.retain(|c| c.id.0 != id);
    let deleted = before != project.characters.len();
    if deleted {
        let _ = state.events_tx.send(ServerEvent::StoryChanged);
        state.trigger_save();
    }
    Json(serde_json::json!({ "deleted": deleted }))
}

async fn arc_progression(State(state): State<AppState>) -> Json<serde_json::Value> {
    let guard = state.project.lock();
    match guard.as_ref() {
        Some(p) => {
            let progressions = analyze_all_arcs(p);
            Json(serde_json::to_value(&progressions).unwrap())
        }
        None => Json(serde_json::json!([])),
    }
}
