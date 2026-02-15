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
/// Optionally, clips can be decomposed into sub-beats on a collapsible subtrack.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ArcTrack {
    pub id: TrackId,
    /// Which story arc this track represents.
    pub arc_id: ArcId,
    /// Beat clips on this track, ordered by start time.
    pub clips: Vec<BeatClip>,
    /// Sub-beat clips on the beat subtrack, ordered by start time.
    /// Each sub-beat's `parent_clip_id` links it to a clip on the main track.
    #[serde(default)]
    pub sub_beats: Vec<BeatClip>,
    /// Whether the beat subtrack is visible/expanded in the UI.
    #[serde(default)]
    pub sub_beats_visible: bool,
}

impl ArcTrack {
    pub fn new(arc_id: ArcId) -> Self {
        Self {
            id: TrackId::new(),
            arc_id,
            clips: Vec::new(),
            sub_beats: Vec::new(),
            sub_beats_visible: false,
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

    /// Find a sub-beat on this track by ID.
    pub fn sub_beat(&self, id: ClipId) -> Result<&BeatClip> {
        self.sub_beats
            .iter()
            .find(|c| c.id == id)
            .ok_or(Error::ClipNotFound(id.0))
    }

    /// Find a sub-beat on this track by ID (mutable).
    pub fn sub_beat_mut(&mut self, id: ClipId) -> Result<&mut BeatClip> {
        self.sub_beats
            .iter_mut()
            .find(|c| c.id == id)
            .ok_or(Error::ClipNotFound(id.0))
    }

    /// Get all sub-beats belonging to a parent clip.
    pub fn sub_beats_for_clip(&self, parent_id: ClipId) -> Vec<&BeatClip> {
        self.sub_beats
            .iter()
            .filter(|b| b.parent_clip_id == Some(parent_id))
            .collect()
    }

    /// Sort clips by start time.
    pub fn sort_clips(&mut self) {
        self.clips.sort_by_key(|c| c.time_range.start_ms);
    }

    /// Sort sub-beats by start time.
    pub fn sort_sub_beats(&mut self) {
        self.sub_beats.sort_by_key(|c| c.time_range.start_ms);
    }
}
