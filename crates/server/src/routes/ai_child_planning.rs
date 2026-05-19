use axum::Json;
use axum::extract::State;
use eidetic_core::ai::backend::{ChildPlan, ChildPlanId, ChildProposal};
use eidetic_core::ai::prompt::build_generate_children_request;
use eidetic_core::timeline::node::NodeId;
use serde::Deserialize;
use uuid::Uuid;

use crate::ai_backends::Backend;
use crate::prompt_format::build_decompose_prompt;
use crate::state::AppState;

use super::{active_sqlite_project, attach_ai_bible_context_to_children};

#[derive(Deserialize)]
pub(super) struct GenerateChildrenBody {
    node_id: Uuid,
}

/// AI-powered decomposition: analyzes a node's notes and returns
/// a structured child plan that the user can edit before applying.
pub(super) async fn generate_children(
    State(state): State<AppState>,
    Json(body): Json<GenerateChildrenBody>,
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

        if node.content.notes.trim().is_empty() {
            return Json(serde_json::json!({ "error": "node has no notes" }));
        }

        let request = match build_generate_children_request(&project, node_id) {
            Ok(req) => req,
            Err(e) => {
                return Json(serde_json::json!({ "error": e.to_string() }));
            }
        };
        (request, project_path)
    };
    if let Err(error) =
        attach_ai_bible_context_to_children(&mut request, project_path.clone(), node_id).await
    {
        return Json(serde_json::json!({ "error": error }));
    }

    let config = state.ai_config.lock().clone();
    let backend = Backend::from_config(&config);
    let prompt = build_decompose_prompt(&request);

    let json_text = match backend.generate_json(&prompt, &config).await {
        Ok(text) => text,
        Err(e) => {
            tracing::error!("Child decomposition failed for node {}: {e}", body.node_id);
            return Json(serde_json::json!({ "error": e.to_string() }));
        }
    };

    let children = match parse_child_proposals(&json_text, body.node_id) {
        Ok(children) => children,
        Err(error) => return Json(error),
    };

    let plan = ChildPlan {
        id: ChildPlanId::new(format!("child_plan.{}", uuid::Uuid::new_v4()))
            .expect("generated child plan ids are non-empty"),
        parent_node_id: node_id,
        target_child_level: request.target_child_level,
        children,
    };
    let mut conn = match state.project_database.open_active_write_connection() {
        Ok(conn) => conn,
        Err(error) => return Json(serde_json::json!({ "error": error.to_string() })),
    };
    if let Err(error) = crate::child_plan_store::record_child_plan(&mut conn, &plan, 0) {
        return Json(serde_json::json!({ "error": error.to_string() }));
    }

    Json(serde_json::to_value(&plan).expect("child plans serialize to JSON"))
}

fn parse_child_proposals(
    json_text: &str,
    node_id: Uuid,
) -> Result<Vec<ChildProposal>, serde_json::Value> {
    match serde_json::from_str::<Vec<ChildProposal>>(json_text) {
        Ok(children) => Ok(children),
        Err(_) => parse_wrapped_or_single_child_proposal(json_text, node_id),
    }
}

fn parse_wrapped_or_single_child_proposal(
    json_text: &str,
    node_id: Uuid,
) -> Result<Vec<ChildProposal>, serde_json::Value> {
    #[derive(serde::Deserialize)]
    struct Wrapped {
        #[serde(
            alias = "acts",
            alias = "beats",
            alias = "children",
            alias = "sequences",
            alias = "scenes"
        )]
        items: Vec<ChildProposal>,
    }
    match serde_json::from_str::<Wrapped>(json_text) {
        Ok(wrapped) => Ok(wrapped.items),
        Err(_) => match serde_json::from_str::<ChildProposal>(json_text) {
            Ok(single) => Ok(vec![single]),
            Err(error) => {
                tracing::warn!(
                    "Failed to parse child plan JSON for node {node_id}: {error}\nRaw: {json_text}"
                );
                Err(serde_json::json!({
                    "error": format!("failed to parse AI response: {error}"),
                    "raw": json_text,
                }))
            }
        },
    }
}
