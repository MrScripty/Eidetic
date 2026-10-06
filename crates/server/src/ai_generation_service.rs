use eidetic_core::ai::prompt::build_generate_request;
use eidetic_core::timeline::node::NodeId;
use serde::{Deserialize, Serialize};
use uuid::Uuid;

use crate::ai_generation_runtime::{mark_node_generating, run_generation};
use crate::ai_service::{
    active_sqlite_project, attach_ai_generation_context, attach_ai_generation_context_at_story_time,
};
use crate::backend_error::BackendError;
use crate::state::{AppState, ServerEvent};

#[derive(Debug, Clone, Deserialize)]
pub struct AiGenerateRequest {
    pub node_id: Uuid,
    #[serde(default)]
    pub story_time_ms: Option<u64>,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
pub struct AiGenerateResponse {
    pub status: String,
    pub node_id: String,
}

#[derive(Debug, Clone, Deserialize)]
pub struct AiGenerateBatchRequest {
    pub parent_node_id: Uuid,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
pub struct AiGenerateBatchResponse {
    pub status: String,
    pub parent_node_id: String,
    pub child_count: usize,
}

pub async fn start_generation(
    state: &AppState,
    body: AiGenerateRequest,
) -> Result<AiGenerateResponse, BackendError> {
    // Capture before any project-snapshot await. A replacement during admission
    // may conservatively disable retrieval, never rebind an old request.
    let retrieval_scope = state.vector_store.lock().scope();
    let session_id = *state.project_session_id.lock();
    let node_id = NodeId(body.node_id);
    let (mut request, project_path) = {
        let (project, project_path) = active_sqlite_project(state).await?;
        let node = project
            .timeline
            .node(node_id)
            .map_err(|_| BackendError::not_found(format!("node not found: {}", body.node_id)))?;

        if node.locked {
            return Err(BackendError::bad_request("node is locked"));
        }
        if node.content.notes.trim().is_empty() {
            return Err(BackendError::bad_request("node has no notes"));
        }
        if state.generating.lock().contains(&body.node_id) {
            return Err(BackendError::conflict("generation already in progress"));
        }

        let request = build_generate_request(&project, node_id)
            .map_err(|error| BackendError::bad_request(error.to_string()))?;
        (request, project_path)
    };
    attach_ai_generation_context_at_story_time(
        &mut request,
        project_path.clone(),
        node_id,
        body.story_time_ms,
    )
    .await?;

    admit_generation(state, &project_path, &request, session_id, body.node_id).await?;
    mark_node_generating(state, project_path.clone(), node_id, body.node_id).await;

    let state_clone = state.clone();
    let node_uuid = body.node_id;
    state.task_supervisor.spawn("ai-generation", async move {
        run_generation(
            state_clone,
            project_path,
            node_uuid,
            request,
            retrieval_scope,
            session_id,
        )
        .await;
    });

    Ok(AiGenerateResponse {
        status: "started".to_string(),
        node_id: body.node_id.to_string(),
    })
}

pub async fn start_generation_batch(
    state: &AppState,
    body: AiGenerateBatchRequest,
) -> Result<AiGenerateBatchResponse, BackendError> {
    let retrieval_scope = state.vector_store.lock().scope();
    let session_id = *state.project_session_id.lock();
    let parent_id = NodeId(body.parent_node_id);
    let child_ids: Vec<Uuid> = {
        let (project, _) = active_sqlite_project(state).await?;
        project
            .timeline
            .children_of(parent_id)
            .iter()
            .map(|node| node.id.0)
            .collect()
    };

    if child_ids.is_empty() {
        return Err(BackendError::bad_request("no children found for this node"));
    }

    let child_count = child_ids.len();
    let state_clone = state.clone();
    state
        .task_supervisor
        .spawn("ai-generation-batch", async move {
            for child_uuid in &child_ids {
                generate_child_in_batch(
                    state_clone.clone(),
                    *child_uuid,
                    retrieval_scope,
                    session_id,
                )
                .await;
            }
        });

    Ok(AiGenerateBatchResponse {
        status: "started".to_string(),
        parent_node_id: body.parent_node_id.to_string(),
        child_count,
    })
}

async fn generate_child_in_batch(
    state: AppState,
    child_uuid: Uuid,
    retrieval_scope: Uuid,
    session_id: Uuid,
) {
    let child_id = NodeId(child_uuid);
    let (mut request, project_path) = {
        let (project, project_path) = match active_sqlite_project(&state).await {
            Ok(project) => project,
            Err(error) => {
                let _ = state.events_tx.send(ServerEvent::GenerationError {
                    node_id: child_uuid,
                    error: error.message().to_string(),
                });
                return;
            }
        };

        let node = match project.timeline.node(child_id) {
            Ok(node) => node,
            Err(_) => return,
        };

        if node.locked {
            return;
        }

        let request = match build_generate_request(&project, child_id) {
            Ok(request) => request,
            Err(error) => {
                tracing::error!("Failed to build request for child node {child_uuid}: {error}");
                return;
            }
        };
        (request, project_path)
    };
    if let Err(error) =
        attach_ai_generation_context(&mut request, project_path.clone(), child_id).await
    {
        let _ = state.events_tx.send(ServerEvent::GenerationError {
            node_id: child_uuid,
            error: error.message().to_string(),
        });
        return;
    }

    if let Err(error) =
        admit_generation(&state, &project_path, &request, session_id, child_uuid).await
    {
        let _ = state.events_tx.send(ServerEvent::GenerationError {
            node_id: child_uuid,
            error: error.to_string(),
        });
        return;
    }
    mark_node_generating(&state, project_path.clone(), child_id, child_uuid).await;
    run_generation(
        state,
        project_path,
        child_uuid,
        request,
        retrieval_scope,
        session_id,
    )
    .await;
}

async fn admit_generation(
    state: &AppState,
    path: &std::path::Path,
    request: &eidetic_core::ai::backend::GenerateRequest,
    session_id: Uuid,
    node_uuid: Uuid,
) -> Result<(), BackendError> {
    let _session = state.project_session_gate.clone().lock_owned().await;
    if state.project_database.active_path().as_deref() != Some(path)
        || *state.project_session_id.lock() != session_id
    {
        return Err(BackendError::conflict("generation project session changed"));
    }
    let binding = request
        .generation_target
        .clone()
        .ok_or_else(|| BackendError::internal("generation target custody missing"))?;
    let path = path.to_path_buf();
    tokio::task::spawn_blocking(move || {
        let conn = crate::sqlite::open_write_connection(&path)
            .map_err(|error| BackendError::internal(error.to_string()))?;
        crate::script_generation_target::validate_admission(&conn, &binding)
            .map_err(|error| BackendError::conflict(error.to_string()))
    })
    .await
    .map_err(|error| BackendError::internal(error.to_string()))??;
    if !state.generating.lock().insert(node_uuid) {
        return Err(BackendError::conflict("generation already in progress"));
    }
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::{AiGenerateRequest, start_generation};
    use crate::state::AppState;
    use uuid::Uuid;

    #[tokio::test]
    async fn start_generation_requires_loaded_project() {
        let state = AppState::new().await;

        let error = start_generation(
            &state,
            AiGenerateRequest {
                node_id: Uuid::new_v4(),
                story_time_ms: None,
            },
        )
        .await
        .expect_err("missing project");

        assert_eq!(error.message(), "no project loaded");
    }
}
