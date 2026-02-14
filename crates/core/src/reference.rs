use serde::{Deserialize, Serialize};
use uuid::Uuid;

/// Unique identifier for a reference document.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub struct ReferenceId(pub Uuid);

impl ReferenceId {
    pub fn new() -> Self {
        Self(Uuid::new_v4())
    }
}

/// Type of reference material.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub enum ReferenceType {
    CharacterBible,
    StyleGuide,
    WorldBuilding,
    PreviousEpisode,
    Custom(String),
}

/// A reference document uploaded by the user.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ReferenceDocument {
    pub id: ReferenceId,
    pub name: String,
    pub content: String,
    pub doc_type: ReferenceType,
}

impl ReferenceDocument {
    pub fn new(name: impl Into<String>, content: impl Into<String>, doc_type: ReferenceType) -> Self {
        Self {
            id: ReferenceId::new(),
            name: name.into(),
            content: content.into(),
            doc_type,
        }
    }
}

/// A chunk of a reference document for embedding and retrieval.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ReferenceChunk {
    pub id: Uuid,
    pub document_id: ReferenceId,
    pub document_name: String,
    pub content: String,
    pub offset: usize,
}

/// Split a document into overlapping chunks at paragraph boundaries.
pub fn chunk_document(doc: &ReferenceDocument, max_chunk_chars: usize, overlap_chars: usize) -> Vec<ReferenceChunk> {
    let text = &doc.content;
    if text.is_empty() {
        return Vec::new();
    }

    let paragraphs: Vec<&str> = text.split("\n\n").collect();
    let mut chunks = Vec::new();
    let mut current = String::new();
    let mut offset: usize = 0;
    let mut chunk_start = 0;

    for para in &paragraphs {
        let para_trimmed = para.trim();
        if para_trimmed.is_empty() {
            offset += para.len() + 2; // account for \n\n separator
            continue;
        }

        if !current.is_empty() && current.len() + para_trimmed.len() + 2 > max_chunk_chars {
            // Emit current chunk.
            chunks.push(ReferenceChunk {
                id: Uuid::new_v4(),
                document_id: doc.id,
                document_name: doc.name.clone(),
                content: current.clone(),
                offset: chunk_start,
            });

            // Start new chunk with overlap from the end of the previous.
            let overlap_start = current.len().saturating_sub(overlap_chars);
            current = current[overlap_start..].to_string();
            chunk_start = offset.saturating_sub(current.len());
        }

        if !current.is_empty() {
            current.push_str("\n\n");
        }
        current.push_str(para_trimmed);
        if chunks.is_empty() && current.len() == para_trimmed.len() {
            chunk_start = offset;
        }
        offset += para.len() + 2;
    }

    // Emit final chunk.
    if !current.is_empty() {
        chunks.push(ReferenceChunk {
            id: Uuid::new_v4(),
            document_id: doc.id,
            document_name: doc.name.clone(),
            content: current,
            offset: chunk_start,
        });
    }

    chunks
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn chunk_short_document_single_chunk() {
        let doc = ReferenceDocument::new("test", "Short text.", ReferenceType::StyleGuide);
        let chunks = chunk_document(&doc, 500, 50);
        assert_eq!(chunks.len(), 1);
        assert_eq!(chunks[0].content, "Short text.");
    }

    #[test]
    fn chunk_long_document_multiple_chunks() {
        let para = "A".repeat(200);
        let content = format!("{}\n\n{}\n\n{}", para, para, para);
        let doc = ReferenceDocument::new("test", content, ReferenceType::WorldBuilding);
        let chunks = chunk_document(&doc, 300, 50);
        assert!(chunks.len() >= 2, "expected >= 2 chunks, got {}", chunks.len());
    }
}
