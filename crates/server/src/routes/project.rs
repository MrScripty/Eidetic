use axum::extract::State;
use axum::routing::{get, post, put};
use axum::{Json, Router};
use serde::Deserialize;

use eidetic_core::Template;

use crate::error::ApiError;
use crate::persistence;
use crate::state::{AppState, ServerEvent};
use crate::ydoc::{ContentField, DocCommand};

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
    // Populate Y.Doc with all node text from the new project.
    populate_ydoc_from_project(&state, &project).await;
    *state.project.lock() = Some(project);
    *state.project_path.lock() = Some(save_path);
    state.trigger_save();
    Json(json)
}

async fn get_project(
    State(state): State<AppState>,
) -> Result<Json<serde_json::Value>, ApiError> {
    let guard = state.project.lock();
    match guard.as_ref() {
        Some(project) => {
            let json = serde_json::to_value(project)
                .map_err(|e| ApiError::internal(e.to_string()))?;
            Ok(Json(json))
        }
        None => Err(ApiError::no_project()),
    }
}

#[derive(Deserialize)]
pub struct UpdateProjectRequest {
    pub name: Option<String>,
    pub premise: Option<String>,
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
            if let Some(premise) = body.premise {
                project.premise = premise;
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
            // Populate Y.Doc with all node text from the loaded project.
            populate_ydoc_from_project(&state, &project).await;
            *state.project.lock() = Some(project);
            // Normalize save path to .db so future auto-saves write SQLite.
            let save_path = if path.extension().map_or(false, |ext| ext == "json") {
                path.with_file_name("project.db")
            } else {
                path
            };
            *state.project_path.lock() = Some(save_path);
            state.trigger_save();
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

async fn undo(
    State(state): State<AppState>,
) -> Result<Json<serde_json::Value>, ApiError> {
    let mut undo_guard = state.undo_stack.lock();
    let mut project_guard = state.project.lock();
    let Some(current) = project_guard.as_ref() else {
        return Err(ApiError::no_project());
    };

    match undo_guard.undo(current.clone()) {
        Some(prev) => {
            let json = serde_json::to_value(&prev)
                .map_err(|e| ApiError::internal(e.to_string()))?;
            *project_guard = Some(prev);
            let can_undo = undo_guard.can_undo();
            let can_redo = undo_guard.can_redo();
            drop(undo_guard);
            drop(project_guard);
            let _ = state.events_tx.send(ServerEvent::ProjectMutated);
            let _ = state.events_tx.send(ServerEvent::UndoRedoChanged {
                can_undo,
                can_redo,
            });
            state.trigger_save();
            Ok(Json(json))
        }
        None => Err(ApiError::conflict("nothing to undo")),
    }
}

async fn redo(
    State(state): State<AppState>,
) -> Result<Json<serde_json::Value>, ApiError> {
    let mut undo_guard = state.undo_stack.lock();
    let mut project_guard = state.project.lock();
    let Some(current) = project_guard.as_ref() else {
        return Err(ApiError::no_project());
    };

    match undo_guard.redo(current.clone()) {
        Some(next) => {
            let json = serde_json::to_value(&next)
                .map_err(|e| ApiError::internal(e.to_string()))?;
            *project_guard = Some(next);
            let can_undo = undo_guard.can_undo();
            let can_redo = undo_guard.can_redo();
            drop(undo_guard);
            drop(project_guard);
            let _ = state.events_tx.send(ServerEvent::ProjectMutated);
            let _ = state.events_tx.send(ServerEvent::UndoRedoChanged {
                can_undo,
                can_redo,
            });
            state.trigger_save();
            Ok(Json(json))
        }
        None => Err(ApiError::conflict("nothing to redo")),
    }
}

/// Populate the Y.Doc with all node text content from a project.
///
/// Called on project create/load to initialize the CRDT layer from
/// the project's cached text fields.
async fn populate_ydoc_from_project(state: &AppState, project: &eidetic_core::Project) {
    for node in &project.timeline.nodes {
        let _ = state
            .doc_tx
            .send(DocCommand::EnsureNode { node_id: node.id })
            .await;

        if !node.content.notes.is_empty() {
            let _ = state
                .doc_tx
                .send(DocCommand::WriteNodeContent {
                    node_id: node.id,
                    field: ContentField::Notes,
                    text: node.content.notes.clone(),
                    author: "system:load".into(),
                })
                .await;
        }

        if !node.content.content.is_empty() {
            let _ = state
                .doc_tx
                .send(DocCommand::WriteNodeContent {
                    node_id: node.id,
                    field: ContentField::Content,
                    text: node.content.content.clone(),
                    author: "system:load".into(),
                })
                .await;
        }
    }
}
