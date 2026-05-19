use axum::extract::State;
use axum::routing::{get, post, put};
use axum::{Json, Router};
use serde::Deserialize;

use eidetic_core::Template;

use crate::error::ApiError;
use crate::persistence;
use crate::state::AppState;
use crate::validation;
use crate::ydoc::{ContentField, DocCommand};

pub fn router() -> Router<AppState> {
    Router::new()
        .route("/project", post(create_project))
        .route("/project", get(get_project))
        .route("/project", put(update_project))
        .route("/project/save", post(save_project))
        .route("/project/load", post(load_project))
        .route("/project/list", get(list_projects))
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
) -> Result<Json<serde_json::Value>, ApiError> {
    validation::validate_name(&body.name, "project name")?;

    let template = match body.template.as_str() {
        "single_cam" => Template::SingleCam,
        "animated" => Template::Animated,
        _ => Template::MultiCam,
    };

    let project = template.build_project(body.name);
    let project_root = persistence::default_project_dir();
    let save_path = validation::validate_project_path(
        persistence::project_save_path(&project.name)
            .to_string_lossy()
            .as_ref(),
        &project_root,
    )?;
    let json = serde_json::to_value(&project).map_err(|e| ApiError::internal(e.to_string()))?;
    // Populate Y.Doc with all node text from the new project.
    populate_ydoc_from_project(&state, &project).await;
    *state.project.lock() = Some(project);
    state.project_database.set_active_path(save_path);
    state.trigger_save();
    Ok(Json(json))
}

async fn get_project(State(state): State<AppState>) -> Result<Json<serde_json::Value>, ApiError> {
    let guard = state.project.lock();
    match guard.as_ref() {
        Some(project) => {
            let json =
                serde_json::to_value(project).map_err(|e| ApiError::internal(e.to_string()))?;
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
) -> Result<Json<serde_json::Value>, ApiError> {
    let mut guard = state.project.lock();
    match guard.as_mut() {
        Some(project) => {
            if let Some(name) = body.name {
                validation::validate_name(&name, "project name")?;
                project.name = name;
            }
            if let Some(premise) = body.premise {
                project.premise = premise;
            }
            let json =
                serde_json::to_value(&*project).map_err(|e| ApiError::internal(e.to_string()))?;
            drop(guard);
            state.trigger_save();
            Ok(Json(json))
        }
        None => Err(ApiError::no_project()),
    }
}

#[derive(Deserialize)]
struct SaveRequest {
    path: Option<String>,
}

async fn save_project(
    State(state): State<AppState>,
    Json(body): Json<SaveRequest>,
) -> Result<Json<serde_json::Value>, ApiError> {
    let project = state.project.lock().clone();
    let Some(project) = project else {
        return Err(ApiError::no_project());
    };

    let project_root = persistence::default_project_dir();
    let requested_path = body.path.unwrap_or_else(|| {
        state
            .project_database
            .active_path()
            .unwrap_or_else(|| persistence::project_save_path(&project.name))
            .display()
            .to_string()
    });
    let path = validation::validate_project_path(&requested_path, &project_root)?;

    // Serialize Y.Doc state to persist alongside structural data.
    let ydoc_state = crate::ydoc::serialize_doc(&state.doc_tx).await;

    match persistence::save_project(&project, &path, ydoc_state).await {
        Ok(()) => {
            state.project_database.set_active_path(path.clone());
            Ok(Json(
                serde_json::json!({ "saved": path.display().to_string() }),
            ))
        }
        Err(e) => Err(ApiError::internal(e)),
    }
}

#[derive(Deserialize)]
struct LoadRequest {
    path: String,
}

async fn load_project(
    State(state): State<AppState>,
    Json(body): Json<LoadRequest>,
) -> Result<Json<serde_json::Value>, ApiError> {
    let project_root = persistence::default_project_dir();
    let path = validation::validate_project_path(&body.path, &project_root)?;

    match persistence::load_project(&path).await {
        Ok((project, ydoc_state)) => {
            let json =
                serde_json::to_value(&project).map_err(|e| ApiError::internal(e.to_string()))?;

            // Restore Y.Doc: prefer persisted blob, fall back to populating
            // from the project's cached text fields.
            if let Some(blob) = ydoc_state {
                if let Err(e) = crate::ydoc::load_doc(&state.doc_tx, blob).await {
                    tracing::warn!("failed to load Y.Doc state, populating from project: {e}");
                    populate_ydoc_from_project(&state, &project).await;
                }
            } else {
                populate_ydoc_from_project(&state, &project).await;
            }

            *state.project.lock() = Some(project);
            // Normalize save path to .db so future auto-saves write SQLite.
            let save_path = if path.extension().map_or(false, |ext| ext == "json") {
                path.with_file_name("project.db")
            } else {
                path
            };
            state.project_database.set_active_path(save_path);
            state.trigger_save();
            Ok(Json(json))
        }
        Err(e) => Err(ApiError::bad_request(e)),
    }
}

async fn list_projects() -> Json<serde_json::Value> {
    let base_dir = persistence::default_project_dir();
    let entries = persistence::list_projects(&base_dir).await;
    Json(serde_json::to_value(&entries).unwrap())
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

#[cfg(test)]
mod tests {
    use super::router;
    use crate::state::AppState;
    use axum::body::Body;
    use axum::http::{Request, StatusCode};
    use tower::util::ServiceExt;

    #[tokio::test]
    async fn create_project_rejects_invalid_name() {
        let app = router().with_state(AppState::new().await);

        let response = app
            .oneshot(
                Request::builder()
                    .method("POST")
                    .uri("/project")
                    .header("content-type", "application/json")
                    .body(Body::from(r#"{"name":"bad/name","template":"multi_cam"}"#))
                    .unwrap(),
            )
            .await
            .unwrap();

        assert_eq!(response.status(), StatusCode::BAD_REQUEST);
    }

    #[tokio::test]
    async fn save_project_without_loaded_project_returns_not_found() {
        let app = router().with_state(AppState::new().await);

        let response = app
            .oneshot(
                Request::builder()
                    .method("POST")
                    .uri("/project/save")
                    .header("content-type", "application/json")
                    .body(Body::from(r#"{}"#))
                    .unwrap(),
            )
            .await
            .unwrap();

        assert_eq!(response.status(), StatusCode::NOT_FOUND);
    }

    #[tokio::test]
    async fn load_project_rejects_path_outside_root() {
        let app = router().with_state(AppState::new().await);

        let response = app
            .oneshot(
                Request::builder()
                    .method("POST")
                    .uri("/project/load")
                    .header("content-type", "application/json")
                    .body(Body::from(r#"{"path":"/tmp/outside/project.db"}"#))
                    .unwrap(),
            )
            .await
            .unwrap();

        assert_eq!(response.status(), StatusCode::BAD_REQUEST);
    }
}
