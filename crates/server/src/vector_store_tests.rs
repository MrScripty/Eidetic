use super::*;
use crate::embeddings::EmbeddingClient;
use eidetic_core::reference::{ReferenceType, chunk_document};

fn vector(values: Vec<f32>) -> Embedding {
    Embedding::new(
        EmbeddingClient::new("http://localhost:18080/v1", "model").identity(),
        values,
    )
    .unwrap()
}

fn fixture() -> (
    VectorStore,
    IndexTicket,
    Vec<ReferenceDocument>,
    ReferenceChunk,
) {
    let doc = ReferenceDocument::new("Notes", "Original text", ReferenceType::StyleGuide);
    let chunk = chunk_document(&doc, 500, 50).remove(0);
    let mut store = VectorStore::new();
    let ticket = store.begin_document(Arc::new(doc.clone()));
    (store, ticket, vec![doc], chunk)
}

#[test]
fn deletion_invalidates_pending_publication_and_existing_results() {
    let (mut store, ticket, sources, chunk) = fixture();
    assert!(store.insert(&ticket, &sources, chunk.clone(), vector(vec![1.0, 0.0])));
    store.remove_document(chunk.document_id);
    assert!(!store.insert(&ticket, &sources, chunk, vector(vec![1.0, 0.0])));
    assert!(
        store
            .search(store.scope(), &sources, &vector(vec![1.0, 0.0]), 3)
            .is_empty()
    );
}

#[test]
fn changed_and_removed_sources_are_excluded_even_without_explicit_invalidation() {
    let (mut store, ticket, mut sources, chunk) = fixture();
    assert!(store.insert(&ticket, &sources, chunk.clone(), vector(vec![1.0])));
    sources[0].content = "Changed text".into();
    assert!(!store.insert(&ticket, &sources, chunk, vector(vec![1.0])));
    assert!(
        store
            .search(store.scope(), &sources, &vector(vec![1.0]), 3)
            .is_empty()
    );
    assert!(
        store
            .search(store.scope(), &[], &vector(vec![1.0]), 3)
            .is_empty()
    );
}

#[test]
fn replacement_and_project_reset_reject_old_tickets_and_queries() {
    let (mut store, ticket, sources, chunk) = fixture();
    let replacement = store.begin_document(Arc::new(sources[0].clone()));
    assert!(!store.insert(&ticket, &sources, chunk.clone(), vector(vec![1.0])));
    assert!(store.insert(&replacement, &sources, chunk.clone(), vector(vec![1.0])));
    let old_scope = store.scope();
    store.reset();
    let current = store.begin_document(Arc::new(sources[0].clone()));
    assert!(!store.insert(&replacement, &sources, chunk.clone(), vector(vec![1.0])));
    assert!(store.insert(&current, &sources, chunk, vector(vec![1.0])));
    assert!(
        store
            .search(old_scope, &sources, &vector(vec![1.0]), 3)
            .is_empty()
    );
    assert_eq!(
        store
            .search(store.scope(), &sources, &vector(vec![1.0]), 3)
            .len(),
        1
    );
}

#[test]
fn incompatible_model_endpoint_and_dimension_never_participate_in_ranking() {
    let (mut store, ticket, sources, chunk) = fixture();
    assert!(store.insert(&ticket, &sources, chunk, vector(vec![-1.0, 0.0])));
    // A valid negative score must not lose to an incompatible fake zero score.
    assert_eq!(
        store.search(store.scope(), &sources, &vector(vec![1.0, 0.0]), 3)[0].1,
        -1.0
    );
    for query in [
        vector(vec![1.0]),
        Embedding::new(
            EmbeddingClient::new("http://localhost:18081/v1", "model").identity(),
            vec![1.0, 0.0],
        )
        .unwrap(),
        Embedding::new(
            EmbeddingClient::new("http://localhost:18080/v1", "other").identity(),
            vec![1.0, 0.0],
        )
        .unwrap(),
    ] {
        assert!(store.search(store.scope(), &sources, &query, 3).is_empty());
    }
}

#[test]
fn ranking_is_stable_bounded_and_numerically_finite() {
    let (mut store, ticket, sources, mut chunk) = fixture();
    for id in [3, 1, 2] {
        chunk.id = Uuid::from_u128(id);
        assert!(store.insert(
            &ticket,
            &sources,
            chunk.clone(),
            vector(vec![f32::MAX, f32::MAX])
        ));
    }
    let results = store.search(
        store.scope(),
        &sources,
        &vector(vec![f32::MAX, f32::MAX]),
        2,
    );
    assert_eq!(
        results
            .iter()
            .map(|(chunk, _)| chunk.id)
            .collect::<Vec<_>>(),
        vec![Uuid::from_u128(1), Uuid::from_u128(2)]
    );
    assert!(results.iter().all(|(_, score)| (*score - 1.0).abs() < 1e-6));
    assert_eq!(
        cosine_similarity(&[f32::MIN_POSITIVE], &[f32::MIN_POSITIVE]),
        1.0
    );
    assert_eq!(cosine_similarity(&[1.0, 0.0], &[0.0, 1.0]), 0.0);
    assert!(
        store
            .search(store.scope(), &sources, &vector(vec![1.0, 1.0]), 0)
            .is_empty()
    );
}

#[test]
fn source_name_type_and_document_identity_are_bound() {
    let (mut store, ticket, sources, chunk) = fixture();
    assert!(store.insert(&ticket, &sources, chunk.clone(), vector(vec![1.0])));
    let mut wrong_chunk = chunk;
    wrong_chunk.document_id = ReferenceId::new();
    assert!(!store.insert(&ticket, &sources, wrong_chunk, vector(vec![1.0])));
    let mut renamed = sources.clone();
    renamed[0].name = "Renamed".into();
    let mut retyped = sources.clone();
    retyped[0].doc_type = ReferenceType::WorldBuilding;
    for changed in [renamed, retyped] {
        assert!(
            store
                .search(store.scope(), &changed, &vector(vec![1.0]), 3)
                .is_empty()
        );
    }
}
