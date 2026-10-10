use std::collections::HashMap;
use std::sync::Arc;

use eidetic_core::reference::{ReferenceChunk, ReferenceDocument, ReferenceId};
use uuid::Uuid;

use crate::embeddings::Embedding;

/// A capability to publish derived chunks for one exact source and index lifetime.
/// Removing/replacing the source or resetting the project invalidates it.
#[derive(Debug, Clone)]
pub(crate) struct IndexTicket {
    scope: Uuid,
    revision: Uuid,
    document_id: ReferenceId,
}

struct IndexedDocument {
    revision: Uuid,
    source: Arc<ReferenceDocument>,
    error: Option<String>,
    complete: bool,
    stale: bool,
}

/// Disposable retrieval index. Canonical references remain in the project.
pub struct VectorStore {
    scope: Uuid,
    documents: HashMap<ReferenceId, IndexedDocument>,
    entries: HashMap<Uuid, (ReferenceChunk, Embedding)>,
}

impl VectorStore {
    pub fn new() -> Self {
        Self {
            scope: Uuid::new_v4(),
            documents: HashMap::new(),
            entries: HashMap::new(),
        }
    }

    pub(crate) fn scope(&self) -> Uuid {
        self.scope
    }

    /// Discard only derived state when a project is opened or replaced.
    pub(crate) fn reset(&mut self) {
        *self = Self::new();
    }

    pub(crate) fn begin_document(&mut self, source: Arc<ReferenceDocument>) -> IndexTicket {
        self.remove_document(source.id);
        let ticket = IndexTicket {
            scope: self.scope,
            revision: Uuid::new_v4(),
            document_id: source.id,
        };
        self.documents.insert(
            source.id,
            IndexedDocument {
                revision: ticket.revision,
                source,
                error: None,
                complete: false,
                stale: false,
            },
        );
        ticket
    }

    pub(crate) fn ticket_current(&self, ticket: &IndexTicket) -> bool {
        ticket.scope == self.scope
            && self
                .documents
                .get(&ticket.document_id)
                .is_some_and(|doc| doc.revision == ticket.revision)
    }

    pub(crate) fn finish_document(
        &mut self,
        ticket: &IndexTicket,
        sources: &[ReferenceDocument],
        error: Option<String>,
    ) {
        if !self.ticket_current(ticket) {
            return;
        }
        let Some(doc) = self.documents.get_mut(&ticket.document_id) else {
            return;
        };
        if !source_is_current(&doc.source, sources) {
            return;
        }
        doc.complete = true;
        doc.error = error;
        if doc.error.is_some() {
            self.entries
                .retain(|_, (chunk, _)| chunk.document_id != ticket.document_id);
        }
    }

    pub(crate) fn mark_incompatible(&mut self, query: &Embedding) {
        for (chunk, embedding) in self.entries.values() {
            if (embedding.identity != query.identity
                || embedding.values.len() != query.values.len())
                && let Some(doc) = self.documents.get_mut(&chunk.document_id)
            {
                doc.stale = true;
                doc.error = Some(
                    "Embedding model revision or dimensions changed; Reindex references".into(),
                );
            }
        }
    }

    pub(crate) fn document_status(
        &self,
        source: &ReferenceDocument,
    ) -> crate::reference_service::ReferenceDocumentIndexStatus {
        let document = self
            .documents
            .get(&source.id)
            .filter(|doc| source_is_current(&doc.source, std::slice::from_ref(source)));
        let entries: Vec<_> = self
            .entries
            .values()
            .filter(|(chunk, _)| chunk.document_id == source.id)
            .collect();
        crate::reference_service::ReferenceDocumentIndexStatus {
            document_id: source.id.0,
            name: source.name.clone(),
            state: match document {
                None => "needs_reindex",
                Some(doc) if doc.stale => "stale",
                Some(doc) if doc.error.is_some() => "failed",
                Some(doc) if doc.complete => "ready",
                _ => "indexing",
            }
            .into(),
            indexed_chunks: entries.len(),
            dimensions: entries.first().map(|(_, embedding)| embedding.values.len()),
            model: entries
                .first()
                .map(|(_, embedding)| embedding.model().to_owned()),
            revision: entries
                .first()
                .map(|(_, embedding)| embedding.revision_label()),
            error: document.and_then(|doc| doc.error.clone()),
        }
    }

    /// Caller holds the project guard through publication, closing the deletion race.
    pub(crate) fn insert(
        &mut self,
        ticket: &IndexTicket,
        current_sources: &[ReferenceDocument],
        chunk: ReferenceChunk,
        embedding: Embedding,
    ) -> bool {
        let Some(document) = self.documents.get(&ticket.document_id) else {
            return false;
        };
        if ticket.scope != self.scope
            || ticket.revision != document.revision
            || chunk.document_id != ticket.document_id
            || !source_is_current(&document.source, current_sources)
        {
            return false;
        }
        self.entries.insert(chunk.id, (chunk, embedding));
        true
    }

    pub fn remove_document(&mut self, doc_id: ReferenceId) {
        self.documents.remove(&doc_id);
        self.entries
            .retain(|_, (chunk, _)| chunk.document_id != doc_id);
    }

    /// Exclude stale sources and incompatible spaces before ranking, never score
    /// them as zero. Equal scores use stable chunk IDs rather than HashMap order.
    pub(crate) fn search(
        &self,
        scope: Uuid,
        current_sources: &[ReferenceDocument],
        query: &Embedding,
        top_k: usize,
    ) -> Vec<(&ReferenceChunk, f32)> {
        if scope != self.scope {
            return Vec::new();
        }
        let mut scored: Vec<_> = self
            .entries
            .values()
            .filter(|(chunk, embedding)| {
                embedding.identity == query.identity
                    && embedding.values.len() == query.values.len()
                    && self
                        .documents
                        .get(&chunk.document_id)
                        .is_some_and(|document| {
                            source_is_current(&document.source, current_sources)
                        })
            })
            .map(|(chunk, embedding)| (chunk, cosine_similarity(&query.values, &embedding.values)))
            .collect();
        scored.sort_by(|a, b| b.1.total_cmp(&a.1).then_with(|| a.0.id.cmp(&b.0.id)));
        scored.truncate(top_k);
        scored
    }

    pub fn is_empty(&self) -> bool {
        self.entries.is_empty()
    }
}

fn source_is_current(source: &ReferenceDocument, current_sources: &[ReferenceDocument]) -> bool {
    current_sources.iter().any(|current| {
        current.id == source.id
            && current.name == source.name
            && current.content == source.content
            && current.doc_type == source.doc_type
    })
}

fn cosine_similarity(a: &[f32], b: &[f32]) -> f32 {
    // Validated finite f32 inputs accumulated in f64 avoid overflow and underflow
    // at f32 extremes. Callers have already checked nonzero norms and dimensions.
    let (mut dot, mut mag_a, mut mag_b) = (0.0_f64, 0.0_f64, 0.0_f64);
    for (&x, &y) in a.iter().zip(b) {
        let (x, y) = (f64::from(x), f64::from(y));
        dot += x * y;
        mag_a += x * x;
        mag_b += y * y;
    }
    (dot / (mag_a.sqrt() * mag_b.sqrt())).clamp(-1.0, 1.0) as f32
}

#[cfg(test)]
#[path = "vector_store_tests.rs"]
mod tests;
