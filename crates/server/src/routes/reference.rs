use axum::extract::{Path, State};
use axum::routing::{delete, get, post};
use axum::{Json, Router};
use serde::Deserialize;
use uuid::Uuid;

use eidetic_core::reference::{chunk_document, ReferenceDocument, ReferenceId, ReferenceType};

use crate::error::{json_value, ApiError, ApiJson};
use crate::embeddings::EmbeddingClient;
use crate::state::AppState;
use crate::validation;

pub fn router() -> Router<AppState> {
    Router::new()
        .route("/references", get(list_references))
        .route("/references", post(upload_reference))
        .route("/references/{id}", delete(delete_reference))
}

async fn list_references(State(state): State<AppState>) -> ApiJson {
    let guard = state.project.lock();
    match guard.as_ref() {
        Some(p) => json_value(&p.references),
        None => Err(ApiError::no_project()),
    }
}

#[derive(Deserialize)]
struct UploadReferenceRequest {
    name: String,
    content: String,
    doc_type: String,
}

async fn upload_reference(
    State(state): State<AppState>,
    Json(body): Json<UploadReferenceRequest>,
) -> ApiJson {
    validation::validate_name(&body.name, "reference name")?;
    if body.content.trim().is_empty() {
        return Err(ApiError::bad_request("reference content is required"));
    }

    let doc_type = match body.doc_type.as_str() {
        "character_bible" => ReferenceType::CharacterBible,
        "style_guide" => ReferenceType::StyleGuide,
        "world_building" => ReferenceType::WorldBuilding,
        "previous_episode" => ReferenceType::PreviousEpisode,
        other => ReferenceType::Custom(other.to_string()),
    };

    let doc = ReferenceDocument::new(body.name, body.content, doc_type);
    let json = serde_json::to_value(&doc).map_err(|e| ApiError::internal(e.to_string()))?;

    // Chunk the document for embedding.
    let chunks = chunk_document(
        &doc,
        crate::state::constants::REFERENCE_CHUNK_SIZE,
        crate::state::constants::REFERENCE_CHUNK_OVERLAP,
    );

    // Store the document in the project.
    {
        let mut guard = state.project.lock();
        let Some(project) = guard.as_mut() else {
            return Err(ApiError::no_project());
        };
        project.references.push(doc);
    }
    state.trigger_save();

    // Spawn async embedding task — doesn't block the response.
    let state_clone = state.clone();
    tokio::spawn(async move {
        let config = state_clone.ai_config.lock().clone();
        let client = EmbeddingClient::new(&config.base_url, crate::state::constants::EMBEDDING_MODEL);

        for chunk in chunks {
            match client.embed(&chunk.content).await {
                Ok(embedding) => {
                    state_clone.vector_store.lock().insert(chunk, embedding);
                }
                Err(e) => {
                    tracing::warn!("Failed to embed chunk: {e}");
                }
            }
        }
        tracing::info!("Reference material embedding complete");
    });

    Ok(Json(json))
}

async fn delete_reference(
    State(state): State<AppState>,
    Path(id): Path<Uuid>,
) -> ApiJson {
    let ref_id = ReferenceId(id);

    {
        let mut guard = state.project.lock();
        let Some(project) = guard.as_mut() else {
            return Err(ApiError::no_project());
        };
        project.references.retain(|r| r.id != ref_id);
    }

    // Remove from vector store.
    state.vector_store.lock().remove_document(ref_id);
    state.trigger_save();

    Ok(Json(serde_json::json!({ "deleted": true })))
}
