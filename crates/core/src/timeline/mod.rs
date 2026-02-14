pub mod clip;
pub mod relationship;
pub mod scene;
pub mod structure;
pub mod timing;
pub mod track;

use serde::{Deserialize, Serialize};

use crate::error::{Error, Result};
use crate::story::arc::ArcId;
use clip::{BeatClip, ClipId};
use relationship::{Relationship, RelationshipId};
use structure::EpisodeStructure;
use timing::TimeRange;
use track::{ArcTrack, TrackId};

/// A gap on a track where no beat clip exists.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TimelineGap {
    pub track_id: TrackId,
    pub arc_id: ArcId,
    pub time_range: TimeRange,
    pub preceding_clip_id: Option<ClipId>,
    pub following_clip_id: Option<ClipId>,
}

/// The central data structure: a timeline with arc tracks and episode structure.
///
/// Represents the full runtime of an episode (~22 min for 30-min TV). Arc tracks
/// hold beat clips; relationships connect clips across tracks; the structure bar
/// marks act breaks.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Timeline {
    /// Total episode duration (typically ~22 min content for a 30-min slot).
    pub total_duration_ms: u64,
    /// One track per story arc, each containing beat clips.
    pub tracks: Vec<ArcTrack>,
    /// Edge-bundled curves connecting clips across tracks.
    pub relationships: Vec<Relationship>,
    /// Act structure (cold open, acts, commercial breaks, tag).
    pub structure: EpisodeStructure,
}

impl Timeline {
    /// Create a new empty timeline with the given duration and structure.
    pub fn new(total_duration_ms: u64, structure: EpisodeStructure) -> Self {
        Self {
            total_duration_ms,
            tracks: Vec::new(),
            relationships: Vec::new(),
            structure,
        }
    }

    /// Add an arc track. Fails if this arc already has a track.
    pub fn add_track(&mut self, track: ArcTrack) -> Result<()> {
        if self.tracks.iter().any(|t| t.arc_id == track.arc_id) {
            return Err(Error::DuplicateArcTrack(track.arc_id.0));
        }
        self.tracks.push(track);
        Ok(())
    }

    /// Remove a track by ID, returning it. Also removes relationships that
    /// reference clips on this track.
    pub fn remove_track(&mut self, id: TrackId) -> Result<ArcTrack> {
        let idx = self
            .tracks
            .iter()
            .position(|t| t.id == id)
            .ok_or(Error::TrackNotFound(id.0))?;

        let track = self.tracks.remove(idx);

        let clip_ids: Vec<ClipId> = track.clips.iter().map(|c| c.id).collect();
        self.relationships
            .retain(|r| !clip_ids.contains(&r.from_clip) && !clip_ids.contains(&r.to_clip));

        Ok(track)
    }

    /// Find a track by ID.
    pub fn track(&self, id: TrackId) -> Result<&ArcTrack> {
        self.tracks
            .iter()
            .find(|t| t.id == id)
            .ok_or(Error::TrackNotFound(id.0))
    }

    /// Find a track by ID (mutable).
    pub fn track_mut(&mut self, id: TrackId) -> Result<&mut ArcTrack> {
        self.tracks
            .iter_mut()
            .find(|t| t.id == id)
            .ok_or(Error::TrackNotFound(id.0))
    }

    /// Find the track containing a specific clip.
    pub fn track_for_clip(&self, clip_id: ClipId) -> Option<&ArcTrack> {
        self.tracks
            .iter()
            .find(|t| t.clips.iter().any(|c| c.id == clip_id))
    }

    /// Add a clip to a track, validating it fits within the timeline duration.
    pub fn add_clip(&mut self, track_id: TrackId, clip: BeatClip) -> Result<()> {
        if clip.time_range.end_ms > self.total_duration_ms {
            return Err(Error::ClipExceedsTimeline {
                clip_end_ms: clip.time_range.end_ms,
                timeline_ms: self.total_duration_ms,
            });
        }
        clip.time_range.validate()?;
        self.track_mut(track_id)?.clips.push(clip);
        Ok(())
    }

    /// Move a clip to a new time range.
    pub fn move_clip(&mut self, clip_id: ClipId, new_range: TimeRange) -> Result<()> {
        new_range.validate()?;
        if new_range.end_ms > self.total_duration_ms {
            return Err(Error::ClipExceedsTimeline {
                clip_end_ms: new_range.end_ms,
                timeline_ms: self.total_duration_ms,
            });
        }
        let clip = self.clip_mut(clip_id)?;
        clip.time_range = new_range;
        Ok(())
    }

    /// Remove a clip by ID from any track. Also removes relationships referencing it.
    pub fn remove_clip(&mut self, clip_id: ClipId) -> Result<BeatClip> {
        for track in &mut self.tracks {
            if let Some(idx) = track.clips.iter().position(|c| c.id == clip_id) {
                let clip = track.clips.remove(idx);
                self.relationships
                    .retain(|r| r.from_clip != clip_id && r.to_clip != clip_id);
                return Ok(clip);
            }
        }
        Err(Error::ClipNotFound(clip_id.0))
    }

    /// Find a clip by ID across all tracks.
    pub fn clip(&self, id: ClipId) -> Result<&BeatClip> {
        self.tracks
            .iter()
            .flat_map(|t| &t.clips)
            .find(|c| c.id == id)
            .ok_or(Error::ClipNotFound(id.0))
    }

    /// Find a clip by ID across all tracks (mutable).
    pub fn clip_mut(&mut self, id: ClipId) -> Result<&mut BeatClip> {
        self.tracks
            .iter_mut()
            .flat_map(|t| &mut t.clips)
            .find(|c| c.id == id)
            .ok_or(Error::ClipNotFound(id.0))
    }

    /// Add a relationship between two clips.
    pub fn add_relationship(&mut self, rel: Relationship) -> Result<()> {
        // Verify both clips exist.
        self.clip(rel.from_clip)?;
        self.clip(rel.to_clip)?;
        self.relationships.push(rel);
        Ok(())
    }

    /// Remove a relationship by ID.
    pub fn remove_relationship(&mut self, id: RelationshipId) -> Result<Relationship> {
        let idx = self
            .relationships
            .iter()
            .position(|r| r.id == id)
            .ok_or(Error::RelationshipNotFound(id.0))?;
        Ok(self.relationships.remove(idx))
    }

    /// Get all clips that overlap a given time position (vertical slice).
    pub fn clips_at(&self, time_ms: u64) -> Vec<(&ArcTrack, &BeatClip)> {
        self.tracks
            .iter()
            .flat_map(|track| {
                track
                    .clips
                    .iter()
                    .filter(move |clip| clip.time_range.contains(time_ms))
                    .map(move |clip| (track, clip))
            })
            .collect()
    }

    /// Split a clip at the given time point, producing two clips.
    /// Returns the IDs of the two resulting clips.
    pub fn split_clip(&mut self, clip_id: ClipId, at_ms: u64) -> Result<(ClipId, ClipId)> {
        let clip = self.clip(clip_id)?;
        let range = clip.time_range;

        if at_ms <= range.start_ms || at_ms >= range.end_ms {
            return Err(Error::SplitOutOfRange {
                split_ms: at_ms,
                start_ms: range.start_ms,
                end_ms: range.end_ms,
            });
        }

        // Clone fields we need before mutating.
        let beat_type = clip.beat_type.clone();
        let name = clip.name.clone();
        let locked = clip.locked;

        let left_id = ClipId::new();
        let right_id = ClipId::new();

        let left = BeatClip {
            id: left_id,
            time_range: TimeRange::new(range.start_ms, at_ms)?,
            beat_type: beat_type.clone(),
            name: format!("{} (L)", name),
            content: clip::BeatContent::default(),
            locked,
        };

        let right = BeatClip {
            id: right_id,
            time_range: TimeRange::new(at_ms, range.end_ms)?,
            beat_type,
            name: format!("{} (R)", name),
            content: clip::BeatContent::default(),
            locked,
        };

        // Find which track the clip is on and replace it.
        for track in &mut self.tracks {
            if let Some(idx) = track.clips.iter().position(|c| c.id == clip_id) {
                track.clips.remove(idx);
                track.clips.insert(idx, left);
                track.clips.insert(idx + 1, right);

                // Repoint relationships from the old clip to the left half.
                for rel in &mut self.relationships {
                    if rel.from_clip == clip_id {
                        rel.from_clip = left_id;
                    }
                    if rel.to_clip == clip_id {
                        rel.to_clip = right_id;
                    }
                }

                return Ok((left_id, right_id));
            }
        }

        Err(Error::ClipNotFound(clip_id.0))
    }

    /// Close a specific gap on a track by shifting all clips after it leftward.
    ///
    /// `gap_end_ms` is the start time of the first clip after the gap. All clips
    /// starting at or after this point are shifted left by the gap's duration.
    pub fn close_gap(&mut self, track_id: TrackId, gap_end_ms: u64) -> Result<()> {
        let track = self.track_mut(track_id)?;
        let mut sorted_indices: Vec<usize> = (0..track.clips.len()).collect();
        sorted_indices.sort_by_key(|&i| track.clips[i].time_range.start_ms);

        // Find the gap: the space between the last clip ending before gap_end_ms and gap_end_ms.
        let mut gap_start_ms: u64 = 0;
        for &i in &sorted_indices {
            let end = track.clips[i].time_range.end_ms;
            if end <= gap_end_ms {
                gap_start_ms = gap_start_ms.max(end);
            }
        }

        if gap_start_ms >= gap_end_ms {
            return Ok(()); // No gap to close.
        }

        let shift = gap_end_ms - gap_start_ms;

        for clip in &mut track.clips {
            if clip.time_range.start_ms >= gap_end_ms {
                clip.time_range.start_ms -= shift;
                clip.time_range.end_ms -= shift;
            }
        }

        Ok(())
    }

    /// Close all gaps on a track, making clips contiguous from the first clip's position.
    pub fn close_all_gaps(&mut self, track_id: TrackId) -> Result<()> {
        let track = self.track_mut(track_id)?;
        track.clips.sort_by_key(|c| c.time_range.start_ms);

        let mut cursor = match track.clips.first() {
            Some(c) => c.time_range.start_ms,
            None => return Ok(()),
        };

        for clip in &mut track.clips {
            let duration = clip.time_range.end_ms - clip.time_range.start_ms;
            clip.time_range.start_ms = cursor;
            clip.time_range.end_ms = cursor + duration;
            cursor += duration;
        }

        Ok(())
    }

    /// Find gaps on all tracks where no beat clips exist.
    ///
    /// Returns gaps longer than `min_duration_ms` between consecutive clips
    /// and between the timeline edges and the first/last clips.
    pub fn find_gaps(&self, min_duration_ms: u64) -> Vec<TimelineGap> {
        let mut gaps = Vec::new();

        for track in &self.tracks {
            let mut sorted: Vec<&BeatClip> = track.clips.iter().collect();
            sorted.sort_by_key(|c| c.time_range.start_ms);

            let mut cursor = 0u64;
            let mut prev_clip_id: Option<ClipId> = None;

            for clip in &sorted {
                if clip.time_range.start_ms > cursor {
                    let duration = clip.time_range.start_ms - cursor;
                    if duration >= min_duration_ms {
                        if let Ok(range) = TimeRange::new(cursor, clip.time_range.start_ms) {
                            gaps.push(TimelineGap {
                                track_id: track.id,
                                arc_id: track.arc_id,
                                time_range: range,
                                preceding_clip_id: prev_clip_id,
                                following_clip_id: Some(clip.id),
                            });
                        }
                    }
                }
                cursor = clip.time_range.end_ms;
                prev_clip_id = Some(clip.id);
            }

            // Gap between last clip and timeline end.
            if cursor < self.total_duration_ms {
                let duration = self.total_duration_ms - cursor;
                if duration >= min_duration_ms {
                    if let Ok(range) = TimeRange::new(cursor, self.total_duration_ms) {
                        gaps.push(TimelineGap {
                            track_id: track.id,
                            arc_id: track.arc_id,
                            time_range: range,
                            preceding_clip_id: prev_clip_id,
                            following_clip_id: None,
                        });
                    }
                }
            }
        }

        gaps
    }
}
