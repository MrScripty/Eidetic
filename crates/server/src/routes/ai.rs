use axum::extract::{Path, State};
use axum::routing::{get, post, put};
use axum::{Json, Router};
use futures::StreamExt;
use serde::Deserialize;
use uuid::Uuid;

use crate::ai_backends::Backend;
use crate::embeddings::EmbeddingClient;
use crate::prompt_format::{build_chat_prompt, build_decompose_prompt};
use crate::state::{AppState, BackendType, ServerEvent};
use eidetic_core::ai::backend::{ChildPlan, ChildProposal, RagChunk};
use eidetic_core::ai::prompt::{build_generate_children_request, build_generate_request};
use eidetic_core::timeline::node::{ContentStatus, NodeId};

pub fn router() -> Router<AppState> {
    Router::new()
        .route("/ai/generate", post(generate))
        .route("/ai/generate-children", post(generate_children))
        .route("/ai/generate-batch", post(generate_batch))
        .route("/ai/react", post(react))
        .route("/ai/extract", post(extract_entities))
        .route("/ai/extract/commit", post(commit_extraction))
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
    let request = {
        let project_guard = state.project.lock();
        let Some(project) = project_guard.as_ref() else {
            return Json(serde_json::json!({ "error": "no project loaded" }));
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

        match build_generate_request(project, node_id) {
            Ok(req) => req,
            Err(e) => {
                return Json(serde_json::json!({ "error": e.to_string() }));
            }
        }
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
        run_generation(state_clone, node_uuid, request).await;
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
        let project_guard = state.project.lock();
        let Some(project) = project_guard.as_ref() else {
            return Json(serde_json::json!({ "error": "no project loaded" }));
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

        match build_generate_children_request(project, node_id) {
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
            tracing::error!(
                "Child decomposition failed for node {}: {e}",
                body.node_id
            );
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
                #[serde(alias = "acts", alias = "beats", alias = "children", alias = "sequences", alias = "scenes")]
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
        let project_guard = state.project.lock();
        let Some(project) = project_guard.as_ref() else {
            return Json(serde_json::json!({ "error": "no project loaded" }));
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

            let request = {
                let project_guard = state_clone.project.lock();
                let Some(project) = project_guard.as_ref() else {
                    break;
                };

                let node = match project.timeline.node(child_id) {
                    Ok(n) => n,
                    Err(_) => continue,
                };

                if node.locked {
                    continue;
                }

                match build_generate_request(project, child_id) {
                    Ok(req) => req,
                    Err(e) => {
                        tracing::error!(
                            "Failed to build request for child node {child_uuid}: {e}"
                        );
                        continue;
                    }
                }
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

            run_generation(state_clone.clone(), *child_uuid, request).await;
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
    node_uuid: Uuid,
    mut request: eidetic_core::ai::backend::GenerateRequest,
) {
    let node_id = NodeId(node_uuid);
    let config = state.ai_config.lock().clone();
    let backend = Backend::from_config(&config);

    // RAG: retrieve relevant reference chunks if vector store has entries.
    if !state.vector_store.lock().is_empty() {
        let query = &request.target_node.content.notes;
        let embed_client = EmbeddingClient::new(
            &config.base_url,
            crate::state::constants::EMBEDDING_MODEL,
        );
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
                tracing::warn!(
                    "Stream error during generation for node {node_uuid}: {e}"
                );
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
        {
            let mut project_guard = state.project.lock();
            if let Some(project) = project_guard.as_mut() {
                if let Ok(node) = project.timeline.node_mut(node_id) {
                    node.content.content = full_text.clone();
                    node.content.status = ContentStatus::HasContent;
                }
            }
        }
        let _ = state
            .events_tx
            .send(ServerEvent::GenerationComplete { node_id: node_uuid });
        let _ = state
            .events_tx
            .send(ServerEvent::NodeUpdated { node_id: node_uuid });
        state.trigger_save();

        // Generate scene recap for continuity context (non-fatal).
        generate_scene_recap(&state, node_uuid, &full_text).await;

        // Auto-extract entities from the newly generated text.
        let extract_state = state.clone();
        tokio::spawn(async move {
            auto_extract_and_commit(extract_state, node_uuid).await;
        });
    }

    state.generating.lock().remove(&node_uuid);
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

    let project_guard = state.project.lock();
    let Some(project) = project_guard.as_ref() else {
        return Json(serde_json::json!({ "error": "no project loaded" }));
    };

    let request = match build_generate_request(project, node_id) {
        Ok(req) => req,
        Err(e) => return Json(serde_json::json!({ "error": e.to_string() })),
    };

    let prompt = build_chat_prompt(&request);

    Json(serde_json::json!({
        "system": prompt.system,
        "user": prompt.user,
    }))
}

#[derive(Deserialize)]
struct ReactBody {
    node_id: Uuid,
}

async fn react(
    State(state): State<AppState>,
    Json(body): Json<ReactBody>,
) -> Json<serde_json::Value> {
    let node_id = NodeId(body.node_id);

    let (edit_context, downstream_info) = {
        let project_guard = state.project.lock();
        let Some(project) = project_guard.as_ref() else {
            return Json(serde_json::json!({ "error": "no project loaded" }));
        };

        let edit_ctx =
            match eidetic_core::ai::consistency::build_edit_context(project, node_id) {
                Ok(ctx) => ctx,
                Err(e) => return Json(serde_json::json!({ "error": e.to_string() })),
            };

        let downstream_ids =
            eidetic_core::ai::consistency::downstream_node_ids(project, node_id);
        let mut downstream: Vec<(Uuid, String, String)> = Vec::new();
        for did in downstream_ids {
            if let Ok(node) = project.timeline.node(did) {
                let text = &node.content.content;
                if !text.is_empty() {
                    downstream.push((did.0, node.name.clone(), text.clone()));
                }
            }
        }

        (edit_ctx, downstream)
    };

    if downstream_info.is_empty() {
        return Json(serde_json::json!({ "status": "no_downstream_nodes" }));
    }

    let state_clone = state.clone();
    let source_node_id = body.node_id;
    tokio::spawn(async move {
        run_consistency_check(state_clone, source_node_id, edit_context, downstream_info)
            .await;
    });

    Json(serde_json::json!({ "status": "checking" }))
}

async fn run_consistency_check(
    state: AppState,
    source_node_id: Uuid,
    edit_context: eidetic_core::ai::backend::EditContext,
    downstream_nodes: Vec<(Uuid, String, String)>,
) {
    use crate::prompt_format::build_consistency_prompt;

    let config = state.ai_config.lock().clone();
    let backend = Backend::from_config(&config);
    let prompt = build_consistency_prompt(&edit_context, &downstream_nodes);

    let response = match backend.generate_full(&prompt, &config).await {
        Ok(text) => text,
        Err(e) => {
            tracing::error!(
                "Consistency check failed for node {source_node_id}: {e}"
            );
            let _ = state.events_tx.send(ServerEvent::ConsistencyComplete {
                source_node_id,
                suggestion_count: 0,
            });
            return;
        }
    };

    let json_str = extract_json_array(&response);
    let suggestions: Vec<RawConsistencySuggestion> =
        serde_json::from_str(json_str).unwrap_or_default();

    let count = suggestions.len();
    for s in suggestions {
        let _ = state.events_tx.send(ServerEvent::ConsistencySuggestion {
            source_node_id,
            target_node_id: s.target_node_id,
            original_text: s.original_text,
            suggested_text: s.suggested_text,
            reason: s.reason,
        });
    }

    let _ = state.events_tx.send(ServerEvent::ConsistencyComplete {
        source_node_id,
        suggestion_count: count,
    });
}

#[derive(Deserialize)]
struct RawConsistencySuggestion {
    target_node_id: Uuid,
    original_text: String,
    suggested_text: String,
    reason: String,
}

// ──────────────────────────────────────────────
// Entity Extraction
// ──────────────────────────────────────────────

#[derive(Deserialize)]
struct ExtractBody {
    node_id: Uuid,
}

/// Manual extraction endpoint — awaits the LLM and returns results for user review.
async fn extract_entities(
    State(state): State<AppState>,
    Json(body): Json<ExtractBody>,
) -> Json<serde_json::Value> {
    let node_id = NodeId(body.node_id);

    if !state.extracting.lock().insert(body.node_id) {
        return Json(
            serde_json::json!({ "error": "extraction already in progress for this node" }),
        );
    }

    let result = extract_entities_inner(&state, node_id).await;

    state.extracting.lock().remove(&body.node_id);
    result
}

async fn extract_entities_inner(
    state: &AppState,
    node_id: NodeId,
) -> Json<serde_json::Value> {
    let (script, known_entities, time_ms) = {
        let project_guard = state.project.lock();
        let Some(project) = project_guard.as_ref() else {
            return Json(serde_json::json!({ "error": "no project loaded" }));
        };

        let node = match project.timeline.node(node_id) {
            Ok(n) => n,
            Err(_) => {
                return Json(serde_json::json!({ "error": "node not found" }));
            }
        };

        let script = node.content.content.clone();

        if script.trim().is_empty() {
            return Json(
                serde_json::json!({ "error": "node has no content to extract from" }),
            );
        }

        let time_ms = node.time_range.start_ms + node.time_range.duration_ms() / 2;

        let known: Vec<eidetic_core::story::bible::ResolvedEntity> = project
            .bible
            .entities
            .iter()
            .map(|e| eidetic_core::story::bible::ResolvedEntity {
                entity_id: e.id,
                name: e.name.clone(),
                category: e.category.clone(),
                compact_text: e.to_prompt_text(time_ms),
                full_text: None,
            })
            .collect();

        (script, known, time_ms)
    };

    match run_extraction(state, &script, &known_entities, time_ms).await {
        Some(result) => Json(serde_json::to_value(&result).unwrap()),
        None => Json(serde_json::json!({
            "new_entities": [],
            "snapshot_suggestions": [],
            "entities_present": []
        })),
    }
}

/// Core extraction logic.
async fn run_extraction(
    state: &AppState,
    script: &str,
    known_entities: &[eidetic_core::story::bible::ResolvedEntity],
    time_ms: u64,
) -> Option<eidetic_core::story::bible::ExtractionResult> {
    use crate::prompt_format::build_extraction_prompt;

    let config = state.ai_config.lock().clone();
    let backend = Backend::from_config(&config);
    let prompt = build_extraction_prompt(script, known_entities, time_ms);

    let response = match backend.generate_json(&prompt, &config).await {
        Ok(text) => text,
        Err(e) => {
            tracing::error!("Entity extraction failed: {e}");
            return None;
        }
    };

    let json_str = extract_json_object(&response);
    match serde_json::from_str(json_str) {
        Ok(result) => Some(result),
        Err(e) => {
            tracing::warn!("Failed to parse extraction result: {e}");
            tracing::debug!("Raw extraction response: {response}");
            None
        }
    }
}

/// Auto-extraction after generation — runs extraction and commits results to the bible.
async fn auto_extract_and_commit(state: AppState, node_uuid: Uuid) {
    use eidetic_core::script::element::ScriptElement;
    use eidetic_core::script::format::parse_script_elements;
    use eidetic_core::story::arc::Color;
    use eidetic_core::story::bible::{Entity, EntityCategory, EntitySnapshot, SnapshotOverrides};
    use std::collections::HashSet;

    let node_id = NodeId(node_uuid);

    if !state.extracting.lock().insert(node_uuid) {
        tracing::info!(
            "Skipping auto-extraction for node {node_uuid} — extraction already in progress"
        );
        return;
    }

    let (script, known_entities, time_ms) = {
        let project_guard = state.project.lock();
        let Some(project) = project_guard.as_ref() else {
            state.extracting.lock().remove(&node_uuid);
            return;
        };

        let node = match project.timeline.node(node_id) {
            Ok(n) => n,
            Err(_) => {
                state.extracting.lock().remove(&node_uuid);
                return;
            }
        };

        let script = node.content.content.clone();

        if script.trim().is_empty() {
            state.extracting.lock().remove(&node_uuid);
            return;
        }

        let time_ms = node.time_range.start_ms + node.time_range.duration_ms() / 2;

        let known: Vec<eidetic_core::story::bible::ResolvedEntity> = project
            .bible
            .entities
            .iter()
            .map(|e| eidetic_core::story::bible::ResolvedEntity {
                entity_id: e.id,
                name: e.name.clone(),
                category: e.category.clone(),
                compact_text: e.to_prompt_text(time_ms),
                full_text: None,
            })
            .collect();

        (script, known, time_ms)
    };

    let result = match run_extraction(&state, &script, &known_entities, time_ms).await {
        Some(r) => r,
        None => {
            state.extracting.lock().remove(&node_uuid);
            return;
        }
    };

    let snapshot_count = result.snapshot_suggestions.len();

    if result.new_entities.is_empty()
        && snapshot_count == 0
        && result.entities_present.is_empty()
    {
        tracing::info!(
            "Auto-extraction for node {node_uuid}: LLM returned no entities — skipping commit"
        );
        state.extracting.lock().remove(&node_uuid);
        return;
    }

    fn category_color(cat: &EntityCategory) -> Color {
        match cat {
            EntityCategory::Character => Color::new(100, 149, 237),
            EntityCategory::Location => Color::new(34, 197, 94),
            EntityCategory::Prop => Color::new(249, 115, 22),
            EntityCategory::Theme => Color::new(168, 85, 247),
            EntityCategory::Event => Color::new(239, 68, 68),
        }
    }

    let elements = parse_script_elements(&script);
    let script_characters: HashSet<String> = elements
        .iter()
        .filter_map(|el| match el {
            ScriptElement::Character(name) => {
                let base = if let Some(i) = name.find('(') {
                    name[..i].trim()
                } else {
                    name.trim()
                };
                Some(base.to_lowercase())
            }
            _ => None,
        })
        .collect();
    let script_locations: HashSet<String> = elements
        .iter()
        .filter_map(|el| match el {
            ScriptElement::SceneHeading(h) => Some(h.to_lowercase()),
            _ => None,
        })
        .collect();

    let new_entity_categories: std::collections::HashMap<String, EntityCategory> = result
        .new_entities
        .iter()
        .map(|sug| (sug.name.to_lowercase(), sug.category.clone()))
        .collect();

    let new_count = {
        let mut project_guard = state.project.lock();
        let Some(project) = project_guard.as_mut() else {
            state.extracting.lock().remove(&node_uuid);
            return;
        };

        let mut created = 0usize;

        for sug in &result.new_entities {
            let already_exists = project
                .bible
                .entities
                .iter()
                .any(|e| e.name.eq_ignore_ascii_case(&sug.name));
            if already_exists {
                if let Some(entity) = project
                    .bible
                    .entities
                    .iter_mut()
                    .find(|e| e.name.eq_ignore_ascii_case(&sug.name))
                {
                    if !entity.node_refs.contains(&node_id) {
                        entity.node_refs.push(node_id);
                    }
                }
                continue;
            }
            let mut entity = Entity::new(
                sug.name.clone(),
                sug.category.clone(),
                category_color(&sug.category),
            );
            entity.tagline = sug.tagline.clone();
            entity.description = sug.description.clone();
            entity.node_refs.push(node_id);
            project.bible.entities.push(entity);
            created += 1;
        }

        for sug in &result.snapshot_suggestions {
            let entity = project
                .bible
                .entities
                .iter_mut()
                .find(|e| e.name.eq_ignore_ascii_case(&sug.entity_name));
            if let Some(entity) = entity {
                let overrides = if sug.emotional_state.is_some()
                    || sug.audience_knowledge.is_some()
                    || sug.location.is_some()
                {
                    Some(SnapshotOverrides {
                        emotional_state: sug.emotional_state.clone(),
                        audience_knowledge: sug.audience_knowledge.clone(),
                        location: sug.location.clone(),
                        ..Default::default()
                    })
                } else {
                    None
                };
                entity.add_snapshot(EntitySnapshot {
                    at_ms: time_ms,
                    source_node_id: Some(node_id),
                    description: sug.description.clone(),
                    state_overrides: overrides,
                });
                if !entity.node_refs.contains(&node_id) {
                    entity.node_refs.push(node_id);
                }
            }
        }

        for name in &result.entities_present {
            let found = project
                .bible
                .entities
                .iter_mut()
                .find(|e| e.name.eq_ignore_ascii_case(name));
            if let Some(entity) = found {
                if !entity.node_refs.contains(&node_id) {
                    entity.node_refs.push(node_id);
                }
            } else {
                let cat = new_entity_categories
                    .get(&name.to_lowercase())
                    .cloned()
                    .unwrap_or_else(|| {
                        let lower = name.to_lowercase();
                        if script_locations.contains(&lower)
                            || lower.starts_with("int.")
                            || lower.starts_with("ext.")
                        {
                            EntityCategory::Location
                        } else if script_characters.contains(&lower) {
                            EntityCategory::Character
                        } else {
                            EntityCategory::Prop
                        }
                    });
                let mut entity =
                    Entity::new(name.clone(), cat.clone(), category_color(&cat));
                entity.node_refs.push(node_id);
                project.bible.entities.push(entity);
                created += 1;
            }
        }

        created
    };

    state.extracting.lock().remove(&node_uuid);

    let _ = state.events_tx.send(ServerEvent::BibleChanged);
    let _ = state.events_tx.send(ServerEvent::EntityExtractionComplete {
        node_id: node_uuid,
        new_entity_count: new_count,
        snapshot_count,
    });
    state.trigger_save();

    tracing::info!(
        "Auto-extraction for node {node_uuid}: {new_count} new entities, {snapshot_count} snapshots"
    );
}

// ──────────────────────────────────────────────
// Commit Extraction (manual path)
// ──────────────────────────────────────────────

#[derive(Deserialize)]
struct CommitExtractionBody {
    node_id: Uuid,
    result: eidetic_core::story::bible::ExtractionResult,
    accepted_entities: Vec<bool>,
    accepted_snapshots: Vec<bool>,
}

async fn commit_extraction(
    State(state): State<AppState>,
    Json(body): Json<CommitExtractionBody>,
) -> Json<serde_json::Value> {
    use eidetic_core::script::element::ScriptElement;
    use eidetic_core::script::format::parse_script_elements;
    use eidetic_core::story::arc::Color;
    use eidetic_core::story::bible::{Entity, EntityCategory, EntitySnapshot, SnapshotOverrides};

    fn category_color(cat: &EntityCategory) -> Color {
        match cat {
            EntityCategory::Character => Color::new(100, 149, 237),
            EntityCategory::Location => Color::new(34, 197, 94),
            EntityCategory::Prop => Color::new(249, 115, 22),
            EntityCategory::Theme => Color::new(168, 85, 247),
            EntityCategory::Event => Color::new(239, 68, 68),
        }
    }

    let node_uuid = body.node_id;
    let node_id = NodeId(node_uuid);
    let result = body.result;

    let (script, time_ms) = {
        let project_guard = state.project.lock();
        let Some(project) = project_guard.as_ref() else {
            return Json(serde_json::json!({ "error": "no project loaded" }));
        };
        let node = match project.timeline.node(node_id) {
            Ok(n) => n,
            Err(_) => return Json(serde_json::json!({ "error": "node not found" })),
        };
        let script = node.content.content.clone();
        let time_ms = node.time_range.start_ms + node.time_range.duration_ms() / 2;
        (script, time_ms)
    };

    let elements = parse_script_elements(&script);
    let script_characters: std::collections::HashSet<String> = elements
        .iter()
        .filter_map(|el| match el {
            ScriptElement::Character(name) => {
                let base = if let Some(i) = name.find('(') {
                    name[..i].trim()
                } else {
                    name.trim()
                };
                Some(base.to_lowercase())
            }
            _ => None,
        })
        .collect();
    let script_locations: std::collections::HashSet<String> = elements
        .iter()
        .filter_map(|el| match el {
            ScriptElement::SceneHeading(h) => Some(h.to_lowercase()),
            _ => None,
        })
        .collect();

    let new_entity_categories: std::collections::HashMap<String, EntityCategory> = result
        .new_entities
        .iter()
        .map(|sug| (sug.name.to_lowercase(), sug.category.clone()))
        .collect();

    state.snapshot_for_undo();

    let mut created = 0usize;
    let mut snapshot_count = 0usize;

    {
        let mut project_guard = state.project.lock();
        let Some(project) = project_guard.as_mut() else {
            return Json(serde_json::json!({ "error": "no project loaded" }));
        };

        for (i, sug) in result.new_entities.iter().enumerate() {
            if !body.accepted_entities.get(i).copied().unwrap_or(false) {
                continue;
            }
            let already_exists = project
                .bible
                .entities
                .iter()
                .any(|e| e.name.eq_ignore_ascii_case(&sug.name));
            if already_exists {
                if let Some(entity) = project
                    .bible
                    .entities
                    .iter_mut()
                    .find(|e| e.name.eq_ignore_ascii_case(&sug.name))
                {
                    if !entity.node_refs.contains(&node_id) {
                        entity.node_refs.push(node_id);
                    }
                }
                continue;
            }
            let mut entity = Entity::new(
                sug.name.clone(),
                sug.category.clone(),
                category_color(&sug.category),
            );
            entity.tagline = sug.tagline.clone();
            entity.description = sug.description.clone();
            entity.node_refs.push(node_id);
            project.bible.entities.push(entity);
            created += 1;
        }

        for (i, sug) in result.snapshot_suggestions.iter().enumerate() {
            if !body.accepted_snapshots.get(i).copied().unwrap_or(false) {
                continue;
            }
            let entity = project
                .bible
                .entities
                .iter_mut()
                .find(|e| e.name.eq_ignore_ascii_case(&sug.entity_name));
            if let Some(entity) = entity {
                let overrides = if sug.emotional_state.is_some()
                    || sug.audience_knowledge.is_some()
                    || sug.location.is_some()
                {
                    Some(SnapshotOverrides {
                        emotional_state: sug.emotional_state.clone(),
                        audience_knowledge: sug.audience_knowledge.clone(),
                        location: sug.location.clone(),
                        ..Default::default()
                    })
                } else {
                    None
                };
                entity.add_snapshot(EntitySnapshot {
                    at_ms: time_ms,
                    source_node_id: Some(node_id),
                    description: sug.description.clone(),
                    state_overrides: overrides,
                });
                if !entity.node_refs.contains(&node_id) {
                    entity.node_refs.push(node_id);
                }
                snapshot_count += 1;
            }
        }

        for name in &result.entities_present {
            let found = project
                .bible
                .entities
                .iter_mut()
                .find(|e| e.name.eq_ignore_ascii_case(name));
            if let Some(entity) = found {
                if !entity.node_refs.contains(&node_id) {
                    entity.node_refs.push(node_id);
                }
            } else {
                let cat = new_entity_categories
                    .get(&name.to_lowercase())
                    .cloned()
                    .unwrap_or_else(|| {
                        let lower = name.to_lowercase();
                        if script_locations.contains(&lower)
                            || lower.starts_with("int.")
                            || lower.starts_with("ext.")
                        {
                            EntityCategory::Location
                        } else if script_characters.contains(&lower) {
                            EntityCategory::Character
                        } else {
                            EntityCategory::Prop
                        }
                    });
                let mut entity =
                    Entity::new(name.clone(), cat.clone(), category_color(&cat));
                entity.node_refs.push(node_id);
                project.bible.entities.push(entity);
                created += 1;
            }
        }
    }

    let _ = state.events_tx.send(ServerEvent::BibleChanged);
    let _ = state.events_tx.send(ServerEvent::EntityExtractionComplete {
        node_id: node_uuid,
        new_entity_count: created,
        snapshot_count,
    });
    state.trigger_save();

    Json(serde_json::json!({
        "new_entity_count": created,
        "snapshot_count": snapshot_count,
    }))
}

/// Extract a JSON object from LLM output, handling markdown code fences.
fn extract_json_object(text: &str) -> &str {
    if let Some(start) = text.find("```json") {
        let after_fence = &text[start + 7..];
        if let Some(end) = after_fence.find("```") {
            return after_fence[..end].trim();
        }
    }
    if let Some(start) = text.find("```") {
        let after_fence = &text[start + 3..];
        if let Some(end) = after_fence.find("```") {
            return after_fence[..end].trim();
        }
    }
    if let Some(start) = text.find('{') {
        if let Some(end) = text.rfind('}') {
            return &text[start..=end];
        }
    }
    text.trim()
}

/// Extract the first JSON array from LLM output, handling markdown code fences.
fn extract_json_array(text: &str) -> &str {
    if let Some(start) = text.find("```json") {
        let after_fence = &text[start + 7..];
        if let Some(end) = after_fence.find("```") {
            return after_fence[..end].trim();
        }
    }
    if let Some(start) = text.find("```") {
        let after_fence = &text[start + 3..];
        if let Some(end) = after_fence.find("```") {
            return after_fence[..end].trim();
        }
    }
    if let Some(start) = text.find('[') {
        if let Some(end) = text.rfind(']') {
            return &text[start..=end];
        }
    }
    text.trim()
}
