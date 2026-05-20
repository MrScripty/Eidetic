use serde::Deserialize;

use eidetic_core::Template;

use crate::backend_error::BackendError;
use crate::persistence;
use crate::state::AppState;
use crate::validation;
use crate::ydoc::{ContentField, DocCommand};

#[derive(Deserialize)]
pub struct CreateProjectRequest {
    pub name: String,
    /// "multi_cam", "single_cam", or "animated".
    pub template: String,
}

#[derive(Deserialize)]
pub struct UpdateProjectRequest {
    pub name: Option<String>,
    pub premise: Option<String>,
}

#[derive(Deserialize)]
pub struct SaveProjectRequest {
    pub path: Option<String>,
}

#[derive(Deserialize)]
pub struct LoadProjectRequest {
    pub path: String,
}

pub async fn create_project(
    state: &AppState,
    request: CreateProjectRequest,
) -> Result<serde_json::Value, BackendError> {
    validation::validate_name(&request.name, "project name")?;

    let template = match request.template.as_str() {
        "single_cam" => Template::SingleCam,
        "animated" => Template::Animated,
        _ => Template::MultiCam,
    };

    let project = template.build_project(request.name);
    let project_root = persistence::default_project_dir();
    let save_path = validation::validate_project_path(
        persistence::project_save_path(&project.name)
            .to_string_lossy()
            .as_ref(),
        &project_root,
    )?;
    let json = serde_json::to_value(&project).map_err(|e| BackendError::internal(e.to_string()))?;
    populate_ydoc_from_project(state, &project).await;
    *state.project.lock() = Some(project);
    state.project_database.set_active_path(save_path);
    state.trigger_save();
    Ok(json)
}

pub fn get_project(state: &AppState) -> Result<serde_json::Value, BackendError> {
    let guard = state.project.lock();
    let Some(project) = guard.as_ref() else {
        return Err(BackendError::no_project());
    };

    serde_json::to_value(project).map_err(|e| BackendError::internal(e.to_string()))
}

pub fn update_project(
    state: &AppState,
    request: UpdateProjectRequest,
) -> Result<serde_json::Value, BackendError> {
    let mut guard = state.project.lock();
    let Some(project) = guard.as_mut() else {
        return Err(BackendError::no_project());
    };

    if let Some(name) = request.name {
        validation::validate_name(&name, "project name")?;
        project.name = name;
    }
    if let Some(premise) = request.premise {
        project.premise = premise;
    }
    let json =
        serde_json::to_value(&*project).map_err(|e| BackendError::internal(e.to_string()))?;
    drop(guard);
    state.trigger_save();
    Ok(json)
}

pub async fn save_project(
    state: &AppState,
    request: SaveProjectRequest,
) -> Result<serde_json::Value, BackendError> {
    let project = state.project.lock().clone();
    let Some(project) = project else {
        return Err(BackendError::no_project());
    };

    let project_root = persistence::default_project_dir();
    let requested_path = request.path.unwrap_or_else(|| {
        state
            .project_database
            .active_path()
            .unwrap_or_else(|| persistence::project_save_path(&project.name))
            .display()
            .to_string()
    });
    let path = validation::validate_project_path(&requested_path, &project_root)?;

    let ydoc_state = crate::ydoc::serialize_doc(&state.doc_tx).await;
    persistence::save_project(&project, &path, ydoc_state)
        .await
        .map_err(BackendError::internal)?;

    state.project_database.set_active_path(path.clone());
    Ok(serde_json::json!({ "saved": path.display().to_string() }))
}

pub async fn load_project(
    state: &AppState,
    request: LoadProjectRequest,
) -> Result<serde_json::Value, BackendError> {
    let project_root = persistence::default_project_dir();
    let path = validation::validate_project_path(&request.path, &project_root)?;

    let (project, ydoc_state) = persistence::load_project(&path)
        .await
        .map_err(BackendError::bad_request)?;
    let json = serde_json::to_value(&project).map_err(|e| BackendError::internal(e.to_string()))?;

    if let Some(blob) = ydoc_state {
        if let Err(error) = crate::ydoc::load_doc(&state.doc_tx, blob).await {
            tracing::warn!("failed to load Y.Doc state, populating from project: {error}");
            populate_ydoc_from_project(state, &project).await;
        }
    } else {
        populate_ydoc_from_project(state, &project).await;
    }

    *state.project.lock() = Some(project);
    let save_path = if path
        .extension()
        .is_some_and(|extension| extension == "json")
    {
        path.with_file_name("project.db")
    } else {
        path
    };
    state.project_database.set_active_path(save_path);
    state.trigger_save();
    Ok(json)
}

pub async fn list_projects() -> serde_json::Value {
    let base_dir = persistence::default_project_dir();
    let entries = persistence::list_projects(&base_dir).await;
    serde_json::to_value(&entries).unwrap_or_else(|_| serde_json::json!([]))
}

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
    use super::{CreateProjectRequest, create_project};
    use crate::state::AppState;

    #[tokio::test]
    async fn create_project_rejects_invalid_name_without_http_boundary() {
        let state = AppState::new().await;
        let error = create_project(
            &state,
            CreateProjectRequest {
                name: "bad/name".into(),
                template: "multi_cam".into(),
            },
        )
        .await
        .unwrap_err();

        assert_eq!(
            error.message(),
            "project name contains unsupported characters"
        );
    }
}
