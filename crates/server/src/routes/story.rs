use axum::extract::{Path, State};
use axum::routing::{delete, get, post, put};
use axum::{Json, Router};
use serde::Deserialize;
use uuid::Uuid;

use eidetic_core::story::arc::{ArcType, Color, StoryArc};
use eidetic_core::story::progression::analyze_all_arcs;

use crate::error::{ApiError, ApiJson, json_value};
use crate::state::{AppState, ServerEvent};
use crate::validation;

pub fn router() -> Router<AppState> {
    Router::new()
        .route("/arcs", get(list_arcs))
        .route("/arcs", post(create_arc))
        .route("/arcs/{id}", put(update_arc))
        .route("/arcs/{id}", delete(delete_arc))
        .route("/arcs/progression", get(arc_progression))
}

// ──────────────────────────────────────────────
// Arcs (unchanged)
// ──────────────────────────────────────────────

async fn list_arcs(State(state): State<AppState>) -> ApiJson {
    let guard = state.project.lock();
    match guard.as_ref() {
        Some(p) => json_value(&p.arcs),
        None => Err(ApiError::no_project()),
    }
}

#[derive(Deserialize)]
struct CreateArcRequest {
    name: String,
    arc_type: String,
    color: Option<[u8; 3]>,
}

async fn create_arc(State(state): State<AppState>, Json(body): Json<CreateArcRequest>) -> ApiJson {
    validation::validate_name(&body.name, "arc name")?;
    let mut guard = state.project.lock();
    let Some(project) = guard.as_mut() else {
        return Err(ApiError::no_project());
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
    let json = serde_json::to_value(&arc).map_err(|e| ApiError::internal(e.to_string()))?;
    project.arcs.push(arc);
    let _ = state.events_tx.send(ServerEvent::StoryChanged);
    state.trigger_save();
    Ok(Json(json))
}

async fn update_arc(
    State(state): State<AppState>,
    Path(id): Path<Uuid>,
    Json(body): Json<serde_json::Value>,
) -> ApiJson {
    let mut guard = state.project.lock();
    let Some(project) = guard.as_mut() else {
        return Err(ApiError::no_project());
    };

    if let Some(arc) = project.arcs.iter_mut().find(|a| a.id.0 == id) {
        if let Some(name) = body.get("name").and_then(|v| v.as_str()) {
            validation::validate_name(name, "arc name")?;
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
        let json = serde_json::to_value(&*arc).map_err(|e| ApiError::internal(e.to_string()))?;
        let _ = state.events_tx.send(ServerEvent::StoryChanged);
        state.trigger_save();
        Ok(Json(json))
    } else {
        Err(ApiError::not_found("arc not found"))
    }
}

async fn delete_arc(State(state): State<AppState>, Path(id): Path<Uuid>) -> ApiJson {
    let mut guard = state.project.lock();
    let Some(project) = guard.as_mut() else {
        return Err(ApiError::no_project());
    };

    let before = project.arcs.len();
    project.arcs.retain(|a| a.id.0 != id);
    let deleted = before != project.arcs.len();
    if deleted {
        let _ = state.events_tx.send(ServerEvent::StoryChanged);
        state.trigger_save();
    }
    Ok(Json(serde_json::json!({ "deleted": deleted })))
}

async fn arc_progression(State(state): State<AppState>) -> ApiJson {
    let guard = state.project.lock();
    match guard.as_ref() {
        Some(p) => {
            let progressions = analyze_all_arcs(p);
            json_value(&progressions)
        }
        None => Err(ApiError::no_project()),
    }
}
