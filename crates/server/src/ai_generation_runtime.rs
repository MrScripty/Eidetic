use std::path::PathBuf;

use eidetic_core::ai::backend::{GenerateRequest, RagChunk};
use eidetic_core::contracts::{
    CommandEnvelope, CommandId, ScriptBlockId, ScriptBlockKind, ScriptDocumentId, ScriptSegmentId,
    ScriptSegmentStatus, ScriptSpanProvenance, SetScriptBlockCommand,
};
use eidetic_core::timeline::node::{ContentStatus, NodeId};
use futures::StreamExt;
use uuid::Uuid;

use crate::ai_backends::Backend;
use crate::embeddings::EmbeddingClient;
use crate::prompt_format::build_chat_prompt;
use crate::script_document_command;
use crate::state::{AppState, ServerEvent};
use crate::timeline_node_store;

use crate::ai_service::active_sqlite_project;

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
) {
    let node_id = NodeId(node_uuid);
    let config = state.ai_config.lock().clone();
    let backend = Backend::from_config(&config);

    attach_rag_context(&state, &config, &mut request).await;
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

    let full_text = stream_generated_text(&state, node_uuid, stream).await;
    if full_text.is_empty() {
        handle_empty_generation(&state, project_path, node_id, node_uuid).await;
        return;
    }

    persist_successful_generation(state, project_path, node_id, node_uuid, full_text).await;
}

async fn attach_rag_context(
    state: &AppState,
    config: &crate::state::AiConfig,
    request: &mut GenerateRequest,
) {
    if state.vector_store.lock().is_empty() {
        return;
    }
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

async fn stream_generated_text(
    state: &AppState,
    node_uuid: Uuid,
    mut stream: eidetic_core::ai::backend::GenerateStream,
) -> String {
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
    full_text
}

async fn handle_generation_failure(
    state: &AppState,
    project_path: PathBuf,
    node_id: NodeId,
    node_uuid: Uuid,
    error: String,
) {
    tracing::error!("AI generation failed for node {node_uuid}: {error}");
    if let Err(status_error) =
        persist_node_content_status(project_path, node_id, ContentStatus::NotesOnly).await
    {
        tracing::warn!(
            "Failed to persist generation error status for node {node_uuid}: {status_error}"
        );
    }
    set_project_node_status(state, node_id, ContentStatus::NotesOnly);
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
    if let Err(error) =
        persist_node_content_status(project_path, node_id, ContentStatus::NotesOnly).await
    {
        tracing::warn!("Failed to persist empty-generation status for node {node_uuid}: {error}");
    }
    set_project_node_status(state, node_id, ContentStatus::NotesOnly);
    let _ = state.events_tx.send(ServerEvent::GenerationError {
        node_id: node_uuid,
        error: "AI produced no output".into(),
    });
    state.generating.lock().remove(&node_uuid);
}

async fn persist_successful_generation(
    state: AppState,
    project_path: PathBuf,
    node_id: NodeId,
    node_uuid: Uuid,
    full_text: String,
) {
    if let Err(error) =
        persist_node_content_status(project_path.clone(), node_id, ContentStatus::HasContent).await
    {
        tracing::warn!("Failed to persist generated-content status for node {node_uuid}: {error}");
    }
    let metadata = match successful_generation_metadata(&state, node_id, node_uuid) {
        Some(metadata) => metadata,
        None => return,
    };
    if let Err(error) =
        persist_generated_script_block(project_path, node_uuid, metadata, full_text.clone()).await
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
    generate_scene_recap(&state, node_uuid, &full_text).await;
    state.generating.lock().remove(&node_uuid);
}

fn successful_generation_metadata(
    state: &AppState,
    node_id: NodeId,
    node_uuid: Uuid,
) -> Option<GeneratedScriptMetadata> {
    let mut project_guard = state.project.lock();
    let Some(project) = project_guard.as_mut() else {
        let _ = state.events_tx.send(ServerEvent::GenerationError {
            node_id: node_uuid,
            error: "no project loaded".into(),
        });
        state.generating.lock().remove(&node_uuid);
        return None;
    };
    let Ok(node) = project.timeline.node_mut(node_id) else {
        let _ = state.events_tx.send(ServerEvent::GenerationError {
            node_id: node_uuid,
            error: "node not found".into(),
        });
        state.generating.lock().remove(&node_uuid);
        return None;
    };
    node.content.status = ContentStatus::HasContent;
    Some(GeneratedScriptMetadata {
        project_name: project.name.clone(),
        start_ms: node.time_range.start_ms,
        end_ms: node.time_range.end_ms,
    })
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
