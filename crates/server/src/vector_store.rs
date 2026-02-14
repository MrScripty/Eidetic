use std::collections::HashMap;
use uuid::Uuid;

use eidetic_core::reference::{ReferenceChunk, ReferenceId};

/// In-memory vector store for reference material chunks.
pub struct VectorStore {
    entries: HashMap<Uuid, (ReferenceChunk, Vec<f32>)>,
}

impl VectorStore {
    pub fn new() -> Self {
        Self {
            entries: HashMap::new(),
        }
    }

    /// Insert a chunk with its embedding vector.
    pub fn insert(&mut self, chunk: ReferenceChunk, embedding: Vec<f32>) {
        self.entries.insert(chunk.id, (chunk, embedding));
    }

    /// Remove all chunks belonging to a document.
    pub fn remove_document(&mut self, doc_id: ReferenceId) {
        self.entries.retain(|_, (chunk, _)| chunk.document_id != doc_id);
    }

    /// Search for the top-k most similar chunks to a query embedding.
    pub fn search(&self, query: &[f32], top_k: usize) -> Vec<(&ReferenceChunk, f32)> {
        let mut scored: Vec<(&ReferenceChunk, f32)> = self
            .entries
            .values()
            .map(|(chunk, emb)| (chunk, cosine_similarity(query, emb)))
            .collect();

        scored.sort_by(|a, b| b.1.partial_cmp(&a.1).unwrap_or(std::cmp::Ordering::Equal));
        scored.truncate(top_k);
        scored
    }

    /// Check if the store has any entries.
    pub fn is_empty(&self) -> bool {
        self.entries.is_empty()
    }
}

fn cosine_similarity(a: &[f32], b: &[f32]) -> f32 {
    if a.len() != b.len() || a.is_empty() {
        return 0.0;
    }

    let mut dot = 0.0f32;
    let mut mag_a = 0.0f32;
    let mut mag_b = 0.0f32;

    for (x, y) in a.iter().zip(b.iter()) {
        dot += x * y;
        mag_a += x * x;
        mag_b += y * y;
    }

    let denom = mag_a.sqrt() * mag_b.sqrt();
    if denom == 0.0 {
        0.0
    } else {
        dot / denom
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn cosine_similarity_identical_vectors() {
        let v = vec![1.0, 2.0, 3.0];
        let score = cosine_similarity(&v, &v);
        assert!((score - 1.0).abs() < 1e-6);
    }

    #[test]
    fn cosine_similarity_orthogonal_vectors() {
        let a = vec![1.0, 0.0];
        let b = vec![0.0, 1.0];
        let score = cosine_similarity(&a, &b);
        assert!(score.abs() < 1e-6);
    }
}
