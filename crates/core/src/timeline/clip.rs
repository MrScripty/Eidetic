use serde::{Deserialize, Serialize};
use uuid::Uuid;

use super::timing::TimeRange;

/// Unique identifier for a beat clip.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub struct ClipId(pub Uuid);

impl ClipId {
    pub fn new() -> Self {
        Self(Uuid::new_v4())
    }
}

/// A beat clip on an arc track — a narrative turning point with a time range.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct BeatClip {
    pub id: ClipId,
    pub time_range: TimeRange,
    pub beat_type: BeatType,
    pub name: String,
    pub content: BeatContent,
    /// If true, AI won't regenerate this clip's script.
    pub locked: bool,
}

impl BeatClip {
    pub fn new(name: impl Into<String>, beat_type: BeatType, time_range: TimeRange) -> Self {
        Self {
            id: ClipId::new(),
            time_range,
            beat_type,
            name: name.into(),
            content: BeatContent::default(),
            locked: false,
        }
    }
}

/// The type of narrative beat this clip represents.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub enum BeatType {
    Setup,
    Complication,
    Escalation,
    Climax,
    Resolution,
    Payoff,
    Callback,
    Custom(String),
}

/// The content of a beat clip, progressing through stages.
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct BeatContent {
    /// User's markdown description of what happens in this beat.
    pub beat_notes: String,
    /// AI-generated screenplay text from the beat notes.
    pub generated_script: Option<String>,
    /// User's edits to the generated script.
    pub user_refined_script: Option<String>,
    pub status: ContentStatus,
    /// Compact structured recap of the scene's end state, generated after
    /// script generation. Used as continuity context for subsequent clips.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub scene_recap: Option<String>,
}

/// Tracks the content lifecycle of a beat clip.
#[derive(Debug, Clone, Copy, Default, PartialEq, Eq, Serialize, Deserialize)]
pub enum ContentStatus {
    /// No content yet.
    #[default]
    Empty,
    /// User has written beat notes, no script generated.
    NotesOnly,
    /// AI is currently generating script.
    Generating,
    /// AI has generated script from beat notes.
    Generated,
    /// User has edited the generated script.
    UserRefined,
    /// User wrote the script directly (no AI generation).
    UserWritten,
}
