use std::path::PathBuf;

use eidetic_core::ai::backend::{GenerateRequest, RagChunk};
use eidetic_core::contracts::{
    CommandEnvelope, CommandId, GenerateScriptBlockCommand, ScriptBlockId, ScriptBlockKind,
    ScriptContextBlock, ScriptDocumentId, ScriptSegmentId, ScriptSegmentStatus,
    ScriptSpanProvenance, SetScriptBlockCommand,
};
use eidetic_core::timeline::node::{ContentStatus, NodeId};
use futures::StreamExt;
use uuid::Uuid;

use crate::ai_backends::Backend;
use crate::embeddings::{Embedding, EmbeddingClient};
use crate::prompt_format::build_chat_prompt;
use crate::script_document_command;
use crate::state::{AppState, ServerEvent};
use crate::timeline_node_store;

use crate::ai_service::active_sqlite_project;

#[derive(Default)]
struct GenerationInputs {
    script_inputs: Option<Vec<ScriptContextBlock>>,
    bible_inputs: Option<Vec<eidetic_core::contracts::BibleFieldInput>>,
    bible_relationship_inputs: Option<Vec<eidetic_core::contracts::BibleRelationshipInput>>,
    bible_context_scope: Option<eidetic_core::contracts::BibleContextScope>,
    script_context_scope: Option<eidetic_core::contracts::ScriptContextScope>,
    target_binding: Option<eidetic_core::contracts::ScriptGenerationTarget>,
    session_id: Option<Uuid>,
}

pub(crate) async fn mark_node_generating(
    state: &AppState,
    project_path: PathBuf,
    node_id: NodeId,
    node_uuid: Uuid,
) {
    if let Err(error) =
        persist_node_content_status(project_path, node_id, ContentStatus::Generating).await
    {
        tracing::warn!("Failed to persist generating status for node {node_uuid}: {error}");
    }
    {
        let mut project_guard = state.project.lock();
        if let Some(project) = project_guard.as_mut()
            && let Ok(node) = project.timeline.node_mut(node_id)
        {
            node.content.status = ContentStatus::Generating;
        }
    }
    let _ = state
        .events_tx
        .send(ServerEvent::NodeUpdated { node_id: node_uuid });
}

pub(crate) async fn run_generation(
    state: AppState,
    project_path: PathBuf,
    node_uuid: Uuid,
    mut request: GenerateRequest,
    retrieval_scope: Uuid,
    session_id: Uuid,
) {
    let node_id = NodeId(node_uuid);
    let config = state.ai_config.lock().clone();
    let backend = Backend::from_config(&config);

    attach_rag_context(
        &state,
        &config,
        &project_path,
        retrieval_scope,
        &mut request,
    )
    .await;
    let prompt = build_chat_prompt(&request);

    let _ = state.events_tx.send(ServerEvent::GenerationContext {
        node_id: node_uuid,
        system_prompt: prompt.system.clone(),
        user_prompt: prompt.user.clone(),
    });

    let stream = match backend.generate(&prompt, &config).await {
        Ok(stream) => stream,
        Err(error) => {
            handle_generation_failure(&state, project_path, node_id, node_uuid, error.to_string())
                .await;
            return;
        }
    };

    finish_generation_stream(
        state,
        project_path,
        node_uuid,
        stream,
        GenerationInputs {
            script_inputs: request.script_context,
            bible_relationship_inputs: request.bible_relationship_inputs,
            bible_inputs: request.bible_inputs,
            bible_context_scope: request.bible_context_scope,
            script_context_scope: request.script_context_scope,
            target_binding: request.generation_target,
            session_id: Some(session_id),
        },
    )
    .await;
}

async fn finish_generation_stream(
    state: AppState,
    project_path: PathBuf,
    node_uuid: Uuid,
    stream: eidetic_core::ai::backend::GenerateStream,
    inputs: GenerationInputs,
) {
    let node_id = NodeId(node_uuid);
    let full_text = match stream_generated_text(stream, |token, tokens_generated| {
        let _ = state.events_tx.send(ServerEvent::GenerationProgress {
            node_id: node_uuid,
            token,
            tokens_generated,
        });
    })
    .await
    {
        Ok(text) => text,
        Err(error) => {
            handle_generation_failure(&state, project_path, node_id, node_uuid, error.to_string())
                .await;
            return;
        }
    };
    if full_text.is_empty() {
        handle_empty_generation(&state, project_path, node_id, node_uuid).await;
        return;
    }

    persist_successful_generation(state, project_path, node_uuid, full_text, inputs).await;
}

async fn attach_rag_context(
    state: &AppState,
    config: &crate::state::AiConfig,
    project_path: &std::path::Path,
    scope: Uuid,
    request: &mut GenerateRequest,
) {
    request.rag_context.clear();
    {
        let store = state.vector_store.lock();
        if scope != store.scope() || store.is_empty() {
            return;
        }
    }
    let embed_client =
        EmbeddingClient::new(&config.base_url, crate::state::constants::EMBEDDING_MODEL);
    match embed_client.embed(&request.target_node.content.notes).await {
        Ok(query_embedding) => {
            attach_rag_embedding(state, project_path, scope, &query_embedding, request)
        }
        Err(_) => tracing::warn!("Reference retrieval unavailable: query embedding failed"),
    }
}

/// Final publication boundary after external model I/O. The request carries the
/// epoch captured at admission, rather than choosing a new epoch when scheduled.
fn attach_rag_embedding(
    state: &AppState,
    project_path: &std::path::Path,
    scope: Uuid,
    query_embedding: &Embedding,
    request: &mut GenerateRequest,
) {
    request.rag_context.clear();
    let config = state.ai_config.lock().clone();
    let current_client =
        EmbeddingClient::new(&config.base_url, crate::state::constants::EMBEDDING_MODEL);
    if query_embedding.identity != current_client.identity() {
        return;
    }
    let guard = state.project.lock();
    let Some(project) = guard.as_ref() else {
        return;
    };
    if state.project_database.active_path().as_deref() != Some(project_path) {
        return;
    }
    let store = state.vector_store.lock();
    request.rag_context = store
        .search(
            scope,
            &project.references,
            query_embedding,
            crate::state::constants::RAG_TOP_K,
        )
        .into_iter()
        .map(|(chunk, score)| RagChunk {
            source: chunk.document_name.clone(),
            content: chunk.content.clone(),
            relevance_score: score,
        })
        .collect();
}

async fn stream_generated_text(
    mut stream: eidetic_core::ai::backend::GenerateStream,
    mut on_token: impl FnMut(String, usize),
) -> Result<String, eidetic_core::Error> {
    let mut full_text = String::new();
    let mut tokens_generated: usize = 0;

    while let Some(item) = stream.next().await {
        let token = item?;
        full_text.push_str(&token);
        tokens_generated += 1;
        on_token(token, tokens_generated);
    }
    Ok(full_text)
}

async fn handle_generation_failure(
    state: &AppState,
    project_path: PathBuf,
    node_id: NodeId,
    node_uuid: Uuid,
    error: String,
) {
    tracing::error!("AI generation failed for node {node_uuid}: {error}");
    restore_generation_status(state, project_path, node_id, node_uuid).await;
    let _ = state.events_tx.send(ServerEvent::GenerationError {
        node_id: node_uuid,
        error,
    });
    state.generating.lock().remove(&node_uuid);
}

async fn handle_empty_generation(
    state: &AppState,
    project_path: PathBuf,
    node_id: NodeId,
    node_uuid: Uuid,
) {
    restore_generation_status(state, project_path, node_id, node_uuid).await;
    let _ = state.events_tx.send(ServerEvent::GenerationError {
        node_id: node_uuid,
        error: "AI produced no output".into(),
    });
    state.generating.lock().remove(&node_uuid);
}

async fn restore_generation_status(
    state: &AppState,
    path: PathBuf,
    node_id: NodeId,
    node_uuid: Uuid,
) {
    // Failed/refused output cannot demote an existing saved screenplay to NotesOnly.
    let result = tokio::task::spawn_blocking(move || {
        let mut conn = crate::sqlite::open_write_connection(&path)?;
        let tx = conn.transaction()?;
        let Some(node) = timeline_node_store::load_node_ancestor_stack(&tx, node_id)?
            .into_iter().find(|node| node.id == node_id) else { return Ok(None); };
        let saved: bool = tx.query_row(
            "SELECT EXISTS(SELECT 1 FROM script_segments s JOIN script_blocks b ON b.segment_id=s.id
             WHERE s.source_node_id=?1 AND s.document_id='script.document.main' AND s.deleted_event_id IS NULL AND b.deleted_event_id IS NULL)",
            [node_id.0.to_string()], |row| row.get(0),
        )?;
        let status = if saved { ContentStatus::HasContent } else if node.content.notes.is_empty() {
            ContentStatus::Empty
        } else { ContentStatus::NotesOnly };
        timeline_node_store::update_node_content_status(&tx, node_id, status)?;
        tx.commit()?;
        Ok::<_, crate::history_store::HistoryStoreError>(Some(status))
    }).await;
    match result {
        Ok(Ok(Some(status))) => {
            set_project_node_status(state, node_id, status);
        }
        Ok(Ok(None)) => {}
        other => tracing::warn!("Failed to restore generation status for {node_uuid}: {other:?}"),
    }
    let _ = state
        .events_tx
        .send(ServerEvent::NodeUpdated { node_id: node_uuid });
}

async fn persist_successful_generation(
    state: AppState,
    project_path: PathBuf,
    node_uuid: Uuid,
    full_text: String,
    inputs: GenerationInputs,
) {
    let node_id = NodeId(node_uuid);
    // External I/O holds no session lock. Completion keeps the existing gate
    // through canonical persistence/publication and refuses a replaced session.
    let session = state.project_session_gate.clone().lock_owned().await;
    if state.project_database.active_path().as_ref() != Some(&project_path)
        || inputs
            .session_id
            .is_some_and(|id| id != *state.project_session_id.lock())
    {
        let _ = state.events_tx.send(ServerEvent::GenerationError {
            node_id: node_uuid,
            error: "generation project session changed; saved screenplay was preserved".into(),
        });
        state.generating.lock().remove(&node_uuid);
        return;
    }
    if let Err(error) =
        persist_generated_script_block(project_path.clone(), node_uuid, full_text.clone(), inputs)
            .await
    {
        handle_generation_failure(&state, project_path, node_id, node_uuid, error).await;
        return;
    }
    set_project_node_status(&state, node_id, ContentStatus::HasContent);
    let _ = state
        .events_tx
        .send(ServerEvent::GenerationComplete { node_id: node_uuid });
    let _ = state
        .events_tx
        .send(ServerEvent::NodeUpdated { node_id: node_uuid });
    let _ = state.events_tx.send(ServerEvent::ScriptChanged);
    state.trigger_save();
    drop(session);
    generate_scene_recap(&state, node_uuid, &full_text).await;
    state.generating.lock().remove(&node_uuid);
}

fn set_project_node_status(state: &AppState, node_id: NodeId, status: ContentStatus) {
    let mut project_guard = state.project.lock();
    if let Some(project) = project_guard.as_mut()
        && let Ok(node) = project.timeline.node_mut(node_id)
    {
        node.content.status = status;
    }
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
    full_text: String,
    inputs: GenerationInputs,
) -> Result<(), String> {
    tokio::task::spawn_blocking(move || {
        let mut conn = crate::sqlite::open_write_connection(&project_path)
            .map_err(|error| error.to_string())?;
        let node = crate::timeline_node_store::load_node_ancestor_stack(&conn, NodeId(node_uuid))
            .map_err(|error| error.to_string())?
            .into_iter()
            .find(|node| node.id.0 == node_uuid)
            .ok_or_else(|| "generation target no longer exists".to_string())?;
        let metadata = GeneratedScriptMetadata {
            project_name: conn
                .query_row("SELECT name FROM project WHERE id=1", [], |row| row.get(0))
                .map_err(|error| error.to_string())?,
            start_ms: node.time_range.start_ms,
            end_ms: node.time_range.end_ms,
        };
        let command =
            generated_script_block_command(Uuid::new_v4(), node_uuid, metadata, full_text)?;
        script_document_command::apply_generated_script_block(
            &mut conn,
            &CommandEnvelope {
                id: command.id,
                payload: GenerateScriptBlockCommand {
                    target_binding: inputs.target_binding,
                    block: command.payload,
                    script_inputs: inputs.script_inputs,
                    bible_relationship_inputs: inputs.bible_relationship_inputs,
                    bible_inputs: inputs.bible_inputs,
                    bible_context_scope: inputs.bible_context_scope,
                    script_context_scope: inputs.script_context_scope,
                },
            },
            0,
        )
        .map_err(|error| error.to_string())?;
        Ok(())
    })
    .await
    .map_err(|error| format!("script persistence task failed: {error}"))?
}

async fn persist_node_content_status(
    project_path: PathBuf,
    node_id: NodeId,
    status: ContentStatus,
) -> Result<(), String> {
    tokio::task::spawn_blocking(move || {
        let conn = crate::sqlite::open_write_connection(&project_path)
            .map_err(|error| error.to_string())?;
        timeline_node_store::update_node_content_status(&conn, node_id, status)
            .map_err(|error| error.to_string())
    })
    .await
    .map_err(|error| format!("node status persistence task failed: {error}"))?
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

async fn generate_scene_recap(state: &AppState, node_uuid: Uuid, script: &str) {
    use crate::prompt_format::build_recap_prompt;

    let node_id = NodeId(node_uuid);
    let (project_path, preceding_recap) = {
        let (project, project_path) = match active_sqlite_project(state).await {
            Ok(project) => project,
            Err(_) => {
                return;
            }
        };

        let siblings = project.timeline.siblings_of(node_id);
        let node = match project.timeline.node(node_id) {
            Ok(n) => n,
            Err(_) => return,
        };
        let preceding_recap = siblings
            .iter()
            .rfind(|s| s.time_range.end_ms <= node.time_range.start_ms)
            .and_then(|s| s.content.scene_recap.clone());
        (project_path, preceding_recap)
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

    if let Err(error) = persist_node_scene_recap(project_path, node_id, recap_text.clone()).await {
        tracing::warn!("Failed to persist scene recap for node {node_uuid}: {error}");
    }
    {
        let mut project_guard = state.project.lock();
        if let Some(project) = project_guard.as_mut()
            && let Ok(node) = project.timeline.node_mut(node_id)
        {
            node.content.scene_recap = Some(recap_text);
        }
    }

    let _ = state
        .events_tx
        .send(ServerEvent::NodeUpdated { node_id: node_uuid });
    state.trigger_save();

    tracing::info!("Scene recap generated for node {node_uuid}");
}

async fn persist_node_scene_recap(
    project_path: PathBuf,
    node_id: NodeId,
    scene_recap: String,
) -> Result<(), String> {
    tokio::task::spawn_blocking(move || {
        let conn = crate::sqlite::open_write_connection(&project_path)
            .map_err(|error| error.to_string())?;
        timeline_node_store::update_node_scene_recap(&conn, node_id, scene_recap)
            .map_err(|error| error.to_string())
    })
    .await
    .map_err(|error| format!("scene recap persistence task failed: {error}"))?
}

#[cfg(test)]
mod tests {
    use super::*;
    use ScriptSpanProvenance::AiGenerated;

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
        assert_eq!(command.payload.span_provenance, AiGenerated);
    }
}

#[cfg(test)]
#[path = "ai_generation_stream_tests.rs"]
mod stream_tests;

#[cfg(test)]
#[path = "ai_generation_runtime_tests.rs"]
mod runtime_tests;

#[cfg(test)]
#[path = "reference_retrieval_tests.rs"]
mod reference_retrieval_tests;
