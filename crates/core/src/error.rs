use uuid::Uuid;

/// Errors produced by eidetic-core operations.
#[derive(Debug, thiserror::Error)]
pub enum Error {
    #[error("track not found: {0}")]
    TrackNotFound(Uuid),

    #[error("clip not found: {0}")]
    ClipNotFound(Uuid),

    #[error("arc not found: {0}")]
    ArcNotFound(Uuid),

    #[error("character not found: {0}")]
    CharacterNotFound(Uuid),

    #[error("relationship not found: {0}")]
    RelationshipNotFound(Uuid),

    #[error("clip time range is invalid (start {start_ms}ms >= end {end_ms}ms)")]
    InvalidTimeRange { start_ms: u64, end_ms: u64 },

    #[error("clip exceeds timeline duration ({clip_end_ms}ms > {timeline_ms}ms)")]
    ClipExceedsTimeline { clip_end_ms: u64, timeline_ms: u64 },

    #[error("split point {split_ms}ms is outside clip range {start_ms}ms..{end_ms}ms")]
    SplitOutOfRange {
        split_ms: u64,
        start_ms: u64,
        end_ms: u64,
    },

    #[error("duplicate arc on timeline: arc {0} already has a track")]
    DuplicateArcTrack(Uuid),

    #[error("serialization failed: {0}")]
    Serialization(#[from] serde_json::Error),

    #[error("AI backend error: {0}")]
    AiBackend(String),

    #[error("generation already in progress for clip {0}")]
    GenerationInProgress(Uuid),

    #[error("clip is locked and cannot be regenerated: {0}")]
    ClipLocked(Uuid),

    #[error("clip has no beat notes to generate from: {0}")]
    NoBeatNotes(Uuid),
}

pub type Result<T> = std::result::Result<T, Error>;
