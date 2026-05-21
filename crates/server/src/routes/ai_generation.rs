use axum::Json;
use axum::extract::State;
use eidetic_core::ai::prompt::build_generate_request;
use eidetic_core::timeline::node::NodeId;
use serde::Deserialize;
use uuid::Uuid;

use crate::state::{AppState, ServerEvent};

use super::ai_generation_runtime::{mark_node_generating, run_generation};
use super::{active_sqlite_project, attach_ai_bible_context};

#[derive(Deserialize)]
pub(super) struct GenerateBody {
    node_id: Uuid,
}

pub(super) async fn generate(
    State(state): State<AppState>,
    Json(body): Json<GenerateBody>,
) -> Json<serde_json::Value> {
    let node_id = NodeId(body.node_id);

    let (mut request, project_path) = {
        let (project, project_path) = match active_sqlite_project(&state).await {
            Ok(project) => project,
            Err(error) => return Json(serde_json::json!({ "error": error })),
        };

        let node = match project.timeline.node(node_id) {
            Ok(n) => n,
            Err(_) => {
                return Json(
                    serde_json::json!({ "error": format!("node not found: {}", body.node_id) }),
                );
            }
        };

        if node.locked {
            return Json(serde_json::json!({ "error": "node is locked" }));
        }

        if node.content.notes.trim().is_empty() {
            return Json(serde_json::json!({ "error": "node has no notes" }));
        }

        if state.generating.lock().contains(&body.node_id) {
            return Json(serde_json::json!({ "error": "generation already in progress" }));
        }

        let request = match build_generate_request(&project, node_id) {
            Ok(req) => req,
            Err(e) => {
                return Json(serde_json::json!({ "error": e.to_string() }));
            }
        };
        (request, project_path)
    };
    if let Err(error) = attach_ai_bible_context(&mut request, project_path.clone(), node_id).await {
        return Json(serde_json::json!({ "error": error }));
    }

    state.generating.lock().insert(body.node_id);
    mark_node_generating(&state, project_path.clone(), node_id, body.node_id).await;

    let state_clone = state.clone();
    let node_uuid = body.node_id;
    state.task_supervisor.spawn("ai-generation", async move {
        run_generation(state_clone, project_path, node_uuid, request).await;
    });

    Json(serde_json::json!({
        "status": "started",
        "node_id": body.node_id.to_string(),
    }))
}

#[derive(Deserialize)]
pub(super) struct GenerateBatchBody {
    parent_node_id: Uuid,
}

pub(super) async fn generate_batch(
    State(state): State<AppState>,
    Json(body): Json<GenerateBatchBody>,
) -> Json<serde_json::Value> {
    let parent_id = NodeId(body.parent_node_id);

    let child_ids: Vec<Uuid> = {
        let (project, _) = match active_sqlite_project(&state).await {
            Ok(project) => project,
            Err(error) => return Json(serde_json::json!({ "error": error })),
        };

        project
            .timeline
            .children_of(parent_id)
            .iter()
            .map(|n| n.id.0)
            .collect()
    };

    if child_ids.is_empty() {
        return Json(serde_json::json!({ "error": "no children found for this node" }));
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

    Json(serde_json::json!({
        "status": "started",
        "parent_node_id": body.parent_node_id.to_string(),
        "child_count": child_count,
    }))
}

async fn generate_child_in_batch(state: AppState, child_uuid: Uuid) {
    let child_id = NodeId(child_uuid);
    let (mut request, project_path) = {
        let (project, project_path) = match active_sqlite_project(&state).await {
            Ok(project) => project,
            Err(error) => {
                let _ = state.events_tx.send(ServerEvent::GenerationError {
                    node_id: child_uuid,
                    error,
                });
                return;
            }
        };

        let node = match project.timeline.node(child_id) {
            Ok(n) => n,
            Err(_) => return,
        };

        if node.locked {
            return;
        }

        let request = match build_generate_request(&project, child_id) {
            Ok(req) => req,
            Err(e) => {
                tracing::error!("Failed to build request for child node {child_uuid}: {e}");
                return;
            }
        };
        (request, project_path)
    };
    if let Err(error) = attach_ai_bible_context(&mut request, project_path.clone(), child_id).await
    {
        let _ = state.events_tx.send(ServerEvent::GenerationError {
            node_id: child_uuid,
            error,
        });
        return;
    }

    state.generating.lock().insert(child_uuid);
    mark_node_generating(&state, project_path.clone(), child_id, child_uuid).await;
    run_generation(state, project_path, child_uuid, request).await;
}
