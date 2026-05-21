use eidetic_core::ai::prompt::build_generate_request;
use eidetic_core::timeline::node::NodeId;
use serde::{Deserialize, Serialize};
use uuid::Uuid;

use crate::ai_generation_runtime::{mark_node_generating, run_generation};
use crate::ai_service::{active_sqlite_project, attach_ai_bible_context};
use crate::backend_error::BackendError;
use crate::state::{AppState, ServerEvent};

#[derive(Debug, Clone, Deserialize)]
pub struct AiGenerateRequest {
    pub node_id: Uuid,
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
    attach_ai_bible_context(&mut request, project_path.clone(), node_id).await?;

    state.generating.lock().insert(body.node_id);
    mark_node_generating(state, project_path.clone(), node_id, body.node_id).await;

    let state_clone = state.clone();
    let node_uuid = body.node_id;
    state.task_supervisor.spawn("ai-generation", async move {
        run_generation(state_clone, project_path, node_uuid, request).await;
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
                generate_child_in_batch(state_clone.clone(), *child_uuid).await;
            }
        });

    Ok(AiGenerateBatchResponse {
        status: "started".to_string(),
        parent_node_id: body.parent_node_id.to_string(),
        child_count,
    })
}

async fn generate_child_in_batch(state: AppState, child_uuid: Uuid) {
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
    if let Err(error) = attach_ai_bible_context(&mut request, project_path.clone(), child_id).await
    {
        let _ = state.events_tx.send(ServerEvent::GenerationError {
            node_id: child_uuid,
            error: error.message().to_string(),
        });
        return;
    }

    state.generating.lock().insert(child_uuid);
    mark_node_generating(&state, project_path.clone(), child_id, child_uuid).await;
    run_generation(state, project_path, child_uuid, request).await;
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
            },
        )
        .await
        .expect_err("missing project");

        assert_eq!(error.message(), "no project loaded");
    }
}
