use serde::{Deserialize, Serialize};
use uuid::Uuid;

use crate::error::{Error, Result};
use crate::story::arc::ArcId;
use super::clip::{BeatClip, ClipId};

/// Unique identifier for an arc track.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub struct TrackId(pub Uuid);

impl TrackId {
    pub fn new() -> Self {
        Self(Uuid::new_v4())
    }
}

/// A track belonging to a story arc, containing beat clips.
///
/// Each story arc (A-plot, B-plot, C-runner) gets its own track on the timeline.
/// Beat clips within the track represent the narrative turning points of that arc.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ArcTrack {
    pub id: TrackId,
    /// Which story arc this track represents.
    pub arc_id: ArcId,
    /// Beat clips on this track, ordered by start time.
    pub clips: Vec<BeatClip>,
}

impl ArcTrack {
    pub fn new(arc_id: ArcId) -> Self {
        Self {
            id: TrackId::new(),
            arc_id,
            clips: Vec::new(),
        }
    }

    /// Find a clip on this track by ID.
    pub fn clip(&self, id: ClipId) -> Result<&BeatClip> {
        self.clips
            .iter()
            .find(|c| c.id == id)
            .ok_or(Error::ClipNotFound(id.0))
    }

    /// Find a clip on this track by ID (mutable).
    pub fn clip_mut(&mut self, id: ClipId) -> Result<&mut BeatClip> {
        self.clips
            .iter_mut()
            .find(|c| c.id == id)
            .ok_or(Error::ClipNotFound(id.0))
    }

    /// Sort clips by start time.
    pub fn sort_clips(&mut self) {
        self.clips.sort_by_key(|c| c.time_range.start_ms);
    }
}
