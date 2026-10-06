//! Canonical screenplay-aware pending child plans; acceptance remains a timeline command.
use crate::ai_backends::Backend;
use crate::ai_service::{
    AiGenerateChildrenRequest, active_sqlite_project, load_ai_affect_projection,
    load_ai_bible_context_projection,
};
use crate::backend_error::BackendError;
use crate::prompt_format::build_decompose_prompt;
use crate::state::AppState;
use eidetic_core::ai::backend::{ChildPlan, ChildPlanId, ChildProposal, GenerateChildrenRequest};
use eidetic_core::ai::prompt::build_generate_children_request;
use eidetic_core::timeline::node::NodeId;
use serde::Deserialize;
use std::path::PathBuf;
use uuid::Uuid;

pub async fn generate_children(
    state: &AppState,
    body: AiGenerateChildrenRequest,
) -> Result<ChildPlan, BackendError> {
    let session_id = *state.project_session_id.lock();
    let node_id = NodeId(body.node_id);
    let (mut request, project_path) = {
        let (project, project_path) = active_sqlite_project(state).await?;
        let node = project
            .timeline
            .node(node_id)
            .map_err(|_| BackendError::not_found(format!("node not found: {}", body.node_id)))?;
        if node.content.notes.trim().is_empty() {
            return Err(BackendError::bad_request("node has no notes"));
        }

        let request = build_generate_children_request(&project, node_id)
            .map_err(|error| BackendError::bad_request(error.to_string()))?;
        (request, project_path)
    };
    let memory = attach_ai_generation_context_to_children(
        &mut request,
        project_path.clone(),
        node_id,
        body.story_time_ms,
    )
    .await?;

    let config = state.ai_config.lock().clone();
    let backend = Backend::from_config(&config);
    let prompt = build_decompose_prompt(&request);
    let json_text = backend
        .generate_json(&prompt, &config)
        .await
        .map_err(|error| {
            tracing::error!(
                "Child decomposition failed for node {}: {error}",
                body.node_id
            );
            BackendError::internal(error.to_string())
        })?;

    let children = parse_child_proposals(&json_text, body.node_id)?;
    let plan = ChildPlan {
        id: ChildPlanId::new(format!("child_plan.{}", Uuid::new_v4()))
            .expect("generated child plan ids are non-empty"),
        parent_node_id: node_id,
        target_child_level: request.target_child_level,
        children,
        script_context: request.script_context,
    };
    let session = state.project_session_gate.clone().lock_owned().await;
    if *state.project_session_id.lock() != session_id
        || state.project_database.active_path().as_ref() != Some(&project_path)
    {
        return Err(BackendError::conflict(
            "Child plan project changed before completion",
        ));
    }
    let recorded = plan.clone();
    crate::state::complete_project_session_work(state, session, async move {
        tokio::task::spawn_blocking(move || {
            let mut conn = crate::sqlite::open_write_connection(&project_path)
                .map_err(|error| BackendError::internal(error.to_string()))?;
            crate::child_plan_store::record_child_plan_with_memory(
                &mut conn,
                &recorded,
                0,
                Some(memory),
            )
            .map_err(|error| BackendError::bad_request(error.to_string()))?;
            // Storage owns proposal normalization. Present that exact durable
            // material for review, never the unnormalized provider response.
            let mut reviewed = recorded;
            reviewed.children =
                crate::child_plan_projection_store::load_child_plan_children(&conn, &reviewed.id)
                    .map_err(|error| BackendError::internal(error.to_string()))?;
            Ok::<_, BackendError>(reviewed)
        })
        .await
        .map_err(|error| BackendError::internal(error.to_string()))?
    })
    .await
}

pub(crate) async fn attach_ai_generation_context_to_children(
    request: &mut GenerateChildrenRequest,
    path: PathBuf,
    node_id: NodeId,
    story_time_ms: Option<u64>,
) -> Result<crate::child_plan_memory::ChildPlanMemory, BackendError> {
    let script_path = path.clone();
    let parent = request.parent_node.clone();
    let memory = tokio::task::spawn_blocking(move || {
        let conn = crate::sqlite::open_write_connection(&script_path)
            .map_err(|error| BackendError::internal(error.to_string()))?;
        crate::script_store::create_schema(&conn)
            .map_err(|error| BackendError::internal(error.to_string()))?;
        let tx = conn
            .unchecked_transaction()
            .map_err(|error| BackendError::internal(error.to_string()))?;
        let memory = crate::child_plan_memory::capture(&tx, node_id)
            .map_err(|error| BackendError::bad_request(error.to_string()))?;
        crate::child_plan_memory::validate_parent(&memory, &parent)
            .map_err(|error| BackendError::bad_request(error.to_string()))?;
        tx.commit()
            .map_err(|error| BackendError::internal(error.to_string()))?;
        Ok::<_, BackendError>(memory)
    })
    .await
    .map_err(|error| BackendError::internal(error.to_string()))??;
    request.script_context = Some(memory.script_inputs.clone());
    request.surrounding_context = Default::default();
    request.bible_context = Some(
        load_ai_bible_context_projection(path.clone(), node_id, story_time_ms)
            .await?
            .0,
    );
    request.affect_context = Some(load_ai_affect_projection(path, node_id).await?);
    Ok(memory)
}

pub(crate) fn parse_child_proposals(
    json_text: &str,
    node_id: Uuid,
) -> Result<Vec<ChildProposal>, BackendError> {
    match serde_json::from_str::<Vec<ChildProposal>>(json_text) {
        Ok(children) => Ok(children),
        Err(_) => parse_wrapped_or_single_child_proposal(json_text, node_id),
    }
}

fn parse_wrapped_or_single_child_proposal(
    json_text: &str,
    node_id: Uuid,
) -> Result<Vec<ChildProposal>, BackendError> {
    #[derive(Deserialize)]
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
                Err(BackendError::bad_request(format!(
                    "failed to parse AI response: {error}"
                )))
            }
        },
    }
}

#[cfg(test)]
#[path = "child_plan_memory_service_tests.rs"]
mod tests;
