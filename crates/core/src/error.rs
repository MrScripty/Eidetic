use uuid::Uuid;

/// Errors produced by eidetic-core operations.
#[derive(Debug, thiserror::Error)]
pub enum Error {
    #[error("track not found: {0}")]
    TrackNotFound(Uuid),

    #[error("node not found: {0}")]
    NodeNotFound(Uuid),

    #[error("arc not found: {0}")]
    ArcNotFound(Uuid),

    #[error("entity not found: {0}")]
    EntityNotFound(Uuid),

    #[error("relationship not found: {0}")]
    RelationshipNotFound(Uuid),

    #[error("time range is invalid (start {start_ms}ms >= end {end_ms}ms)")]
    InvalidTimeRange { start_ms: u64, end_ms: u64 },

    #[error("node exceeds timeline duration ({node_end_ms}ms > {timeline_ms}ms)")]
    NodeExceedsTimeline { node_end_ms: u64, timeline_ms: u64 },

    #[error("split point {split_ms}ms is outside node range {start_ms}ms..{end_ms}ms")]
    SplitOutOfRange {
        split_ms: u64,
        start_ms: u64,
        end_ms: u64,
    },

    #[error("invalid hierarchy: {0}")]
    InvalidHierarchy(String),

    #[error("serialization failed: {0}")]
    Serialization(#[from] serde_json::Error),

    #[error("AI backend error: {0}")]
    AiBackend(String),

    #[error("generation already in progress for node {0}")]
    GenerationInProgress(Uuid),

    #[error("node is locked and cannot be regenerated: {0}")]
    NodeLocked(Uuid),

    #[error("node has no notes to generate from: {0}")]
    NoNotes(Uuid),

    #[error("invalid operation: {0}")]
    InvalidOperation(String),
}

pub type Result<T> = std::result::Result<T, Error>;
