use std::sync::Arc;

use eidetic_core::reference::{ReferenceDocument, ReferenceId, ReferenceType, chunk_document};
use serde::{Deserialize, Serialize};
use uuid::Uuid;

use crate::backend_error::BackendError;
#[cfg(test)]
use crate::embeddings::Embedding;
use crate::embeddings::EmbeddingClient;
use crate::state::AppState;
use crate::validation;
use crate::vector_store::IndexTicket;

#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct UploadReferenceRequest {
    pub name: String,
    pub content: String,
    pub doc_type: String,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
pub struct DeleteReferenceResponse {
    pub deleted: bool,
}

pub fn list_references(state: &AppState) -> Result<Vec<ReferenceDocument>, BackendError> {
    let guard = state.project.lock();
    let Some(project) = guard.as_ref() else {
        return Err(BackendError::no_project());
    };

    Ok(project.references.clone())
}

pub fn upload_reference(
    state: &AppState,
    request: UploadReferenceRequest,
) -> Result<ReferenceDocument, BackendError> {
    validation::validate_name(&request.name, "reference name")?;
    if request.content.trim().is_empty() {
        return Err(BackendError::bad_request("reference content is required"));
    }

    let doc = ReferenceDocument::new(
        request.name,
        request.content,
        parse_reference_type(&request.doc_type),
    );
    let response = doc.clone();

    let ticket = {
        let mut guard = state.project.lock();
        let Some(project) = guard.as_mut() else {
            return Err(BackendError::no_project());
        };
        let ticket = state
            .vector_store
            .lock()
            .begin_document(Arc::new(doc.clone()));
        project.references.push(doc);
        ticket
    };
    state.trigger_save();

    schedule_document(state, response.clone(), ticket);

    Ok(response)
}

/// Derived index state is session-bound and never persisted as canonical content.
#[derive(Debug, Clone, Serialize, Default)]
pub struct ReferenceIndexStatus {
    pub documents: Vec<ReferenceDocumentIndexStatus>,
}
#[derive(Debug, Clone, Serialize)]
pub struct ReferenceDocumentIndexStatus {
    pub document_id: Uuid,
    pub name: String,
    pub state: String,
    pub indexed_chunks: usize,
    pub dimensions: Option<usize>,
    pub model: Option<String>,
    pub revision: Option<String>,
    pub error: Option<String>,
}

pub fn index_status(state: &AppState) -> Result<ReferenceIndexStatus, BackendError> {
    let project = state.project.lock();
    let project = project.as_ref().ok_or_else(BackendError::no_project)?;
    let store = state.vector_store.lock();
    Ok(ReferenceIndexStatus {
        documents: project
            .references
            .iter()
            .map(|doc| store.document_status(doc))
            .collect(),
    })
}

pub fn schedule_reindex(state: &AppState) {
    rebuild_references(state, true);
}

pub(crate) fn schedule_reopened_index(state: &AppState) {
    rebuild_references(state, false);
}

fn rebuild_references(state: &AppState, reset: bool) {
    let documents = {
        let project = state.project.lock();
        let Some(project) = project.as_ref() else {
            return;
        };
        let mut store = state.vector_store.lock();
        if reset {
            store.reset();
        }
        project
            .references
            .iter()
            .map(|doc| (doc.clone(), store.begin_document(Arc::new(doc.clone()))))
            .collect::<Vec<_>>()
    };
    for (doc, ticket) in documents {
        schedule_document(state, doc, ticket);
    }
}

fn schedule_document(state: &AppState, doc: ReferenceDocument, ticket: IndexTicket) {
    let state = state.clone();
    let config = state.ai_config.lock().embedding.clone();
    state
        .clone()
        .task_supervisor
        .spawn("reference-embedding", async move {
            let result = index_document(&state, &doc, &ticket, &config).await;
            if let Err(error) = result {
                tracing::warn!(document=%doc.name, %error, "Reference indexing failed");
                let project = state.project.lock();
                if let Some(project) = project.as_ref() {
                    state.vector_store.lock().finish_document(
                        &ticket,
                        &project.references,
                        Some(error),
                    );
                }
            }
        });
}

async fn index_document(
    state: &AppState,
    doc: &ReferenceDocument,
    ticket: &IndexTicket,
    config: &crate::embeddings::EmbeddingConfig,
) -> Result<(), String> {
    let client = EmbeddingClient::from_config(config).await?;
    let chunks = chunk_document(
        doc,
        crate::state::constants::REFERENCE_CHUNK_SIZE,
        crate::state::constants::REFERENCE_CHUNK_OVERLAP,
    );
    let mut completed = Vec::new();
    let mut dimensions = None;
    for chunk in chunks {
        // Obsolete tasks stop before the next expensive model operation.
        if state.ai_config.lock().embedding != *config
            || !state.vector_store.lock().ticket_current(ticket)
        {
            return Ok(());
        }
        let embedding = client.embed(&chunk.content).await?;
        if dimensions.is_some_and(|value| value != embedding.values.len()) {
            return Err("Embedding dimensions changed during reference indexing".into());
        }
        dimensions = Some(embedding.values.len());
        completed.push((chunk, embedding));
    }
    // Publish the entire document only after all real operations succeeded.
    let project = state.project.lock();
    let Some(project) = project.as_ref() else {
        return Ok(());
    };
    if state.ai_config.lock().embedding != *config {
        return Ok(());
    }
    let mut store = state.vector_store.lock();
    for (chunk, embedding) in completed {
        store.insert(ticket, &project.references, chunk, embedding);
    }
    store.finish_document(ticket, &project.references, None);
    Ok(())
}

#[cfg(test)]
fn publish_embedding(
    state: &AppState,
    ticket: &IndexTicket,
    chunk: eidetic_core::reference::ReferenceChunk,
    embedding: Embedding,
) -> bool {
    // Lock order is always project -> vector index. Deletion and project
    // replacement cannot interleave with publication. No guard crosses an await.
    let guard = state.project.lock();
    guard.as_ref().is_some_and(|project| {
        state
            .vector_store
            .lock()
            .insert(ticket, &project.references, chunk, embedding)
    })
}

pub fn delete_reference(
    state: &AppState,
    id: Uuid,
) -> Result<DeleteReferenceResponse, BackendError> {
    let ref_id = ReferenceId(id);

    let deleted = {
        let mut guard = state.project.lock();
        let Some(project) = guard.as_mut() else {
            return Err(BackendError::no_project());
        };
        let before_count = project.references.len();
        project
            .references
            .retain(|reference| reference.id != ref_id);
        state.vector_store.lock().remove_document(ref_id);
        project.references.len() != before_count
    };

    state.trigger_save();

    Ok(DeleteReferenceResponse { deleted })
}

fn parse_reference_type(value: &str) -> ReferenceType {
    match value {
        "CharacterBible" | "character_bible" => ReferenceType::CharacterBible,
        "StyleGuide" | "style_guide" => ReferenceType::StyleGuide,
        "WorldBuilding" | "world_building" => ReferenceType::WorldBuilding,
        "PreviousEpisode" | "previous_episode" => ReferenceType::PreviousEpisode,
        other => ReferenceType::Custom(other.to_string()),
    }
}

#[cfg(test)]
mod tests {
    use super::{UploadReferenceRequest, list_references, upload_reference};
    use crate::state::AppState;
    use eidetic_core::Template;
    use eidetic_core::reference::ReferenceType;

    #[tokio::test]
    async fn list_references_requires_loaded_project() {
        let state = AppState::new().await;

        let error = list_references(&state).expect_err("missing project");

        assert_eq!(error.message(), "no project loaded");
    }

    #[tokio::test]
    async fn upload_reference_accepts_frontend_variant_names() {
        let state = AppState::new().await;
        *state.project.lock() = Some(Template::MultiCam.build_project("Reference Test"));

        let reference = upload_reference(
            &state,
            UploadReferenceRequest {
                name: "Tone Guide".into(),
                content: "Keep scene turns precise.".into(),
                doc_type: "StyleGuide".into(),
            },
        )
        .expect("reference upload should succeed");

        assert_eq!(reference.doc_type, ReferenceType::StyleGuide);
        assert_eq!(list_references(&state).unwrap().len(), 1);

        state.shutdown_tasks();
    }
    #[tokio::test]
    async fn completed_embedding_cannot_resurrect_a_deleted_reference() {
        use super::{delete_reference, publish_embedding};
        #[cfg(test)]
        use crate::embeddings::Embedding;
        use crate::embeddings::EmbeddingClient;
        use eidetic_core::reference::{ReferenceDocument, chunk_document};
        use std::sync::Arc;

        let state = AppState::new().await;
        let doc = ReferenceDocument::new("Notes", "Source", ReferenceType::StyleGuide);
        let chunk = chunk_document(&doc, 500, 50).remove(0);
        let mut project = Template::MultiCam.build_project("Test");
        project.references.push(doc.clone());
        *state.project.lock() = Some(project);
        let ticket = state
            .vector_store
            .lock()
            .begin_document(Arc::new(doc.clone()));
        let completed = Embedding::new(
            EmbeddingClient::new("http://localhost:1", "model").identity(),
            vec![1.0],
        )
        .unwrap();
        assert!(delete_reference(&state, doc.id.0).unwrap().deleted);
        assert!(!publish_embedding(&state, &ticket, chunk, completed));
        assert!(state.vector_store.lock().is_empty());
        assert!(list_references(&state).unwrap().is_empty());
        state.shutdown_tasks();
    }
}
