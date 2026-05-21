use std::path::PathBuf;

use eidetic_core::Project;
use eidetic_core::ai::backend::{ChildPlan, ChildPlanId, ChildProposal, GenerateChildrenRequest};
use eidetic_core::ai::prompt::{build_generate_children_request, build_generate_request};
use eidetic_core::contracts::{AiBibleContextProjection, ProjectionEnvelope};
use eidetic_core::timeline::node::NodeId;
use serde::{Deserialize, Serialize};
use uuid::Uuid;

use crate::ai_backends::Backend;
use crate::backend_error::BackendError;
use crate::prompt_format::{build_chat_prompt, build_decompose_prompt};
use crate::state::{AiConfig, AppState, BackendType};

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct AiStatus {
    pub backend: BackendType,
    pub model: String,
    pub connected: bool,
    pub message: Option<String>,
    pub error: Option<String>,
}

#[derive(Debug, Clone, Default, Deserialize)]
pub struct AiConfigUpdate {
    pub backend_type: Option<BackendType>,
    pub model: Option<String>,
    pub temperature: Option<f32>,
    pub max_tokens: Option<usize>,
    pub base_url: Option<String>,
    pub api_key: Option<Option<String>>,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct AiContextPreview {
    pub system: String,
    pub user: String,
}

#[derive(Debug, Clone, Deserialize)]
pub struct AiGenerateChildrenRequest {
    pub node_id: Uuid,
}

pub async fn get_ai_status(state: &AppState) -> AiStatus {
    let config = state.ai_config.lock().clone();
    let backend = Backend::from_config(&config);

    match backend.health_check().await {
        Ok(status) => AiStatus {
            backend: config.backend_type,
            model: display_model(&config, &status.model),
            connected: status.connected,
            message: Some(status.message),
            error: None,
        },
        Err(error) => AiStatus {
            backend: config.backend_type,
            model: config.model,
            connected: false,
            message: None,
            error: Some(error.to_string()),
        },
    }
}

pub async fn preview_ai_context(
    state: &AppState,
    node_uuid: Uuid,
) -> Result<AiContextPreview, BackendError> {
    let node_id = NodeId(node_uuid);
    let (project, project_path) = active_sqlite_project(state).await?;
    let mut request = build_generate_request(&project, node_id)
        .map_err(|error| BackendError::BadRequest(error.to_string()))?;
    attach_ai_bible_context(&mut request, project_path, node_id).await?;
    let prompt = build_chat_prompt(&request);

    Ok(AiContextPreview {
        system: prompt.system,
        user: prompt.user,
    })
}

pub async fn generate_children(
    state: &AppState,
    body: AiGenerateChildrenRequest,
) -> Result<ChildPlan, BackendError> {
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
    attach_ai_bible_context_to_children(&mut request, project_path, node_id).await?;

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
    };
    let mut conn = state
        .project_database
        .open_active_write_connection()
        .map_err(|error| BackendError::internal(error.to_string()))?;
    crate::child_plan_store::record_child_plan(&mut conn, &plan, 0)
        .map_err(|error| BackendError::internal(error.to_string()))?;

    Ok(plan)
}

pub fn update_ai_config(state: &AppState, update: AiConfigUpdate) -> AiConfig {
    let mut config = state.ai_config.lock();
    if let Some(backend_type) = update.backend_type {
        config.backend_type = backend_type;
    }
    if let Some(model) = update.model {
        config.model = model;
    }
    if let Some(temperature) = update.temperature {
        config.temperature = temperature;
    }
    if let Some(max_tokens) = update.max_tokens {
        config.max_tokens = max_tokens;
    }
    if let Some(base_url) = update.base_url {
        config.base_url = base_url;
    }
    if let Some(api_key) = update.api_key {
        config.api_key = api_key.filter(|value| !value.is_empty());
    }
    config.clone()
}

async fn active_sqlite_project(state: &AppState) -> Result<(Project, PathBuf), BackendError> {
    let Some(project_path) = state.project_database.active_path() else {
        return Err(BackendError::NotFound("no project loaded".to_string()));
    };
    if state.project.lock().is_none() {
        return Err(BackendError::NotFound("no project loaded".to_string()));
    }
    let (project, _) = crate::persistence::load_project(&project_path)
        .await
        .map_err(BackendError::Internal)?;
    Ok((project, project_path))
}

async fn attach_ai_bible_context(
    request: &mut eidetic_core::ai::backend::GenerateRequest,
    path: PathBuf,
    node_id: NodeId,
) -> Result<(), BackendError> {
    request.bible_context = Some(load_ai_bible_context_projection(path, node_id).await?);
    Ok(())
}

async fn load_ai_bible_context_projection(
    path: PathBuf,
    node_id: NodeId,
) -> Result<ProjectionEnvelope<AiBibleContextProjection>, BackendError> {
    tokio::task::spawn_blocking(move || {
        let conn = crate::sqlite::open_write_connection(&path).map_err(|error| {
            BackendError::Internal(format!("open AI bible context database failed: {error}"))
        })?;
        crate::ai_context_projection::load_ai_bible_context_projection(&conn, node_id)
            .map_err(|error| BackendError::Internal(error.to_string()))
    })
    .await
    .map_err(|error| {
        BackendError::Internal(format!("AI bible context projection task failed: {error}"))
    })?
}

async fn attach_ai_bible_context_to_children(
    request: &mut GenerateChildrenRequest,
    path: PathBuf,
    node_id: NodeId,
) -> Result<(), BackendError> {
    request.bible_context = Some(load_ai_bible_context_projection(path, node_id).await?);
    Ok(())
}

fn parse_child_proposals(
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

fn display_model(config: &AiConfig, detected_model: &str) -> String {
    if config.model.eq_ignore_ascii_case("auto") || config.model.is_empty() {
        if detected_model.is_empty() {
            "auto".to_string()
        } else {
            detected_model.to_string()
        }
    } else {
        config.model.clone()
    }
}

#[cfg(test)]
mod tests {
    use super::{
        AiConfigUpdate, AiGenerateChildrenRequest, display_model, generate_children,
        preview_ai_context, update_ai_config,
    };
    use crate::state::{AiConfig, AppState, BackendType};
    use eidetic_core::Template;
    use eidetic_core::timeline::node::ContentStatus;
    use uuid::Uuid;

    #[test]
    fn display_model_uses_detected_model_for_auto_config() {
        let config = AiConfig {
            model: "auto".to_string(),
            ..AiConfig::default()
        };

        assert_eq!(display_model(&config, "served-model"), "served-model");
    }

    #[tokio::test]
    async fn update_ai_config_applies_sparse_updates_and_filters_blank_key() {
        let state = AppState::new().await;

        let config = update_ai_config(
            &state,
            AiConfigUpdate {
                backend_type: Some(BackendType::OpenRouter),
                model: Some("open-model".to_string()),
                temperature: Some(0.2),
                max_tokens: Some(1024),
                base_url: Some("https://example.test/v1".to_string()),
                api_key: Some(Some(String::new())),
            },
        );

        assert_eq!(config.backend_type, BackendType::OpenRouter);
        assert_eq!(config.model, "open-model");
        assert_eq!(config.temperature, 0.2);
        assert_eq!(config.max_tokens, 1024);
        assert_eq!(config.base_url, "https://example.test/v1");
        assert_eq!(config.api_key, None);
    }

    #[tokio::test]
    async fn generate_children_requires_loaded_project() {
        let state = AppState::new().await;

        let error = generate_children(
            &state,
            AiGenerateChildrenRequest {
                node_id: Uuid::new_v4(),
            },
        )
        .await
        .expect_err("missing project");

        assert_eq!(error.message(), "no project loaded");
    }

    #[tokio::test]
    async fn preview_ai_context_hydrates_story_arcs_from_sqlite_when_project_mirror_is_stale() {
        let path =
            std::env::temp_dir().join(format!("eidetic-ai-service-context-{}.db", Uuid::new_v4()));
        let state = AppState::new().await;
        let mut project = Template::MultiCam.build_project("AI Context Test");
        let node_arc = project.timeline.node_arcs[0].clone();
        let node = project
            .timeline
            .node_mut(node_arc.node_id)
            .expect("tagged node");
        node.content.notes = "SQLite-only rain argument".to_string();
        node.content.status = ContentStatus::NotesOnly;
        let arc_name = project
            .arcs
            .iter()
            .find(|arc| arc.id == node_arc.arc_id)
            .expect("tagged arc")
            .name
            .clone();
        crate::persistence::save_project(&project, &path, None)
            .await
            .expect("seed project database");
        project.arcs.clear();
        let node = project
            .timeline
            .node_mut(node_arc.node_id)
            .expect("tagged node");
        node.content.notes.clear();
        node.content.status = ContentStatus::Empty;
        *state.project.lock() = Some(project);
        *state.project_path.lock() = Some(path.clone());

        let preview = preview_ai_context(&state, node_arc.node_id.0)
            .await
            .expect("preview");

        assert!(preview.user.contains(&arc_name));
        assert!(preview.user.contains("SQLite-only rain argument"));

        let _ = std::fs::remove_file(path);
    }
}
