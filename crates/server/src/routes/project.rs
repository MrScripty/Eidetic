use axum::extract::State;
use axum::routing::{get, post, put};
use axum::{Json, Router};
use serde::Deserialize;

use eidetic_core::Template;

use crate::persistence;
use crate::state::{AppState, ServerEvent};

pub fn router() -> Router<AppState> {
    Router::new()
        .route("/project", post(create_project))
        .route("/project", get(get_project))
        .route("/project", put(update_project))
        .route("/project/save", post(save_project))
        .route("/project/load", post(load_project))
        .route("/project/list", get(list_projects))
        .route("/project/undo", post(undo))
        .route("/project/redo", post(redo))
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
    let save_path = persistence::project_save_path(&project.name);
    let json = serde_json::to_value(&project).unwrap();
    *state.project.lock() = Some(project);
    *state.project_path.lock() = Some(save_path);
    state.trigger_save();
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
            let json = serde_json::to_value(&*project).unwrap();
            drop(guard);
            state.trigger_save();
            Json(json)
        }
        None => Json(serde_json::json!({ "error": "no project loaded" })),
    }
}

#[derive(Deserialize)]
struct SaveRequest {
    path: Option<String>,
}

async fn save_project(
    State(state): State<AppState>,
    Json(body): Json<SaveRequest>,
) -> Json<serde_json::Value> {
    let project = state.project.lock().clone();
    let Some(project) = project else {
        return Json(serde_json::json!({ "error": "no project loaded" }));
    };

    let path = body
        .path
        .map(std::path::PathBuf::from)
        .unwrap_or_else(|| {
            state
                .project_path
                .lock()
                .clone()
                .unwrap_or_else(|| persistence::project_save_path(&project.name))
        });

    match persistence::save_project(&project, &path).await {
        Ok(()) => {
            *state.project_path.lock() = Some(path.clone());
            Json(serde_json::json!({ "saved": path.display().to_string() }))
        }
        Err(e) => Json(serde_json::json!({ "error": e })),
    }
}

#[derive(Deserialize)]
struct LoadRequest {
    path: String,
}

async fn load_project(
    State(state): State<AppState>,
    Json(body): Json<LoadRequest>,
) -> Json<serde_json::Value> {
    let path = std::path::PathBuf::from(&body.path);

    match persistence::load_project(&path).await {
        Ok(project) => {
            let json = serde_json::to_value(&project).unwrap();
            *state.project.lock() = Some(project);
            *state.project_path.lock() = Some(path);
            Json(json)
        }
        Err(e) => Json(serde_json::json!({ "error": e })),
    }
}

async fn list_projects() -> Json<serde_json::Value> {
    let base_dir = persistence::default_project_dir();
    let entries = persistence::list_projects(&base_dir).await;
    Json(serde_json::to_value(&entries).unwrap())
}

async fn undo(State(state): State<AppState>) -> Json<serde_json::Value> {
    let mut undo_guard = state.undo_stack.lock();
    let mut project_guard = state.project.lock();
    let Some(current) = project_guard.as_ref() else {
        return Json(serde_json::json!({ "error": "no project loaded" }));
    };

    match undo_guard.undo(current.clone()) {
        Some(prev) => {
            let json = serde_json::to_value(&prev).unwrap();
            *project_guard = Some(prev);
            let can_undo = undo_guard.can_undo();
            let can_redo = undo_guard.can_redo();
            drop(undo_guard);
            drop(project_guard);
            let _ = state.events_tx.send(ServerEvent::TimelineChanged);
            let _ = state.events_tx.send(ServerEvent::ScenesChanged);
            let _ = state.events_tx.send(ServerEvent::StoryChanged);
            let _ = state.events_tx.send(ServerEvent::UndoRedoChanged {
                can_undo,
                can_redo,
            });
            state.trigger_save();
            Json(json)
        }
        None => Json(serde_json::json!({ "error": "nothing to undo" })),
    }
}

async fn redo(State(state): State<AppState>) -> Json<serde_json::Value> {
    let mut undo_guard = state.undo_stack.lock();
    let mut project_guard = state.project.lock();
    let Some(current) = project_guard.as_ref() else {
        return Json(serde_json::json!({ "error": "no project loaded" }));
    };

    match undo_guard.redo(current.clone()) {
        Some(next) => {
            let json = serde_json::to_value(&next).unwrap();
            *project_guard = Some(next);
            let can_undo = undo_guard.can_undo();
            let can_redo = undo_guard.can_redo();
            drop(undo_guard);
            drop(project_guard);
            let _ = state.events_tx.send(ServerEvent::TimelineChanged);
            let _ = state.events_tx.send(ServerEvent::ScenesChanged);
            let _ = state.events_tx.send(ServerEvent::StoryChanged);
            let _ = state.events_tx.send(ServerEvent::UndoRedoChanged {
                can_undo,
                can_redo,
            });
            state.trigger_save();
            Json(json)
        }
        None => Json(serde_json::json!({ "error": "nothing to redo" })),
    }
}
