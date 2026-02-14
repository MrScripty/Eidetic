use axum::extract::State;
use axum::routing::{get, post, put};
use axum::{Json, Router};
use serde::Deserialize;

use eidetic_core::Template;

use crate::state::AppState;

pub fn router() -> Router<AppState> {
    Router::new()
        .route("/project", post(create_project))
        .route("/project", get(get_project))
        .route("/project", put(update_project))
}

#[derive(Deserialize)]
pub struct CreateProjectRequest {
    pub name: String,
    /// "multi_cam", "single_cam", or "animated".
    pub template: String,
}

async fn create_project(
    State(state): State<AppState>,
    Json(body): Json<CreateProjectRequest>,
) -> Json<serde_json::Value> {
    let template = match body.template.as_str() {
        "single_cam" => Template::SingleCam,
        "animated" => Template::Animated,
        _ => Template::MultiCam,
    };

    let project = template.build_project(body.name);
    let json = serde_json::to_value(&project).unwrap();
    *state.project.lock() = Some(project);
    Json(json)
}

async fn get_project(State(state): State<AppState>) -> Json<serde_json::Value> {
    let guard = state.project.lock();
    match guard.as_ref() {
        Some(project) => Json(serde_json::to_value(project).unwrap()),
        None => Json(serde_json::json!({ "error": "no project loaded" })),
    }
}

#[derive(Deserialize)]
pub struct UpdateProjectRequest {
    pub name: Option<String>,
}

async fn update_project(
    State(state): State<AppState>,
    Json(body): Json<UpdateProjectRequest>,
) -> Json<serde_json::Value> {
    let mut guard = state.project.lock();
    match guard.as_mut() {
        Some(project) => {
            if let Some(name) = body.name {
                project.name = name;
            }
            Json(serde_json::to_value(&*project).unwrap())
        }
        None => Json(serde_json::json!({ "error": "no project loaded" })),
    }
}
