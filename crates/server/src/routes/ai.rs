use axum::extract::State;
use axum::routing::{get, post, put};
use axum::{Json, Router};
use futures::StreamExt;
use serde::Deserialize;
use uuid::Uuid;

use crate::ai_backends::Backend;
use crate::embeddings::EmbeddingClient;
use crate::prompt_format::build_chat_prompt;
use crate::state::{AppState, BackendType, ServerEvent};
use eidetic_core::ai::backend::RagChunk;
use eidetic_core::ai::prompt::build_generate_request;
use eidetic_core::timeline::clip::{ClipId, ContentStatus};

pub fn router() -> Router<AppState> {
    Router::new()
        .route("/ai/generate", post(generate))
        .route("/ai/react", post(react))
        .route("/ai/extract", post(extract_entities))
        .route("/ai/extract/commit", post(commit_extraction))
        .route("/ai/status", get(status))
        .route("/ai/config", put(config))
}

#[derive(Deserialize)]
struct GenerateBody {
    clip_id: Uuid,
}

async fn generate(
    State(state): State<AppState>,
    Json(body): Json<GenerateBody>,
) -> Json<serde_json::Value> {
    let clip_id = ClipId(body.clip_id);

    // Validate and build the request while holding the project lock briefly.
    let request = {
        let project_guard = state.project.lock();
        let Some(project) = project_guard.as_ref() else {
            return Json(serde_json::json!({ "error": "no project loaded" }));
        };

        // Check clip exists.
        let clip = match project.timeline.clip(clip_id) {
            Ok(c) => c,
            Err(_) => {
                return Json(
                    serde_json::json!({ "error": format!("clip not found: {}", body.clip_id) }),
                );
            }
        };

        // Check clip is not locked.
        if clip.locked {
            return Json(serde_json::json!({ "error": "clip is locked" }));
        }

        // Check clip has beat notes.
        if clip.content.beat_notes.trim().is_empty() {
            return Json(serde_json::json!({ "error": "clip has no beat notes" }));
        }

        // Check not already generating.
        if state.generating.lock().contains(&body.clip_id) {
            return Json(serde_json::json!({ "error": "generation already in progress" }));
        }

        match build_generate_request(project, clip_id) {
            Ok(req) => req,
            Err(e) => {
                return Json(serde_json::json!({ "error": e.to_string() }));
            }
        }
    };

    // Mark as generating.
    state.generating.lock().insert(body.clip_id);

    // Update clip status to Generating.
    {
        let mut project_guard = state.project.lock();
        if let Some(project) = project_guard.as_mut() {
            if let Ok(clip) = project.timeline.clip_mut(clip_id) {
                clip.content.status = ContentStatus::Generating;
            }
        }
    }
    let _ = state
        .events_tx
        .send(ServerEvent::BeatUpdated { clip_id: body.clip_id });

    // Spawn the generation task.
    let state_clone = state.clone();
    let clip_uuid = body.clip_id;
    tokio::spawn(async move {
        run_generation(state_clone, clip_uuid, request).await;
    });

    Json(serde_json::json!({
        "status": "started",
        "clip_id": body.clip_id.to_string(),
    }))
}

async fn run_generation(
    state: AppState,
    clip_id: Uuid,
    mut request: eidetic_core::ai::backend::GenerateRequest,
) {
    let config = state.ai_config.lock().clone();
    let backend = Backend::from_config(&config);

    // RAG: retrieve relevant reference chunks if vector store has entries.
    if !state.vector_store.lock().is_empty() {
        let query = &request.beat_clip.content.beat_notes;
        let embed_client = EmbeddingClient::new(&config.base_url, crate::state::constants::EMBEDDING_MODEL);
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

    // Broadcast the formatted prompt context to connected clients.
    let _ = state.events_tx.send(ServerEvent::GenerationContext {
        clip_id,
        system_prompt: prompt.system.clone(),
        user_prompt: prompt.user.clone(),
    });

    let stream_result = backend.generate(&prompt, &config).await;

    let mut stream = match stream_result {
        Ok(s) => s,
        Err(e) => {
            tracing::error!("AI generation failed for clip {clip_id}: {e}");
            // Revert clip status.
            {
                let mut project_guard = state.project.lock();
                if let Some(project) = project_guard.as_mut() {
                    if let Ok(clip) = project.timeline.clip_mut(ClipId(clip_id)) {
                        clip.content.status = ContentStatus::NotesOnly;
                    }
                }
            }
            let _ = state.events_tx.send(ServerEvent::GenerationError {
                clip_id,
                error: e.to_string(),
            });
            state.generating.lock().remove(&clip_id);
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
                    clip_id,
                    token,
                    tokens_generated,
                });
            }
            Err(e) => {
                tracing::warn!("Stream error during generation for clip {clip_id}: {e}");
                break;
            }
        }
    }

    // Store the result.
    if full_text.is_empty() {
        // No output produced — revert status.
        {
            let mut project_guard = state.project.lock();
            if let Some(project) = project_guard.as_mut() {
                if let Ok(clip) = project.timeline.clip_mut(ClipId(clip_id)) {
                    clip.content.status = ContentStatus::NotesOnly;
                }
            }
        }
        let _ = state.events_tx.send(ServerEvent::GenerationError {
            clip_id,
            error: "AI produced no output".into(),
        });
    } else {
        // Store generated script and update status.
        {
            let mut project_guard = state.project.lock();
            if let Some(project) = project_guard.as_mut() {
                if let Ok(clip) = project.timeline.clip_mut(ClipId(clip_id)) {
                    clip.content.generated_script = Some(full_text.clone());
                    clip.content.status = ContentStatus::Generated;
                }
            }
        }
        let _ = state
            .events_tx
            .send(ServerEvent::GenerationComplete { clip_id });
        let _ = state
            .events_tx
            .send(ServerEvent::BeatUpdated { clip_id });
        state.trigger_save();

        // Generate scene recap for continuity context (non-fatal).
        generate_scene_recap(&state, clip_id, &full_text).await;

        // Auto-extract entities from the newly generated script.
        let extract_state = state.clone();
        tokio::spawn(async move {
            auto_extract_and_commit(extract_state, clip_id).await;
        });
    }

    state.generating.lock().remove(&clip_id);
}

/// Generate a compact scene recap after script generation.
///
/// This is a lightweight AI call (~100-200 output tokens) that captures
/// the scene's end state for use as continuity context in subsequent
/// generations. Failures are logged but do not block the main flow.
async fn generate_scene_recap(state: &AppState, clip_id: Uuid, script: &str) {
    use crate::prompt_format::build_recap_prompt;

    // Find the preceding clip's recap for rolling summary behavior.
    let preceding_recap = {
        let project_guard = state.project.lock();
        let Some(project) = project_guard.as_ref() else {
            return;
        };

        let track = match project.timeline.track_for_clip(ClipId(clip_id)) {
            Some(t) => t,
            None => return,
        };

        let idx = track.clips.iter().position(|c| c.id.0 == clip_id);
        idx.and_then(|i| {
            if i > 0 {
                track.clips[i - 1].content.scene_recap.clone()
            } else {
                None
            }
        })
    };

    let config = state.ai_config.lock().clone();
    let backend = Backend::from_config(&config);

    // Use reduced max_tokens for the recap to save compute.
    let mut recap_config = config.clone();
    recap_config.max_tokens = 512;

    let prompt = build_recap_prompt(script, preceding_recap.as_deref());

    let recap_text = match backend.generate_full(&prompt, &recap_config).await {
        Ok(text) => text.trim().to_string(),
        Err(e) => {
            tracing::warn!("Scene recap generation failed for clip {clip_id}: {e}");
            return;
        }
    };

    if recap_text.is_empty() {
        tracing::warn!("Scene recap was empty for clip {clip_id}");
        return;
    }

    // Store the recap on the clip.
    {
        let mut project_guard = state.project.lock();
        if let Some(project) = project_guard.as_mut() {
            if let Ok(clip) = project.timeline.clip_mut(ClipId(clip_id)) {
                clip.content.scene_recap = Some(recap_text);
            }
        }
    }

    let _ = state
        .events_tx
        .send(ServerEvent::BeatUpdated { clip_id });
    state.trigger_save();

    tracing::info!("Scene recap generated for clip {clip_id}");
}

async fn status(State(state): State<AppState>) -> Json<serde_json::Value> {
    let config = state.ai_config.lock().clone();
    let backend = Backend::from_config(&config);

    match backend.health_check().await {
        Ok(status) => Json(serde_json::json!({
            "backend": config.backend_type,
            "model": config.model,
            "connected": status.connected,
            "message": status.message,
        })),
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

#[derive(Deserialize)]
struct ReactBody {
    clip_id: Uuid,
}

async fn react(
    State(state): State<AppState>,
    Json(body): Json<ReactBody>,
) -> Json<serde_json::Value> {
    let clip_id = ClipId(body.clip_id);

    // Gather edit context and downstream beats while holding lock.
    let (edit_context, downstream_info) = {
        let project_guard = state.project.lock();
        let Some(project) = project_guard.as_ref() else {
            return Json(serde_json::json!({ "error": "no project loaded" }));
        };

        let edit_ctx = match eidetic_core::ai::consistency::build_edit_context(project, clip_id) {
            Ok(ctx) => ctx,
            Err(e) => return Json(serde_json::json!({ "error": e.to_string() })),
        };

        let downstream_ids = eidetic_core::ai::consistency::downstream_clip_ids(project, clip_id);
        let mut downstream: Vec<(Uuid, String, String)> = Vec::new();
        for did in downstream_ids {
            if let Ok(clip) = project.timeline.clip(did) {
                let script = clip
                    .content
                    .user_refined_script
                    .as_ref()
                    .or(clip.content.generated_script.as_ref())
                    .cloned()
                    .unwrap_or_default();
                if !script.is_empty() {
                    downstream.push((did.0, clip.name.clone(), script));
                }
            }
        }

        (edit_ctx, downstream)
    };

    if downstream_info.is_empty() {
        return Json(serde_json::json!({ "status": "no_downstream_beats" }));
    }

    // Spawn the consistency check task.
    let state_clone = state.clone();
    let source_clip_id = body.clip_id;
    tokio::spawn(async move {
        run_consistency_check(state_clone, source_clip_id, edit_context, downstream_info).await;
    });

    Json(serde_json::json!({ "status": "checking" }))
}

async fn run_consistency_check(
    state: AppState,
    source_clip_id: Uuid,
    edit_context: eidetic_core::ai::backend::EditContext,
    downstream_beats: Vec<(Uuid, String, String)>,
) {
    use crate::prompt_format::build_consistency_prompt;

    let config = state.ai_config.lock().clone();
    let backend = Backend::from_config(&config);
    let prompt = build_consistency_prompt(&edit_context, &downstream_beats);

    let response = match backend.generate_full(&prompt, &config).await {
        Ok(text) => text,
        Err(e) => {
            tracing::error!("Consistency check failed for clip {source_clip_id}: {e}");
            let _ = state.events_tx.send(ServerEvent::ConsistencyComplete {
                source_clip_id,
                suggestion_count: 0,
            });
            return;
        }
    };

    // Parse JSON from the response — the LLM may wrap it in markdown code fences.
    let json_str = extract_json_array(&response);
    let suggestions: Vec<RawConsistencySuggestion> =
        serde_json::from_str(json_str).unwrap_or_default();

    let count = suggestions.len();
    for s in suggestions {
        let _ = state.events_tx.send(ServerEvent::ConsistencySuggestion {
            source_clip_id,
            target_clip_id: s.target_clip_id,
            original_text: s.original_text,
            suggested_text: s.suggested_text,
            reason: s.reason,
        });
    }

    let _ = state.events_tx.send(ServerEvent::ConsistencyComplete {
        source_clip_id,
        suggestion_count: count,
    });
}

#[derive(Deserialize)]
struct RawConsistencySuggestion {
    target_clip_id: Uuid,
    original_text: String,
    suggested_text: String,
    reason: String,
}

// ──────────────────────────────────────────────
// Entity Extraction
// ──────────────────────────────────────────────

#[derive(Deserialize)]
struct ExtractBody {
    clip_id: Uuid,
}

/// Manual extraction endpoint — awaits the LLM and returns results for user review.
async fn extract_entities(
    State(state): State<AppState>,
    Json(body): Json<ExtractBody>,
) -> Json<serde_json::Value> {
    let clip_id = ClipId(body.clip_id);

    // Prevent concurrent extractions on the same clip.
    if !state.extracting.lock().insert(body.clip_id) {
        return Json(serde_json::json!({ "error": "extraction already in progress for this clip" }));
    }

    let result = extract_entities_inner(&state, clip_id).await;

    state.extracting.lock().remove(&body.clip_id);
    result
}

async fn extract_entities_inner(
    state: &AppState,
    clip_id: ClipId,
) -> Json<serde_json::Value> {
    // Gather the script and known entities while holding the lock briefly.
    let (script, known_entities, time_ms) = {
        let project_guard = state.project.lock();
        let Some(project) = project_guard.as_ref() else {
            return Json(serde_json::json!({ "error": "no project loaded" }));
        };

        let clip = match project.timeline.clip(clip_id) {
            Ok(c) => c,
            Err(_) => {
                return Json(serde_json::json!({ "error": "clip not found" }));
            }
        };

        let script = clip
            .content
            .user_refined_script
            .as_ref()
            .or(clip.content.generated_script.as_ref())
            .cloned()
            .unwrap_or_default();

        if script.trim().is_empty() {
            return Json(serde_json::json!({ "error": "clip has no script to extract from" }));
        }

        let time_ms = clip.time_range.start_ms + clip.time_range.duration_ms() / 2;

        // Build resolved entities list for the extraction prompt.
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

    // Run extraction synchronously — await the LLM response.
    match run_extraction(state, &script, &known_entities, time_ms).await {
        Some(result) => Json(serde_json::to_value(&result).unwrap()),
        None => Json(serde_json::json!({
            "new_entities": [],
            "snapshot_suggestions": [],
            "entities_present": []
        })),
    }
}

/// Core extraction logic — calls the LLM with JSON mode and parses the result.
/// Returns `None` if extraction fails.
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

    // Use JSON mode so the model is constrained to produce valid JSON.
    let response = match backend.generate_json(&prompt, &config).await {
        Ok(text) => text,
        Err(e) => {
            tracing::error!("Entity extraction failed: {e}");
            return None;
        }
    };

    // With JSON mode the response should be clean, but still strip fences
    // as a safety net.
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
async fn auto_extract_and_commit(state: AppState, clip_id: Uuid) {
    use eidetic_core::story::bible::{
        Entity, EntityCategory, EntitySnapshot, SnapshotOverrides,
    };
    use eidetic_core::story::arc::Color;

    // Prevent concurrent extractions on the same clip (manual + auto race).
    if !state.extracting.lock().insert(clip_id) {
        tracing::info!("Skipping auto-extraction for clip {clip_id} — extraction already in progress");
        return;
    }

    // Gather context from the project.
    let (script, known_entities, time_ms) = {
        let project_guard = state.project.lock();
        let Some(project) = project_guard.as_ref() else {
            state.extracting.lock().remove(&clip_id);
            return;
        };

        let clip = match project.timeline.clip(ClipId(clip_id)) {
            Ok(c) => c,
            Err(_) => {
                state.extracting.lock().remove(&clip_id);
                return;
            }
        };

        let script = clip
            .content
            .user_refined_script
            .as_ref()
            .or(clip.content.generated_script.as_ref())
            .cloned()
            .unwrap_or_default();

        if script.trim().is_empty() {
            state.extracting.lock().remove(&clip_id);
            return;
        }

        let time_ms = clip.time_range.start_ms + clip.time_range.duration_ms() / 2;

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

    // Run extraction.
    let result = match run_extraction(&state, &script, &known_entities, time_ms).await {
        Some(r) => r,
        None => {
            state.extracting.lock().remove(&clip_id);
            return;
        }
    };

    let snapshot_count = result.snapshot_suggestions.len();

    if result.new_entities.is_empty() && snapshot_count == 0 && result.entities_present.is_empty() {
        tracing::info!(
            "Auto-extraction for clip {clip_id}: LLM returned no entities, snapshots, or present names — skipping commit"
        );
        state.extracting.lock().remove(&clip_id);
        return;
    }

    // Default colors per category.
    fn category_color(cat: &EntityCategory) -> Color {
        match cat {
            EntityCategory::Character => Color { r: 100, g: 149, b: 237 }, // cornflower blue
            EntityCategory::Location => Color { r: 34, g: 197, b: 94 },    // green
            EntityCategory::Prop => Color { r: 249, g: 115, b: 22 },       // orange
            EntityCategory::Theme => Color { r: 168, g: 85, b: 247 },      // purple
            EntityCategory::Event => Color { r: 239, g: 68, b: 68 },       // red
        }
    }

    // Parse the script to identify character cues and scene headings for
    // category inference.  LLMs often put entities only in `entities_present`
    // (bare names, no category) rather than `new_entities`, so we need to be
    // able to infer categories from the screenplay structure.
    use eidetic_core::script::format::parse_script_elements;
    use eidetic_core::script::element::ScriptElement;
    use std::collections::HashSet;

    let elements = parse_script_elements(&script);
    let script_characters: HashSet<String> = elements
        .iter()
        .filter_map(|el| match el {
            ScriptElement::Character(name) => {
                // Strip parenthetical extensions like "(V.O.)" or "(CONT'D)".
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

    // Build a category lookup from new_entities (LLM-assigned categories).
    let new_entity_categories: std::collections::HashMap<String, EntityCategory> = result
        .new_entities
        .iter()
        .map(|sug| (sug.name.to_lowercase(), sug.category.clone()))
        .collect();

    // Commit results to the bible.
    let new_count = {
        let mut project_guard = state.project.lock();
        let Some(project) = project_guard.as_mut() else {
            state.extracting.lock().remove(&clip_id);
            return;
        };

        let clip_id_typed = ClipId(clip_id);
        let mut created = 0usize;

        // Create new entities — skip duplicates (case-insensitive).
        for sug in &result.new_entities {
            let already_exists = project.bible.entities.iter().any(|e| {
                e.name.eq_ignore_ascii_case(&sug.name)
            });
            if already_exists {
                tracing::debug!("Skipping duplicate entity '{}' — already in bible", sug.name);
                // Still add clip ref to the existing entity.
                if let Some(entity) = project.bible.entities.iter_mut().find(|e| {
                    e.name.eq_ignore_ascii_case(&sug.name)
                }) {
                    if !entity.clip_refs.contains(&clip_id_typed) {
                        entity.clip_refs.push(clip_id_typed);
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
            entity.clip_refs.push(clip_id_typed);
            project.bible.entities.push(entity);
            created += 1;
        }

        // Apply snapshot suggestions to matching entities.
        for sug in &result.snapshot_suggestions {
            let entity = project.bible.entities.iter_mut().find(|e| {
                e.name.eq_ignore_ascii_case(&sug.entity_name)
            });
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
                    source_clip_id: Some(clip_id_typed),
                    description: sug.description.clone(),
                    state_overrides: overrides,
                });
                // Also add clip ref if not already present.
                if !entity.clip_refs.contains(&clip_id_typed) {
                    entity.clip_refs.push(clip_id_typed);
                }
            } else {
                tracing::warn!(
                    "Snapshot suggestion for '{}' has no matching bible entity — skipped",
                    sug.entity_name,
                );
            }
        }

        // Add clip refs for all entities present in the scene (resolved by name).
        // If a name doesn't match any bible entity (including ones just created
        // from new_entities), create the entity with a category inferred from
        // the screenplay structure.
        for name in &result.entities_present {
            let found = project.bible.entities.iter_mut().find(|e| {
                e.name.eq_ignore_ascii_case(name)
            });
            if let Some(entity) = found {
                if !entity.clip_refs.contains(&clip_id_typed) {
                    entity.clip_refs.push(clip_id_typed);
                }
            } else {
                // Infer category: check new_entities first, then screenplay structure.
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
                            // Not a character cue or scene heading → Prop.
                            EntityCategory::Prop
                        }
                    });
                tracing::info!(
                    "Creating entity '{}' [{}] from entities_present (missing from new_entities)",
                    name, cat
                );
                let mut entity = Entity::new(
                    name.clone(),
                    cat.clone(),
                    category_color(&cat),
                );
                entity.clip_refs.push(clip_id_typed);
                project.bible.entities.push(entity);
                created += 1;
            }
        }

        created
    };

    state.extracting.lock().remove(&clip_id);

    // Notify frontend.
    let _ = state.events_tx.send(ServerEvent::BibleChanged);
    let _ = state.events_tx.send(ServerEvent::EntityExtractionComplete {
        clip_id,
        new_entity_count: new_count,
        snapshot_count,
    });
    state.trigger_save();

    tracing::info!(
        "Auto-extraction for clip {clip_id}: {new_count} new entities, {snapshot_count} snapshots — committed to bible"
    );
}

// ──────────────────────────────────────────────
// Commit Extraction (manual path)
// ──────────────────────────────────────────────

/// Request body for committing manually-reviewed extraction results.
#[derive(Deserialize)]
struct CommitExtractionBody {
    clip_id: Uuid,
    result: eidetic_core::story::bible::ExtractionResult,
    /// Indices of `new_entities` the user accepted (true = accepted).
    accepted_entities: Vec<bool>,
    /// Indices of `snapshot_suggestions` the user accepted.
    accepted_snapshots: Vec<bool>,
}

/// Server-side commit of extraction results — handles dedup, category inference,
/// and entity creation using the same logic as auto-extraction.
async fn commit_extraction(
    State(state): State<AppState>,
    Json(body): Json<CommitExtractionBody>,
) -> Json<serde_json::Value> {
    use eidetic_core::story::bible::{
        Entity, EntityCategory, EntitySnapshot, SnapshotOverrides,
    };
    use eidetic_core::story::arc::Color;
    use eidetic_core::script::format::parse_script_elements;
    use eidetic_core::script::element::ScriptElement;

    fn category_color(cat: &EntityCategory) -> Color {
        match cat {
            EntityCategory::Character => Color { r: 100, g: 149, b: 237 },
            EntityCategory::Location => Color { r: 34, g: 197, b: 94 },
            EntityCategory::Prop => Color { r: 249, g: 115, b: 22 },
            EntityCategory::Theme => Color { r: 168, g: 85, b: 247 },
            EntityCategory::Event => Color { r: 239, g: 68, b: 68 },
        }
    }

    let clip_id = body.clip_id;
    let clip_id_typed = ClipId(clip_id);
    let result = body.result;

    // Get the script text for category inference.
    let (script, time_ms) = {
        let project_guard = state.project.lock();
        let Some(project) = project_guard.as_ref() else {
            return Json(serde_json::json!({ "error": "no project loaded" }));
        };
        let clip = match project.timeline.clip(clip_id_typed) {
            Ok(c) => c,
            Err(_) => return Json(serde_json::json!({ "error": "clip not found" })),
        };
        let script = clip
            .content
            .user_refined_script
            .as_ref()
            .or(clip.content.generated_script.as_ref())
            .cloned()
            .unwrap_or_default();
        let time_ms = clip.time_range.start_ms + clip.time_range.duration_ms() / 2;
        (script, time_ms)
    };

    // Parse screenplay structure for category inference.
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

        // Create accepted new entities — skip duplicates.
        for (i, sug) in result.new_entities.iter().enumerate() {
            if !body.accepted_entities.get(i).copied().unwrap_or(false) {
                continue;
            }
            let already_exists = project.bible.entities.iter().any(|e| {
                e.name.eq_ignore_ascii_case(&sug.name)
            });
            if already_exists {
                if let Some(entity) = project.bible.entities.iter_mut().find(|e| {
                    e.name.eq_ignore_ascii_case(&sug.name)
                }) {
                    if !entity.clip_refs.contains(&clip_id_typed) {
                        entity.clip_refs.push(clip_id_typed);
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
            entity.clip_refs.push(clip_id_typed);
            project.bible.entities.push(entity);
            created += 1;
        }

        // Apply accepted snapshots.
        for (i, sug) in result.snapshot_suggestions.iter().enumerate() {
            if !body.accepted_snapshots.get(i).copied().unwrap_or(false) {
                continue;
            }
            let entity = project.bible.entities.iter_mut().find(|e| {
                e.name.eq_ignore_ascii_case(&sug.entity_name)
            });
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
                    source_clip_id: Some(clip_id_typed),
                    description: sug.description.clone(),
                    state_overrides: overrides,
                });
                if !entity.clip_refs.contains(&clip_id_typed) {
                    entity.clip_refs.push(clip_id_typed);
                }
                snapshot_count += 1;
            }
        }

        // Create entities for unmatched entities_present names.
        for name in &result.entities_present {
            let found = project.bible.entities.iter_mut().find(|e| {
                e.name.eq_ignore_ascii_case(name)
            });
            if let Some(entity) = found {
                if !entity.clip_refs.contains(&clip_id_typed) {
                    entity.clip_refs.push(clip_id_typed);
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
                let mut entity = Entity::new(
                    name.clone(),
                    cat.clone(),
                    category_color(&cat),
                );
                entity.clip_refs.push(clip_id_typed);
                project.bible.entities.push(entity);
                created += 1;
            }
        }
    }

    let _ = state.events_tx.send(ServerEvent::BibleChanged);
    let _ = state.events_tx.send(ServerEvent::EntityExtractionComplete {
        clip_id,
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
    // Try to find content within ```json ... ``` fences.
    if let Some(start) = text.find("```json") {
        let after_fence = &text[start + 7..];
        if let Some(end) = after_fence.find("```") {
            return after_fence[..end].trim();
        }
    }
    // Try plain ``` fences.
    if let Some(start) = text.find("```") {
        let after_fence = &text[start + 3..];
        if let Some(end) = after_fence.find("```") {
            return after_fence[..end].trim();
        }
    }
    // Try to find a bare JSON object.
    if let Some(start) = text.find('{') {
        if let Some(end) = text.rfind('}') {
            return &text[start..=end];
        }
    }
    text.trim()
}

/// Extract the first JSON array from LLM output, handling markdown code fences.
fn extract_json_array(text: &str) -> &str {
    // Try to find content within ```json ... ``` fences.
    if let Some(start) = text.find("```json") {
        let after_fence = &text[start + 7..];
        if let Some(end) = after_fence.find("```") {
            return after_fence[..end].trim();
        }
    }
    // Try plain ``` fences.
    if let Some(start) = text.find("```") {
        let after_fence = &text[start + 3..];
        if let Some(end) = after_fence.find("```") {
            return after_fence[..end].trim();
        }
    }
    // Try to find a bare JSON array.
    if let Some(start) = text.find('[') {
        if let Some(end) = text.rfind(']') {
            return &text[start..=end];
        }
    }
    text.trim()
}

