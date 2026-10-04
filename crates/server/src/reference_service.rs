use std::sync::Arc;

use eidetic_core::reference::{ReferenceDocument, ReferenceId, ReferenceType, chunk_document};
use serde::{Deserialize, Serialize};
use uuid::Uuid;

use crate::backend_error::BackendError;
use crate::embeddings::{Embedding, EmbeddingClient};
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
    let chunks = chunk_document(
        &doc,
        crate::state::constants::REFERENCE_CHUNK_SIZE,
        crate::state::constants::REFERENCE_CHUNK_OVERLAP,
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

    let state_clone = state.clone();
    state
        .task_supervisor
        .spawn("reference-embedding", async move {
            let config = state_clone.ai_config.lock().clone();
            let client =
                EmbeddingClient::new(&config.base_url, crate::state::constants::EMBEDDING_MODEL);

            let (mut indexed, mut failed, mut discarded) = (0, 0, 0);
            for chunk in chunks {
                match client.embed(&chunk.content).await {
                    Ok(embedding) => {
                        let inserted = publish_embedding(&state_clone, &ticket, chunk, embedding);
                        if inserted {
                            indexed += 1;
                        } else {
                            discarded += 1;
                        }
                    }
                    Err(error) => {
                        failed += 1;
                        tracing::warn!("Failed to embed chunk: {error}");
                    }
                }
            }
            tracing::info!(
                indexed,
                failed,
                discarded,
                "Reference embedding task finished"
            );
        });

    Ok(response)
}

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
        use crate::embeddings::{Embedding, EmbeddingClient};
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
