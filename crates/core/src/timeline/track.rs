use serde::{Deserialize, Serialize};
use uuid::Uuid;

use super::node::StoryLevel;

/// Unique identifier for a track.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub struct TrackId(pub Uuid);

impl TrackId {
    pub fn new() -> Self {
        Self(Uuid::new_v4())
    }
}

/// A track on the timeline, representing one hierarchy level.
///
/// Each story level (Act, Sequence, Scene, Beat) gets its own track.
/// Nodes are NOT stored inside the track — they live in `Timeline.nodes`
/// and are mapped to tracks by matching their `level`.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Track {
    pub id: TrackId,
    pub level: StoryLevel,
    pub label: String,
    pub sort_order: u32,
    /// Whether this track row is collapsed in the UI.
    #[serde(default)]
    pub collapsed: bool,
}

impl Track {
    pub fn new(level: StoryLevel) -> Self {
        let label = match level {
            StoryLevel::Premise => "Premise".to_string(),
            _ => format!("{}s", level.label()),
        };
        Self {
            id: TrackId::new(),
            level,
            label,
            sort_order: level as u32,
            collapsed: false,
        }
    }

    /// Create tracks for all hierarchy levels.
    pub fn default_set() -> Vec<Track> {
        StoryLevel::all()
            .iter()
            .map(|&level| Track::new(level))
            .collect()
    }
}
