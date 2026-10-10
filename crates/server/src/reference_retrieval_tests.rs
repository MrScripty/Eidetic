use super::{attach_rag_context, attach_rag_embedding};
use crate::embeddings::{Embedding, EmbeddingClient};
use crate::project_service::replace_active_project;
use crate::state::AppState;
use eidetic_core::ai::backend::{GenerateRequest, RagChunk};
use eidetic_core::ai::prompt::build_generate_request;
use eidetic_core::reference::{ReferenceDocument, ReferenceType, chunk_document};
use eidetic_core::{Project, Template};
use std::path::PathBuf;
use std::sync::Arc;

fn query(state: &AppState) -> Embedding {
    Embedding::new(
        EmbeddingClient::new(&state.ai_config.lock().embedding.base_url, "model").identity(),
        vec![1.0],
    )
    .unwrap()
}

fn install_reference(state: &AppState, source: &ReferenceDocument) {
    let chunk = chunk_document(source, 500, 50).remove(0);
    let embedding = query(state);
    let project = state.project.lock();
    let mut store = state.vector_store.lock();
    let ticket = store.begin_document(Arc::new(source.clone()));
    assert!(store.insert(
        &ticket,
        &project.as_ref().unwrap().references,
        chunk,
        embedding
    ));
}

fn request_with_old_context(project: &Project) -> GenerateRequest {
    let mut request = build_generate_request(project, project.timeline.nodes[0].id).unwrap();
    request.rag_context.push(RagChunk {
        source: "Must be cleared".into(),
        content: "Old context".into(),
        relevance_score: 1.0,
    });
    request
}

async fn fixture() -> (AppState, Project, PathBuf) {
    let state = AppState::new().await;
    state.ai_config.lock().embedding = crate::embeddings::EmbeddingConfig {
        provider: crate::embeddings::EmbeddingProvider::OpenAiCompatible,
        base_url: "http://localhost:18080/v1".into(),
        model: "model".into(),
        revision: "fixture".into(),
        ..Default::default()
    };
    let source = ReferenceDocument::new("Notes", "Original", ReferenceType::StyleGuide);
    let mut project = Template::MultiCam.build_project("Retrieval");
    project.references.push(source.clone());
    let path = std::env::temp_dir().join(format!("eidetic-retrieval-{}.db", uuid::Uuid::new_v4()));
    replace_active_project(&state, project.clone(), path.clone());
    install_reference(&state, &source);
    (state, project, path)
}

#[tokio::test]
async fn queued_generation_cannot_rebind_to_same_path_reopened_project() {
    let (state, mut reopened, path) = fixture().await;
    // Admission captures this epoch before the request's project snapshot awaits.
    let origin = state.vector_store.lock().scope();
    let mut request = request_with_old_context(&reopened);
    reopened.references[0].content = "New session source".into();
    replace_active_project(&state, reopened.clone(), path.clone());
    install_reference(&state, &reopened.references[0]);
    let config = state.ai_config.lock().clone();
    // This is the real asynchronous attach boundary; stale scope exits before I/O.
    attach_rag_context(&state, &config, &path, origin, &mut request).await;
    assert!(request.rag_context.is_empty());
    // Prove the replacement index is populated and usable by its own request.
    let current = state.vector_store.lock().scope();
    attach_rag_embedding(&state, &path, current, &query(&state), &mut request);
    assert_eq!(request.rag_context[0].content, "New session source");
    state.shutdown_tasks();
}

#[tokio::test]
async fn query_completed_after_reopen_or_delete_cannot_publish_old_context() {
    let (state, project, path) = fixture().await;
    let origin = state.vector_store.lock().scope();
    let completed_query = query(&state);
    let mut request = request_with_old_context(&project);
    replace_active_project(&state, project.clone(), path.clone());
    install_reference(&state, &project.references[0]);
    // Deterministically deliver the external result after replacement.
    attach_rag_embedding(&state, &path, origin, &completed_query, &mut request);
    assert!(request.rag_context.is_empty());
    let current = state.vector_store.lock().scope();
    let mut request = request_with_old_context(&project);
    crate::reference_service::delete_reference(&state, project.references[0].id.0).unwrap();
    attach_rag_embedding(&state, &path, current, &completed_query, &mut request);
    assert!(request.rag_context.is_empty());
    state.shutdown_tasks();
}

#[tokio::test]
async fn changed_config_path_or_missing_project_clears_preexisting_context() {
    let (state, project, path) = fixture().await;
    let scope = state.vector_store.lock().scope();
    let completed_query = query(&state);
    let config = state.ai_config.lock().clone();
    state.ai_config.lock().embedding.base_url = "http://localhost:18081/v1".into();
    let mut request = request_with_old_context(&project);
    attach_rag_embedding(&state, &path, scope, &completed_query, &mut request);
    assert!(request.rag_context.is_empty());
    *state.ai_config.lock() = config;
    let mut request = request_with_old_context(&project);
    attach_rag_embedding(
        &state,
        &path.with_extension("other"),
        scope,
        &completed_query,
        &mut request,
    );
    assert!(request.rag_context.is_empty());
    *state.project.lock() = None;
    let mut request = request_with_old_context(&project);
    attach_rag_embedding(&state, &path, scope, &completed_query, &mut request);
    assert!(request.rag_context.is_empty());
    state.shutdown_tasks();
}
