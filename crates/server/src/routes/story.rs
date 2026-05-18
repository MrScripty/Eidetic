use axum::extract::{Path, State};
use axum::routing::{delete, get, post, put};
use axum::{Json, Router};
use serde::Deserialize;
use uuid::Uuid;

use eidetic_core::contracts::{
    CommandEnvelope, CreateStoryArcCommand, DeleteStoryArcCommand, SetStoryArcMetadataCommand,
    StoryArcListProjection,
};
use eidetic_core::story::arc::{ArcId, ArcType, Color, StoryArc};
use eidetic_core::story::progression::analyze_all_arcs;

use crate::error::{ApiError, ApiJson, json_value};
use crate::state::{AppState, ServerEvent};
use crate::story_arc_command::{self, StoryArcCommandError};

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
    let arc_id = ArcId::new();
    let command = CommandEnvelope::new(CreateStoryArcCommand {
        arc_id,
        parent_arc_id: None,
        name: body.name,
        description: String::new(),
        arc_type: parse_arc_type(&body.arc_type),
        color: body
            .color
            .map(|[r, g, b]| Color::new(r, g, b))
            .unwrap_or(Color::new(180, 180, 180)),
    });

    let mut guard = state.project.lock();
    let Some(project) = guard.as_mut() else {
        return Err(ApiError::no_project());
    };

    let projection = story_arc_command::apply_create_story_arc(project, &command)
        .map_err(map_story_arc_command_error)?;
    let arc = find_arc(&projection.payload, arc_id)?;
    let json = serde_json::to_value(arc).map_err(|e| ApiError::internal(e.to_string()))?;
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

    let arc_id = ArcId(id);
    let command = CommandEnvelope::new(SetStoryArcMetadataCommand {
        arc_id,
        name: body
            .get("name")
            .and_then(|value| value.as_str())
            .map(str::to_string),
        description: body
            .get("description")
            .and_then(|value| value.as_str())
            .map(str::to_string),
        arc_type: body
            .get("arc_type")
            .and_then(|value| value.as_str())
            .map(parse_arc_type),
        color: parse_color(body.get("color")),
    });
    let projection = story_arc_command::apply_set_story_arc_metadata(project, &command)
        .map_err(map_story_arc_command_error)?;
    let arc = find_arc(&projection.payload, arc_id)?;
    let json = serde_json::to_value(arc).map_err(|e| ApiError::internal(e.to_string()))?;
    let _ = state.events_tx.send(ServerEvent::StoryChanged);
    state.trigger_save();
    Ok(Json(json))
}

async fn delete_arc(State(state): State<AppState>, Path(id): Path<Uuid>) -> ApiJson {
    let mut guard = state.project.lock();
    let Some(project) = guard.as_mut() else {
        return Err(ApiError::no_project());
    };

    let command = CommandEnvelope::new(DeleteStoryArcCommand { arc_id: ArcId(id) });
    let (deleted, _projection) = story_arc_command::apply_delete_story_arc(project, &command)
        .map_err(map_story_arc_command_error)?;
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

fn parse_arc_type(value: &str) -> ArcType {
    match value {
        "a_plot" => ArcType::APlot,
        "b_plot" => ArcType::BPlot,
        "c_runner" => ArcType::CRunner,
        other => ArcType::Custom(other.to_string()),
    }
}

fn parse_color(value: Option<&serde_json::Value>) -> Option<Color> {
    let color_arr = value.and_then(|value| value.as_array())?;
    if color_arr.len() != 3 {
        return None;
    }
    let r = color_arr[0].as_u64()?;
    let g = color_arr[1].as_u64()?;
    let b = color_arr[2].as_u64()?;
    Some(Color::new(r as u8, g as u8, b as u8))
}

fn find_arc(projection: &StoryArcListProjection, arc_id: ArcId) -> Result<&StoryArc, ApiError> {
    projection
        .arcs
        .iter()
        .find(|arc| arc.id == arc_id)
        .ok_or_else(|| ApiError::internal("story arc projection did not include updated arc"))
}

fn map_story_arc_command_error(error: StoryArcCommandError) -> ApiError {
    match error {
        StoryArcCommandError::InvalidCommand(message) => ApiError::bad_request(message),
        StoryArcCommandError::NotFound(message) => ApiError::not_found(message),
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::path::PathBuf;

    use axum::body::{Body, to_bytes};
    use axum::http::{Request, StatusCode};
    use eidetic_core::Template;
    use serde_json::json;
    use tower::util::ServiceExt;

    async fn app_with_project_path(path: PathBuf) -> Router {
        let state = AppState::new().await;
        *state.project.lock() = Some(Template::MultiCam.build_project("Story Route Test"));
        *state.project_path.lock() = Some(path);
        router().with_state(state)
    }

    async fn app_with_project_path_and_arc(path: PathBuf) -> (Router, ArcId) {
        let state = AppState::new().await;
        let project = Template::MultiCam.build_project("Story Route Test");
        let arc_id = project.arcs[0].id;
        *state.project.lock() = Some(project);
        *state.project_path.lock() = Some(path);
        (router().with_state(state), arc_id)
    }

    #[tokio::test]
    async fn create_arc_route_uses_story_arc_command() {
        let path = temp_db_path("create-arc-route");
        let app = app_with_project_path(path.clone()).await;
        let body = json!({
            "name": "Mystery",
            "arc_type": "a_plot",
            "color": [1, 2, 3],
        });

        let response = app
            .oneshot(json_request("POST", "/arcs", body))
            .await
            .expect("route response");

        assert_eq!(response.status(), StatusCode::OK);
        let value = response_json(response).await;
        assert_eq!(value["name"], "Mystery");
        assert_eq!(value["arc_type"], "APlot");
        assert_eq!(value["color"], json!({ "r": 1, "g": 2, "b": 3 }));

        let _ = std::fs::remove_file(path);
    }

    #[tokio::test]
    async fn update_arc_route_uses_story_arc_command() {
        let path = temp_db_path("update-arc-route");
        let (app, arc_id) = app_with_project_path_and_arc(path.clone()).await;
        let body = json!({
            "name": "Renamed",
            "description": "Updated description",
            "arc_type": "b_plot",
            "color": [4, 5, 6],
        });

        let response = app
            .oneshot(json_request("PUT", &format!("/arcs/{}", arc_id.0), body))
            .await
            .expect("route response");

        assert_eq!(response.status(), StatusCode::OK);
        let value = response_json(response).await;
        assert_eq!(value["name"], "Renamed");
        assert_eq!(value["description"], "Updated description");
        assert_eq!(value["arc_type"], "BPlot");
        assert_eq!(value["color"], json!({ "r": 4, "g": 5, "b": 6 }));

        let _ = std::fs::remove_file(path);
    }

    #[tokio::test]
    async fn delete_arc_route_uses_story_arc_command() {
        let path = temp_db_path("delete-arc-route");
        let (app, arc_id) = app_with_project_path_and_arc(path.clone()).await;

        let response = app
            .oneshot(empty_request("DELETE", &format!("/arcs/{}", arc_id.0)))
            .await
            .expect("route response");

        assert_eq!(response.status(), StatusCode::OK);
        let value = response_json(response).await;
        assert_eq!(value["deleted"], true);

        let _ = std::fs::remove_file(path);
    }

    fn json_request(method: &str, uri: &str, body: serde_json::Value) -> Request<Body> {
        Request::builder()
            .method(method)
            .uri(uri)
            .header("content-type", "application/json")
            .body(Body::from(body.to_string()))
            .unwrap()
    }

    fn empty_request(method: &str, uri: &str) -> Request<Body> {
        Request::builder()
            .method(method)
            .uri(uri)
            .body(Body::empty())
            .unwrap()
    }

    async fn response_json(response: axum::response::Response) -> serde_json::Value {
        let body = to_bytes(response.into_body(), 1024 * 1024)
            .await
            .expect("body");
        serde_json::from_slice(&body).expect("json")
    }

    fn temp_db_path(label: &str) -> PathBuf {
        std::env::temp_dir().join(format!(
            "eidetic-story-route-test-{label}-{}.db",
            uuid::Uuid::new_v4()
        ))
    }
}
