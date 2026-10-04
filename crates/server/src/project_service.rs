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
    let session = state.project_session_gate.clone().lock_owned().await;
    let worker = state.clone();
    crate::state::complete_project_session_work(state, session, async move {
        let state = &worker;

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
        let json =
            serde_json::to_value(&project).map_err(|e| BackendError::internal(e.to_string()))?;
        persist_active_project(state).await?;
        populate_ydoc_from_project(state, &project).await?;
        replace_active_project(state, project, save_path);
        state.trigger_save();
        Ok(json)
    })
    .await
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
    let session = state.project_session_gate.clone().lock_owned().await;
    let worker = state.clone();
    crate::state::complete_project_session_work(state, session, async move {
        let state = &worker;
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

        let source_identity = state.project_database.active_path_identity(&project_root)?;
        let same_destination = source_identity.as_ref() == Some(&path);
        if !same_destination
            && path.try_exists().map_err(|error| {
                BackendError::internal(format!("cannot inspect Save As destination: {error}"))
            })?
        {
            return Err(BackendError::conflict("Save As destination already exists"));
        }
        let (project, source_path, ydoc_state) = persist_active_project(state)
            .await?
            .ok_or_else(BackendError::no_project)?;
        if path != source_path {
            persistence::save_project(&project, &path, Some(ydoc_state))
                .await
                .map_err(BackendError::internal)?;
        }

        // Refresh only SQLite-owned data: metadata/reference producers may have
        // updated the mirror while persistence was awaiting its blocking worker.
        let mut active = state.project.lock();
        if let Some(active) = active.as_mut() {
            active.timeline = project.timeline;
            active.arcs = project.arcs;
        }
        if !same_destination {
            *state.project_session_id.lock() = uuid::Uuid::new_v4();
        }
        state.project_database.set_active_path(path.clone());
        Ok(serde_json::json!({ "saved": path.display().to_string() }))
    })
    .await
}

pub async fn load_project(
    state: &AppState,
    request: LoadProjectRequest,
) -> Result<serde_json::Value, BackendError> {
    let project_root = persistence::default_project_dir();
    let path = validation::validate_project_path(&request.path, &project_root)?;

    let session = state.project_session_gate.clone().lock_owned().await;
    let worker = state.clone();
    crate::state::complete_project_session_work(state, session, async move {
        let state = &worker;
        persist_active_project(state).await?;
        let (project, ydoc_state) = persistence::load_project(&path)
            .await
            .map_err(BackendError::bad_request)?;
        let json =
            serde_json::to_value(&project).map_err(|e| BackendError::internal(e.to_string()))?;

        if let Some(blob) = ydoc_state {
            if let Err(error) = crate::ydoc::load_doc(&state.doc_tx, blob).await {
                tracing::warn!("failed to load Y.Doc state, populating from project: {error}");
                populate_ydoc_from_project(state, &project).await?;
            }
        } else {
            populate_ydoc_from_project(state, &project).await?;
        }

        let save_path = if path
            .extension()
            .is_some_and(|extension| extension == "json")
        {
            path.with_file_name("project.db")
        } else {
            path
        };
        replace_active_project(state, project, save_path);
        state.trigger_save();
        Ok(json)
    })
    .await
}

/// Flush directly rather than waiting for a coalesced/debounced save signal.
/// The caller owns the session gate. Serialization/read/write failures propagate
/// before any document replacement or active-path publication.
async fn persist_active_project(
    state: &AppState,
) -> Result<Option<(eidetic_core::Project, std::path::PathBuf, Vec<u8>)>, BackendError> {
    let snapshot = {
        let active = state.project.lock();
        active.as_ref().map(|project| {
            let path = state
                .project_database
                .active_path()
                .unwrap_or_else(|| persistence::project_save_path(&project.name));
            (project.clone(), path)
        })
    };
    let Some((project, path)) = snapshot else {
        return Ok(None);
    };
    let path = validation::validate_project_path(
        path.to_string_lossy().as_ref(),
        &persistence::default_project_dir(),
    )?;
    let ydoc_state = crate::ydoc::serialize_doc(&state.doc_tx)
        .await
        .ok_or_else(|| BackendError::internal("active project document serialization failed"))?;
    // Same-database save preserves committed timeline/arcs over the stale mirror.
    persistence::save_project(&project, &path, Some(ydoc_state.clone()))
        .await
        .map_err(BackendError::internal)?;
    let (authoritative, _) = persistence::load_project(&path)
        .await
        .map_err(BackendError::internal)?;
    Ok(Some((authoritative, path, ydoc_state)))
}

/// Publish the project mirror, derived-index lifetime and database identity under
/// the same project guard used by reference publication and retrieval.
/// Production transitions hold `project_session_gate` across document replacement
/// and this publication; direct calls also support synchronous test setup.
pub(crate) fn replace_active_project(
    state: &AppState,
    project: eidetic_core::Project,
    path: std::path::PathBuf,
) {
    let mut active = state.project.lock();
    state.vector_store.lock().reset();
    *state.project_session_id.lock() = uuid::Uuid::new_v4();
    state.project_database.set_active_path(path);
    *active = Some(project);
}

pub async fn list_projects() -> serde_json::Value {
    let base_dir = persistence::default_project_dir();
    let entries = persistence::list_projects(&base_dir).await;
    serde_json::to_value(&entries).unwrap_or_else(|_| serde_json::json!([]))
}

async fn populate_ydoc_from_project(
    state: &AppState,
    project: &eidetic_core::Project,
) -> Result<(), BackendError> {
    crate::ydoc::load_doc(&state.doc_tx, Vec::new())
        .await
        .map_err(BackendError::internal)?;
    for node in &project.timeline.nodes {
        state
            .doc_tx
            .send(DocCommand::EnsureNode { node_id: node.id })
            .await
            .map_err(|_| BackendError::internal("doc manager channel closed"))?;

        if !node.content.notes.is_empty() {
            state
                .doc_tx
                .send(DocCommand::WriteNodeContent {
                    node_id: node.id,
                    field: ContentField::Notes,
                    text: node.content.notes.clone(),
                    author: "system:load".into(),
                })
                .await
                .map_err(|_| BackendError::internal("doc manager channel closed"))?;
        }

        if !node.content.content.is_empty() {
            state
                .doc_tx
                .send(DocCommand::WriteNodeContent {
                    node_id: node.id,
                    field: ContentField::Content,
                    text: node.content.content.clone(),
                    author: "system:load".into(),
                })
                .await
                .map_err(|_| BackendError::internal("doc manager channel closed"))?;
        }
    }
    Ok(())
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
