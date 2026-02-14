use serde::{Deserialize, Serialize};

use crate::story::arc::ArcId;
use super::clip::ClipId;
use super::timing::TimeRange;
use super::Timeline;

/// An inferred scene — derived from vertical overlap of beat clips across arc tracks.
///
/// Scenes are NOT explicitly created by the user. They emerge from the timeline:
/// wherever the active combination of arcs changes, a scene boundary appears.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct InferredScene {
    pub time_range: TimeRange,
    /// The arcs active during this scene (one per overlapping track).
    pub active_arcs: Vec<ArcId>,
    /// The specific clips contributing to this scene.
    pub contributing_clips: Vec<ClipId>,
}

impl Timeline {
    /// Infer scenes from the vertical overlap of beat clips across tracks.
    ///
    /// Algorithm: collect all clip start/end times as "events," sweep left-to-right,
    /// and emit a new scene whenever the set of active arcs changes.
    pub fn infer_scenes(&self) -> Vec<InferredScene> {
        // Collect all boundary times from clip edges.
        let mut boundaries: Vec<u64> = Vec::new();
        for track in &self.tracks {
            for clip in &track.clips {
                boundaries.push(clip.time_range.start_ms);
                boundaries.push(clip.time_range.end_ms);
            }
        }
        boundaries.sort_unstable();
        boundaries.dedup();

        if boundaries.len() < 2 {
            return Vec::new();
        }

        let mut scenes = Vec::new();

        // Sweep adjacent boundary pairs.
        for window in boundaries.windows(2) {
            let start = window[0];
            let end = window[1];
            if start == end {
                continue;
            }

            // Use the midpoint to sample which clips are active in this interval.
            let mid = start + (end - start) / 2;
            let active = self.clips_at(mid);

            if active.is_empty() {
                continue;
            }

            let active_arcs: Vec<ArcId> = active.iter().map(|(track, _)| track.arc_id).collect();
            let contributing_clips: Vec<ClipId> = active.iter().map(|(_, clip)| clip.id).collect();

            // Merge with previous scene if the same arc combination is active.
            if let Some(last) = scenes.last_mut() {
                let last_scene: &mut InferredScene = last;
                if last_scene.active_arcs == active_arcs
                    && last_scene.time_range.end_ms == start
                {
                    last_scene.time_range.end_ms = end;
                    // Update contributing clips (may have changed within merged range).
                    last_scene.contributing_clips = contributing_clips;
                    continue;
                }
            }

            scenes.push(InferredScene {
                // Safety: start < end guaranteed by the dedup + skip-if-equal check above.
                time_range: TimeRange { start_ms: start, end_ms: end },
                active_arcs,
                contributing_clips,
            });
        }

        scenes
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::timeline::clip::{BeatClip, BeatType};
    use crate::timeline::structure::EpisodeStructure;
    use crate::timeline::track::ArcTrack;

    fn make_timeline_with_overlapping_clips() -> Timeline {
        let arc_a = ArcId::new();
        let arc_b = ArcId::new();

        let mut track_a = ArcTrack::new(arc_a);
        track_a.clips.push(BeatClip::new(
            "A setup",
            BeatType::Setup,
            TimeRange { start_ms: 0, end_ms: 120_000 },
        ));
        track_a.clips.push(BeatClip::new(
            "A complication",
            BeatType::Complication,
            TimeRange { start_ms: 150_000, end_ms: 400_000 },
        ));

        let mut track_b = ArcTrack::new(arc_b);
        track_b.clips.push(BeatClip::new(
            "B setup",
            BeatType::Setup,
            TimeRange { start_ms: 100_000, end_ms: 250_000 },
        ));

        let mut timeline = Timeline::new(1_320_000, EpisodeStructure::standard_30_min());
        timeline.tracks.push(track_a);
        timeline.tracks.push(track_b);
        timeline
    }

    #[test]
    fn test_infer_scenes_empty_timeline_returns_empty() {
        let timeline = Timeline::new(1_320_000, EpisodeStructure::standard_30_min());
        assert!(timeline.infer_scenes().is_empty());
    }

    #[test]
    fn test_infer_scenes_overlapping_clips_produces_multiple_scenes() {
        let timeline = make_timeline_with_overlapping_clips();
        let scenes = timeline.infer_scenes();

        // Expected boundaries: 0, 100k, 120k, 150k, 250k, 400k
        // Intervals with activity:
        //   0..100k     — A only
        //   100k..120k  — A + B (overlap)
        //   150k..250k  — A + B (overlap)
        //   250k..400k  — A only
        assert!(scenes.len() >= 4, "expected at least 4 scenes, got {}", scenes.len());

        // The overlap scene at 100k..120k should have 2 arcs.
        let overlap = scenes.iter().find(|s| s.time_range.start_ms == 100_000);
        assert!(overlap.is_some());
        assert_eq!(overlap.unwrap().active_arcs.len(), 2);
    }

    #[test]
    fn test_infer_scenes_solo_clip_produces_single_scene() {
        let arc_a = ArcId::new();
        let mut track = ArcTrack::new(arc_a);
        track.clips.push(BeatClip::new(
            "solo",
            BeatType::Setup,
            TimeRange { start_ms: 0, end_ms: 60_000 },
        ));

        let mut timeline = Timeline::new(1_320_000, EpisodeStructure::standard_30_min());
        timeline.tracks.push(track);

        let scenes = timeline.infer_scenes();
        assert_eq!(scenes.len(), 1);
        assert_eq!(scenes[0].active_arcs.len(), 1);
    }
}
