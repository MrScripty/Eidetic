use serde::{Deserialize, Serialize};
use uuid::Uuid;

use crate::story::arc::ArcId;
use super::timing::TimeRange;

// ──────────────────────────────────────────────
// Node ID
// ──────────────────────────────────────────────

/// Unique identifier for a story node at any hierarchy level.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub struct NodeId(pub Uuid);

impl NodeId {
    pub fn new() -> Self {
        Self(Uuid::new_v4())
    }
}

/// Temporary alias during migration — old code referencing ClipId still works.
pub type ClipId = NodeId;

// ──────────────────────────────────────────────
// Story Level
// ──────────────────────────────────────────────

/// The hierarchy level a node occupies on the timeline.
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash, Serialize, Deserialize)]
pub enum StoryLevel {
    Premise = 0,
    Act = 1,
    Sequence = 2,
    Scene = 3,
    Beat = 4,
}

impl StoryLevel {
    /// The level directly below this one in the hierarchy.
    pub fn child_level(&self) -> Option<StoryLevel> {
        match self {
            Self::Premise => Some(Self::Act),
            Self::Act => Some(Self::Sequence),
            Self::Sequence => Some(Self::Scene),
            Self::Scene => Some(Self::Beat),
            Self::Beat => None,
        }
    }

    /// The level directly above this one.
    pub fn parent_level(&self) -> Option<StoryLevel> {
        match self {
            Self::Premise => None,
            Self::Act => Some(Self::Premise),
            Self::Sequence => Some(Self::Act),
            Self::Scene => Some(Self::Sequence),
            Self::Beat => Some(Self::Scene),
        }
    }

    /// Human-readable label for this level.
    pub fn label(&self) -> &'static str {
        match self {
            Self::Premise => "Premise",
            Self::Act => "Act",
            Self::Sequence => "Sequence",
            Self::Scene => "Scene",
            Self::Beat => "Beat",
        }
    }

    /// Human-readable plural label for child level.
    pub fn children_label(&self) -> Option<&'static str> {
        match self {
            Self::Premise => Some("Acts"),
            Self::Act => Some("Sequences"),
            Self::Sequence => Some("Scenes"),
            Self::Scene => Some("Beats"),
            Self::Beat => None,
        }
    }

    /// All levels in hierarchy order.
    pub fn all() -> &'static [StoryLevel] {
        &[Self::Premise, Self::Act, Self::Sequence, Self::Scene, Self::Beat]
    }
}

impl std::fmt::Display for StoryLevel {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.label())
    }
}

// ──────────────────────────────────────────────
// Beat Type (moved from clip.rs)
// ──────────────────────────────────────────────

/// The type of narrative beat a node represents (only meaningful at Beat level).
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

// ──────────────────────────────────────────────
// Content Status (moved from clip.rs)
// ──────────────────────────────────────────────

/// Tracks the content lifecycle of a story node.
///
/// Attribution (who wrote what) lives in the Y.Doc CRDT layer, not here.
/// This enum tracks the high-level state for UI display and AI decision-making.
#[derive(Debug, Clone, Copy, Default, PartialEq, Eq, Serialize, Deserialize)]
pub enum ContentStatus {
    /// No content yet.
    #[default]
    Empty,
    /// Notes present, no script/outline content.
    NotesOnly,
    /// AI is currently generating.
    Generating,
    /// Content present (attribution in Y.Doc distinguishes human vs AI).
    /// Aliases map legacy variants from v2 schema.
    #[serde(alias = "Generated", alias = "UserRefined", alias = "UserWritten")]
    HasContent,
}

// ──────────────────────────────────────────────
// Node Content
// ──────────────────────────────────────────────

/// Content at any hierarchy level, progressing through stages.
///
/// With the CRDT layer, `notes` and `content` are cached from Y.Doc.
/// The Y.Doc is the single source of truth for text; these fields are
/// populated at load time and refreshed via sync.
#[derive(Debug, Clone, Default, Serialize)]
pub struct NodeContent {
    /// User's description/notes for this node (cached from Y.Doc).
    pub notes: String,
    /// Script/outline text (cached from Y.Doc). Replaces the old
    /// `generated_text` + `user_refined_text` split — attribution now
    /// lives in Y.Doc text attributes.
    #[serde(default)]
    pub content: String,
    /// Derived from Y.Doc state; tracked externally during generation.
    pub status: ContentStatus,
    /// Compact structured recap for continuity context (primarily Scene/Beat levels).
    /// Server-computed, not CRDT-managed.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub scene_recap: Option<String>,
}

/// Custom Deserialize for NodeContent that handles both the new schema
/// (`content` field) and legacy v2 schema (`generated_text` + `user_refined_text`).
impl<'de> Deserialize<'de> for NodeContent {
    fn deserialize<D>(deserializer: D) -> std::result::Result<Self, D::Error>
    where
        D: serde::Deserializer<'de>,
    {
        #[derive(Deserialize)]
        struct Raw {
            #[serde(default)]
            notes: String,
            #[serde(default)]
            content: String,
            // Legacy fields — only present in v2 JSON.
            #[serde(default)]
            generated_text: Option<String>,
            #[serde(default)]
            user_refined_text: Option<String>,
            #[serde(default)]
            status: ContentStatus,
            #[serde(default)]
            scene_recap: Option<String>,
        }

        let raw = Raw::deserialize(deserializer)?;

        // Migrate: if `content` is empty, populate from legacy fields.
        let content = if raw.content.is_empty() {
            raw.user_refined_text
                .or(raw.generated_text)
                .unwrap_or_default()
        } else {
            raw.content
        };

        Ok(NodeContent {
            notes: raw.notes,
            content,
            status: raw.status,
            scene_recap: raw.scene_recap,
        })
    }
}

// ──────────────────────────────────────────────
// Story Node
// ──────────────────────────────────────────────

/// A story node at any level of the hierarchy (Premise, Act, Sequence, Scene, or Beat).
///
/// All levels share the same struct — they differ by `level` and `parent_id`.
/// Nodes form a tree: Premise contains Acts, Acts contain Sequences,
/// Sequences contain Scenes, Scenes contain Beats.
/// The `parent_id` links each node to its parent.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct StoryNode {
    pub id: NodeId,
    /// Parent in the hierarchy. Premise has no parent (None).
    /// Acts point to Premise, Sequences to an Act, Scenes to a Sequence, Beats to a Scene.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub parent_id: Option<NodeId>,
    pub level: StoryLevel,
    pub sort_order: u32,
    pub time_range: TimeRange,
    pub name: String,
    pub content: NodeContent,
    /// Only meaningful at Beat level; None for Act/Sequence/Scene.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub beat_type: Option<BeatType>,
    /// If true, AI won't regenerate this node's content.
    pub locked: bool,
}

impl StoryNode {
    pub fn new(
        name: impl Into<String>,
        level: StoryLevel,
        time_range: TimeRange,
    ) -> Self {
        Self {
            id: NodeId::new(),
            parent_id: None,
            level,
            sort_order: 0,
            time_range,
            name: name.into(),
            content: NodeContent::default(),
            beat_type: None,
            locked: false,
        }
    }

    pub fn new_beat(
        name: impl Into<String>,
        beat_type: BeatType,
        time_range: TimeRange,
        parent_id: NodeId,
    ) -> Self {
        Self {
            id: NodeId::new(),
            parent_id: Some(parent_id),
            level: StoryLevel::Beat,
            sort_order: 0,
            time_range,
            name: name.into(),
            content: NodeContent::default(),
            beat_type: Some(beat_type),
            locked: false,
        }
    }

    pub fn new_child(
        name: impl Into<String>,
        level: StoryLevel,
        time_range: TimeRange,
        parent_id: NodeId,
    ) -> Self {
        Self {
            id: NodeId::new(),
            parent_id: Some(parent_id),
            level,
            sort_order: 0,
            time_range,
            name: name.into(),
            content: NodeContent::default(),
            beat_type: None,
            locked: false,
        }
    }

    /// Get the best available text content (content > notes).
    pub fn best_text(&self) -> &str {
        if !self.content.content.is_empty() {
            &self.content.content
        } else {
            &self.content.notes
        }
    }
}

// ──────────────────────────────────────────────
// Node-Arc Junction
// ──────────────────────────────────────────────

/// A many-to-many association between a StoryNode and a StoryArc.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct NodeArc {
    pub node_id: NodeId,
    pub arc_id: ArcId,
}
