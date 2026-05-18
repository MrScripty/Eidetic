use axum::extract::{Path, State};
use axum::routing::{get, post, put};
use axum::{Json, Router};
use futures::StreamExt;
use serde::Deserialize;
use std::path::PathBuf;
use uuid::Uuid;

use crate::ai_backends::Backend;
use crate::embeddings::EmbeddingClient;
use crate::prompt_format::{build_chat_prompt, build_decompose_prompt};
use crate::script_document_command;
use crate::state::{AppState, BackendType, ServerEvent};
use eidetic_core::ai::backend::{ChildPlan, ChildProposal, RagChunk};
use eidetic_core::ai::prompt::{build_generate_children_request, build_generate_request};
use eidetic_core::contracts::{
    CommandEnvelope, CommandId, ScriptBlockId, ScriptBlockKind, ScriptDocumentId, ScriptSegmentId,
    ScriptSegmentStatus, ScriptSpanProvenance, SetScriptBlockCommand,
};
use eidetic_core::timeline::node::{ContentStatus, NodeId};

pub fn router() -> Router<AppState> {
    Router::new()
        .route("/ai/generate", post(generate))
        .route("/ai/generate-children", post(generate_children))
        .route("/ai/generate-batch", post(generate_batch))
        .route("/ai/context/{id}", get(preview_context))
        .route("/ai/status", get(status))
        .route("/ai/config", put(config))
}

#[derive(Deserialize)]
struct GenerateBody {
    node_id: Uuid,
}

async fn generate(
    State(state): State<AppState>,
    Json(body): Json<GenerateBody>,
) -> Json<serde_json::Value> {
    let node_id = NodeId(body.node_id);

    // Validate and build the request while holding the project lock briefly.
    let (request, project_path) = {
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

    // Mark as generating.
    state.generating.lock().insert(body.node_id);

    // Update node status to Generating.
    {
        let mut project_guard = state.project.lock();
        if let Some(project) = project_guard.as_mut() {
            if let Ok(node) = project.timeline.node_mut(node_id) {
                node.content.status = ContentStatus::Generating;
            }
        }
    }
    let _ = state.events_tx.send(ServerEvent::NodeUpdated {
        node_id: body.node_id,
    });

    // Spawn the generation task.
    let state_clone = state.clone();
    let node_uuid = body.node_id;
    tokio::spawn(async move {
        run_generation(state_clone, project_path, node_uuid, request).await;
    });

    Json(serde_json::json!({
        "status": "started",
        "node_id": body.node_id.to_string(),
    }))
}

// ---------------------------------------------------------------------------
// Child decomposition (replaces plan-beats)
// ---------------------------------------------------------------------------

#[derive(Deserialize)]
struct GenerateChildrenBody {
    node_id: Uuid,
}

/// AI-powered decomposition: analyzes a node's notes and returns
/// a structured child plan that the user can edit before applying.
async fn generate_children(
    State(state): State<AppState>,
    Json(body): Json<GenerateChildrenBody>,
) -> Json<serde_json::Value> {
    let node_id = NodeId(body.node_id);

    let request = {
        let (project, _) = match active_sqlite_project(&state).await {
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

        match build_generate_children_request(&project, node_id) {
            Ok(req) => req,
            Err(e) => {
                return Json(serde_json::json!({ "error": e.to_string() }));
            }
        }
    };

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

    // Parse the JSON response into child proposals.
    let children: Vec<ChildProposal> = match serde_json::from_str::<Vec<ChildProposal>>(&json_text)
    {
        Ok(c) => c,
        Err(_) => {
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
            match serde_json::from_str::<Wrapped>(&json_text) {
                Ok(w) => w.items,
                Err(_) => match serde_json::from_str::<ChildProposal>(&json_text) {
                    Ok(single) => vec![single],
                    Err(e) => {
                        tracing::warn!(
                            "Failed to parse child plan JSON for node {}: {e}\nRaw: {json_text}",
                            body.node_id
                        );
                        return Json(serde_json::json!({
                            "error": format!("failed to parse AI response: {e}"),
                            "raw": json_text,
                        }));
                    }
                },
            }
        }
    };

    let plan = ChildPlan {
        parent_node_id: node_id,
        target_child_level: request.target_child_level,
        children,
    };

    Json(serde_json::to_value(&plan).unwrap())
}

// ---------------------------------------------------------------------------
// Batch generation (all children of a parent)
// ---------------------------------------------------------------------------

#[derive(Deserialize)]
struct GenerateBatchBody {
    parent_node_id: Uuid,
}

/// Generate content for all children of a parent node sequentially.
async fn generate_batch(
    State(state): State<AppState>,
    Json(body): Json<GenerateBatchBody>,
) -> Json<serde_json::Value> {
    let parent_id = NodeId(body.parent_node_id);

    // Collect child node IDs in order.
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
    tokio::spawn(async move {
        for child_uuid in &child_ids {
            let child_id = NodeId(*child_uuid);

            let (request, project_path) = {
                let (project, project_path) = match active_sqlite_project(&state_clone).await {
                    Ok(project) => project,
                    Err(error) => {
                        let _ = state_clone.events_tx.send(ServerEvent::GenerationError {
                            node_id: *child_uuid,
                            error,
                        });
                        continue;
                    }
                };

                let node = match project.timeline.node(child_id) {
                    Ok(n) => n,
                    Err(_) => continue,
                };

                if node.locked {
                    continue;
                }

                let request = match build_generate_request(&project, child_id) {
                    Ok(req) => req,
                    Err(e) => {
                        tracing::error!("Failed to build request for child node {child_uuid}: {e}");
                        continue;
                    }
                };
                (request, project_path)
            };

            // Mark as generating.
            state_clone.generating.lock().insert(*child_uuid);
            {
                let mut project_guard = state_clone.project.lock();
                if let Some(project) = project_guard.as_mut() {
                    if let Ok(node) = project.timeline.node_mut(child_id) {
                        node.content.status = ContentStatus::Generating;
                    }
                }
            }
            let _ = state_clone.events_tx.send(ServerEvent::NodeUpdated {
                node_id: *child_uuid,
            });

            run_generation(state_clone.clone(), project_path, *child_uuid, request).await;
        }
    });

    Json(serde_json::json!({
        "status": "started",
        "parent_node_id": body.parent_node_id.to_string(),
        "child_count": child_count,
    }))
}

async fn run_generation(
    state: AppState,
    project_path: PathBuf,
    node_uuid: Uuid,
    mut request: eidetic_core::ai::backend::GenerateRequest,
) {
    let node_id = NodeId(node_uuid);
    let config = state.ai_config.lock().clone();
    let backend = Backend::from_config(&config);

    // RAG: retrieve relevant reference chunks if vector store has entries.
    if !state.vector_store.lock().is_empty() {
        let query = &request.target_node.content.notes;
        let embed_client =
            EmbeddingClient::new(&config.base_url, crate::state::constants::EMBEDDING_MODEL);
        if let Ok(query_embedding) = embed_client.embed(query).await {
            let store = state.vector_store.lock();
            let results = store.search(&query_embedding, crate::state::constants::RAG_TOP_K);
            request.rag_context = results
                .into_iter()
                .map(|(chunk, score)| RagChunk {
                    source: chunk.document_name.clone(),
                    content: chunk.content.clone(),
                    relevance_score: score,
                })
                .collect();
        }
    }

    let prompt = build_chat_prompt(&request);

    // Broadcast the formatted prompt context.
    let _ = state.events_tx.send(ServerEvent::GenerationContext {
        node_id: node_uuid,
        system_prompt: prompt.system.clone(),
        user_prompt: prompt.user.clone(),
    });

    let stream_result = backend.generate(&prompt, &config).await;

    let mut stream = match stream_result {
        Ok(s) => s,
        Err(e) => {
            tracing::error!("AI generation failed for node {node_uuid}: {e}");
            {
                let mut project_guard = state.project.lock();
                if let Some(project) = project_guard.as_mut() {
                    if let Ok(node) = project.timeline.node_mut(node_id) {
                        node.content.status = ContentStatus::NotesOnly;
                    }
                }
            }
            let _ = state.events_tx.send(ServerEvent::GenerationError {
                node_id: node_uuid,
                error: e.to_string(),
            });
            state.generating.lock().remove(&node_uuid);
            return;
        }
    };

    let mut full_text = String::new();
    let mut tokens_generated: usize = 0;

    while let Some(item) = stream.next().await {
        match item {
            Ok(token) => {
                full_text.push_str(&token);
                tokens_generated += 1;
                let _ = state.events_tx.send(ServerEvent::GenerationProgress {
                    node_id: node_uuid,
                    token,
                    tokens_generated,
                });
            }
            Err(e) => {
                tracing::warn!("Stream error during generation for node {node_uuid}: {e}");
                break;
            }
        }
    }

    // Store the result.
    if full_text.is_empty() {
        {
            let mut project_guard = state.project.lock();
            if let Some(project) = project_guard.as_mut() {
                if let Ok(node) = project.timeline.node_mut(node_id) {
                    node.content.status = ContentStatus::NotesOnly;
                }
            }
        }
        let _ = state.events_tx.send(ServerEvent::GenerationError {
            node_id: node_uuid,
            error: "AI produced no output".into(),
        });
    } else {
        let metadata = {
            let mut project_guard = state.project.lock();
            let Some(project) = project_guard.as_mut() else {
                let _ = state.events_tx.send(ServerEvent::GenerationError {
                    node_id: node_uuid,
                    error: "no project loaded".into(),
                });
                state.generating.lock().remove(&node_uuid);
                return;
            };
            let Ok(node) = project.timeline.node_mut(node_id) else {
                let _ = state.events_tx.send(ServerEvent::GenerationError {
                    node_id: node_uuid,
                    error: "node not found".into(),
                });
                state.generating.lock().remove(&node_uuid);
                return;
            };
            node.content.status = ContentStatus::HasContent;
            GeneratedScriptMetadata {
                project_name: project.name.clone(),
                start_ms: node.time_range.start_ms,
                end_ms: node.time_range.end_ms,
            }
        };
        if let Err(error) =
            persist_generated_script_block(project_path, node_uuid, metadata, full_text.clone())
                .await
        {
            tracing::error!("Failed to persist generated script for node {node_uuid}: {error}");
            let _ = state.events_tx.send(ServerEvent::GenerationError {
                node_id: node_uuid,
                error,
            });
            state.generating.lock().remove(&node_uuid);
            return;
        }
        let _ = state
            .events_tx
            .send(ServerEvent::GenerationComplete { node_id: node_uuid });
        let _ = state
            .events_tx
            .send(ServerEvent::NodeUpdated { node_id: node_uuid });
        let _ = state.events_tx.send(ServerEvent::ScriptChanged);
        state.trigger_save();

        // Generate scene recap for continuity context (non-fatal).
        generate_scene_recap(&state, node_uuid, &full_text).await;
    }

    state.generating.lock().remove(&node_uuid);
}

#[derive(Debug, Clone)]
struct GeneratedScriptMetadata {
    project_name: String,
    start_ms: u64,
    end_ms: u64,
}

async fn persist_generated_script_block(
    project_path: PathBuf,
    node_uuid: Uuid,
    metadata: GeneratedScriptMetadata,
    full_text: String,
) -> Result<(), String> {
    tokio::task::spawn_blocking(move || {
        let mut conn = crate::sqlite::open_write_connection(&project_path)
            .map_err(|error| error.to_string())?;
        let command =
            generated_script_block_command(Uuid::new_v4(), node_uuid, metadata, full_text)?;
        script_document_command::apply_set_script_block(&mut conn, &command, 0)
            .map_err(|error| error.to_string())?;
        Ok(())
    })
    .await
    .map_err(|error| format!("script persistence task failed: {error}"))?
}

fn generated_script_block_command(
    command_id: Uuid,
    node_uuid: Uuid,
    metadata: GeneratedScriptMetadata,
    full_text: String,
) -> Result<CommandEnvelope<SetScriptBlockCommand>, String> {
    Ok(CommandEnvelope {
        id: CommandId(command_id),
        payload: SetScriptBlockCommand {
            document_id: ScriptDocumentId::new("script.document.main")
                .map_err(|error| error.to_string())?,
            document_title: metadata.project_name,
            document_sort_order: 0,
            segment_id: ScriptSegmentId::new(format!("script.segment.{node_uuid}"))
                .map_err(|error| error.to_string())?,
            source_node_id: Some(node_uuid.to_string()),
            segment_start_ms: metadata.start_ms,
            segment_end_ms: metadata.end_ms,
            segment_status: ScriptSegmentStatus::Current,
            segment_sort_order: 0,
            block_id: ScriptBlockId::new(format!("script.block.{node_uuid}.generated"))
                .map_err(|error| error.to_string())?,
            block_kind: ScriptBlockKind::Action,
            text: full_text,
            span_provenance: ScriptSpanProvenance::AiGenerated,
            sort_order: 0,
        },
    })
}

/// Generate a compact scene recap after content generation.
async fn generate_scene_recap(state: &AppState, node_uuid: Uuid, script: &str) {
    use crate::prompt_format::build_recap_prompt;

    let node_id = NodeId(node_uuid);

    // Find the preceding sibling's recap for rolling summary behavior.
    let preceding_recap = {
        let project_guard = state.project.lock();
        let Some(project) = project_guard.as_ref() else {
            return;
        };

        let siblings = project.timeline.siblings_of(node_id);
        // Find the sibling that immediately precedes this node in time.
        let node = match project.timeline.node(node_id) {
            Ok(n) => n,
            Err(_) => return,
        };
        siblings
            .iter()
            .filter(|s| s.time_range.end_ms <= node.time_range.start_ms)
            .last()
            .and_then(|s| s.content.scene_recap.clone())
    };

    let config = state.ai_config.lock().clone();
    let backend = Backend::from_config(&config);

    let mut recap_config = config.clone();
    recap_config.max_tokens = 512;

    let prompt = build_recap_prompt(script, preceding_recap.as_deref());

    let recap_text = match backend.generate_full(&prompt, &recap_config).await {
        Ok(text) => text.trim().to_string(),
        Err(e) => {
            tracing::warn!("Scene recap generation failed for node {node_uuid}: {e}");
            return;
        }
    };

    if recap_text.is_empty() {
        tracing::warn!("Scene recap was empty for node {node_uuid}");
        return;
    }

    {
        let mut project_guard = state.project.lock();
        if let Some(project) = project_guard.as_mut() {
            if let Ok(node) = project.timeline.node_mut(node_id) {
                node.content.scene_recap = Some(recap_text);
            }
        }
    }

    let _ = state
        .events_tx
        .send(ServerEvent::NodeUpdated { node_id: node_uuid });
    state.trigger_save();

    tracing::info!("Scene recap generated for node {node_uuid}");
}

async fn status(State(state): State<AppState>) -> Json<serde_json::Value> {
    let config = state.ai_config.lock().clone();
    let backend = Backend::from_config(&config);

    match backend.health_check().await {
        Ok(s) => {
            let display_model =
                if config.model.eq_ignore_ascii_case("auto") || config.model.is_empty() {
                    if s.model.is_empty() {
                        "auto".to_string()
                    } else {
                        s.model.clone()
                    }
                } else {
                    config.model.clone()
                };
            Json(serde_json::json!({
                "backend": config.backend_type,
                "model": display_model,
                "connected": s.connected,
                "message": s.message,
            }))
        }
        Err(e) => Json(serde_json::json!({
            "backend": config.backend_type,
            "model": config.model,
            "connected": false,
            "error": e.to_string(),
        })),
    }
}

#[derive(Deserialize)]
struct ConfigUpdate {
    backend_type: Option<BackendType>,
    model: Option<String>,
    temperature: Option<f32>,
    max_tokens: Option<usize>,
    base_url: Option<String>,
    api_key: Option<String>,
}

async fn config(
    State(state): State<AppState>,
    Json(body): Json<ConfigUpdate>,
) -> Json<serde_json::Value> {
    let mut config = state.ai_config.lock();
    if let Some(bt) = body.backend_type {
        config.backend_type = bt;
    }
    if let Some(m) = body.model {
        config.model = m;
    }
    if let Some(t) = body.temperature {
        config.temperature = t;
    }
    if let Some(mt) = body.max_tokens {
        config.max_tokens = mt;
    }
    if let Some(url) = body.base_url {
        config.base_url = url;
    }
    if let Some(key) = body.api_key {
        config.api_key = Some(key);
    }
    Json(serde_json::to_value(&*config).unwrap())
}

/// Preview the formatted AI context/prompt for a node without generating.
async fn preview_context(
    State(state): State<AppState>,
    Path(id): Path<Uuid>,
) -> Json<serde_json::Value> {
    let node_id = NodeId(id);

    let (project, _) = match active_sqlite_project(&state).await {
        Ok(project) => project,
        Err(error) => return Json(serde_json::json!({ "error": error })),
    };

    let request = match build_generate_request(&project, node_id) {
        Ok(req) => req,
        Err(e) => return Json(serde_json::json!({ "error": e.to_string() })),
    };

    let prompt = build_chat_prompt(&request);

    Json(serde_json::json!({
        "system": prompt.system,
        "user": prompt.user,
    }))
}

async fn active_sqlite_project(
    state: &AppState,
) -> Result<(eidetic_core::Project, PathBuf), String> {
    let Some(project_path) = state.project_path.lock().clone() else {
        return Err("no project loaded".to_string());
    };
    if state.project.lock().is_none() {
        return Err("no project loaded".to_string());
    }
    let (project, _) = crate::persistence::load_project(&project_path).await?;
    Ok((project, project_path))
}

#[cfg(test)]
mod tests {
    use super::*;
    use axum::body::{Body, to_bytes};
    use axum::http::{Request, StatusCode};
    use eidetic_core::Template;
    use tower::util::ServiceExt;

    #[test]
    fn generated_script_block_command_targets_main_document_with_ai_provenance() {
        let command_id = Uuid::new_v4();
        let node_uuid = Uuid::new_v4();
        let command = generated_script_block_command(
            command_id,
            node_uuid,
            GeneratedScriptMetadata {
                project_name: "Pilot".to_string(),
                start_ms: 1_000,
                end_ms: 5_000,
            },
            "INT. KITCHEN - MORNING\n\nAda enters.".to_string(),
        )
        .unwrap();

        assert_eq!(command.id, CommandId(command_id));
        assert_eq!(command.payload.document_id.as_str(), "script.document.main");
        assert_eq!(command.payload.document_title, "Pilot");
        assert_eq!(
            command.payload.segment_id.as_str(),
            format!("script.segment.{node_uuid}")
        );
        assert_eq!(
            command.payload.source_node_id.as_deref(),
            Some(node_uuid.to_string().as_str())
        );
        assert_eq!(command.payload.segment_start_ms, 1_000);
        assert_eq!(command.payload.segment_end_ms, 5_000);
        assert_eq!(command.payload.block_kind, ScriptBlockKind::Action);
        assert_eq!(
            command.payload.span_provenance,
            ScriptSpanProvenance::AiGenerated
        );
    }

    #[tokio::test]
    async fn preview_context_hydrates_story_arcs_from_sqlite_when_project_mirror_is_stale() {
        let path =
            std::env::temp_dir().join(format!("eidetic-ai-context-arcs-{}.db", Uuid::new_v4()));
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
        let app = router().with_state(state);

        let response = app
            .oneshot(
                Request::builder()
                    .method("GET")
                    .uri(format!("/ai/context/{}", node_arc.node_id.0))
                    .body(Body::empty())
                    .unwrap(),
            )
            .await
            .expect("route response");

        assert_eq!(response.status(), StatusCode::OK);
        let value = response_json(response).await;
        assert!(
            value["user"]
                .as_str()
                .expect("user prompt")
                .contains(&arc_name),
            "prompt should include arc name loaded from sqlite"
        );
        assert!(
            value["user"]
                .as_str()
                .expect("user prompt")
                .contains("SQLite-only rain argument"),
            "prompt should include node notes loaded from sqlite"
        );

        let _ = std::fs::remove_file(path);
    }

    async fn response_json(response: axum::response::Response) -> serde_json::Value {
        let body = to_bytes(response.into_body(), 1024 * 1024)
            .await
            .expect("body bytes");
        serde_json::from_slice(&body).expect("json response")
    }
}
