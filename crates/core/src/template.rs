use crate::project::Project;
use crate::story::arc::{ArcType, Color, StoryArc};
use crate::timeline::clip::{BeatClip, BeatType};
use crate::timeline::structure::EpisodeStructure;
use crate::timeline::timing::TimeRange;
use crate::timeline::track::ArcTrack;
use crate::timeline::Timeline;

/// Pre-configured project templates for different TV subgenres.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Template {
    /// Multi-cam sitcom (Seinfeld-style): rapid A/B cutting, shorter beats.
    MultiCam,
    /// Single-cam dramedy (Scrubs-style): longer, flowing scenes.
    SingleCam,
    /// Animated comedy (Bob's Burgers-style): flexible structure, C-runner emphasis.
    Animated,
}

/// Total duration of a standard 30-min TV episode's content: 22 minutes.
const EPISODE_DURATION_MS: u64 = 1_320_000;

impl Template {
    /// Build a new project from this template with pre-placed arcs and beat clips.
    pub fn build_project(self, name: impl Into<String>) -> Project {
        let structure = EpisodeStructure::standard_30_min();
        let mut timeline = Timeline::new(EPISODE_DURATION_MS, structure);

        let a_arc = StoryArc::new("A-Plot", ArcType::APlot, Color::A_PLOT);
        let b_arc = StoryArc::new("B-Plot", ArcType::BPlot, Color::B_PLOT);
        let c_arc = StoryArc::new("C-Runner", ArcType::CRunner, Color::C_RUNNER);

        let a_track = self.build_a_track(a_arc.id);
        let b_track = self.build_b_track(b_arc.id);
        let c_track = self.build_c_track(c_arc.id);

        // These won't fail: arcs are unique, clips fit within the timeline.
        timeline.tracks.push(a_track);
        timeline.tracks.push(b_track);
        timeline.tracks.push(c_track);

        let mut project = Project::new(name, timeline);
        project.arcs.push(a_arc);
        project.arcs.push(b_arc);
        project.arcs.push(c_arc);
        project
    }

    fn build_a_track(self, arc_id: crate::story::arc::ArcId) -> ArcTrack {
        let mut track = ArcTrack::new(arc_id);
        let beats = match self {
            Self::MultiCam => vec![
                ("Setup", BeatType::Setup, 0, 90_000),
                ("Complication", BeatType::Complication, 150_000, 360_000),
                ("Escalation", BeatType::Escalation, 570_000, 780_000),
                ("Climax", BeatType::Climax, 990_000, 1_170_000),
                ("Resolution", BeatType::Resolution, 1_200_000, 1_290_000),
            ],
            Self::SingleCam => vec![
                ("Setup", BeatType::Setup, 0, 120_000),
                ("Complication", BeatType::Complication, 200_000, 450_000),
                ("Escalation", BeatType::Escalation, 570_000, 820_000),
                ("Climax", BeatType::Climax, 990_000, 1_200_000),
                ("Resolution", BeatType::Resolution, 1_220_000, 1_290_000),
            ],
            Self::Animated => vec![
                ("Setup", BeatType::Setup, 0, 100_000),
                ("Complication", BeatType::Complication, 180_000, 400_000),
                ("Escalation", BeatType::Escalation, 570_000, 800_000),
                ("Climax", BeatType::Climax, 990_000, 1_180_000),
                ("Resolution", BeatType::Resolution, 1_200_000, 1_290_000),
            ],
        };
        for (name, beat_type, start, end) in beats {
            track.clips.push(BeatClip::new(
                name,
                beat_type,
                TimeRange { start_ms: start, end_ms: end },
            ));
        }
        track
    }

    fn build_b_track(self, arc_id: crate::story::arc::ArcId) -> ArcTrack {
        let mut track = ArcTrack::new(arc_id);
        let beats = match self {
            Self::MultiCam => vec![
                ("Setup", BeatType::Setup, 90_000, 240_000),
                ("Complication", BeatType::Complication, 360_000, 540_000),
                ("Payoff", BeatType::Payoff, 840_000, 990_000),
            ],
            Self::SingleCam => vec![
                ("Setup", BeatType::Setup, 120_000, 300_000),
                ("Complication", BeatType::Complication, 450_000, 680_000),
                ("Payoff", BeatType::Payoff, 900_000, 1_050_000),
            ],
            Self::Animated => vec![
                ("Setup", BeatType::Setup, 100_000, 260_000),
                ("Complication", BeatType::Complication, 400_000, 600_000),
                ("Payoff", BeatType::Payoff, 850_000, 1_000_000),
            ],
        };
        for (name, beat_type, start, end) in beats {
            track.clips.push(BeatClip::new(
                name,
                beat_type,
                TimeRange { start_ms: start, end_ms: end },
            ));
        }
        track
    }

    fn build_c_track(self, arc_id: crate::story::arc::ArcId) -> ArcTrack {
        let mut track = ArcTrack::new(arc_id);
        let beats = match self {
            Self::MultiCam => vec![
                ("Beat", BeatType::Setup, 60_000, 120_000),
                ("Beat", BeatType::Callback, 480_000, 540_000),
                ("Callback", BeatType::Payoff, 1_170_000, 1_230_000),
            ],
            Self::SingleCam => vec![
                ("Beat", BeatType::Setup, 50_000, 110_000),
                ("Beat", BeatType::Callback, 500_000, 560_000),
                ("Callback", BeatType::Payoff, 1_150_000, 1_220_000),
            ],
            Self::Animated => vec![
                ("Beat", BeatType::Setup, 30_000, 100_000),
                ("Beat", BeatType::Callback, 400_000, 480_000),
                ("Beat", BeatType::Callback, 700_000, 770_000),
                ("Callback", BeatType::Payoff, 1_100_000, 1_180_000),
            ],
        };
        for (name, beat_type, start, end) in beats {
            track.clips.push(BeatClip::new(
                name,
                beat_type,
                TimeRange { start_ms: start, end_ms: end },
            ));
        }
        track
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_multicam_template_builds_valid_project() {
        let project = Template::MultiCam.build_project("Test Episode");
        assert_eq!(project.name, "Test Episode");
        assert_eq!(project.arcs.len(), 3);
        assert_eq!(project.timeline.tracks.len(), 3);

        // A-plot has 5 beats.
        assert_eq!(project.timeline.tracks[0].clips.len(), 5);
        // B-plot has 3 beats.
        assert_eq!(project.timeline.tracks[1].clips.len(), 3);
        // C-runner has 3 beats.
        assert_eq!(project.timeline.tracks[2].clips.len(), 3);
    }

    #[test]
    fn test_animated_template_c_runner_has_four_beats() {
        let project = Template::Animated.build_project("Animated Test");
        // Animated C-runner has 4 beats (extra beat for emphasis).
        assert_eq!(project.timeline.tracks[2].clips.len(), 4);
    }

    #[test]
    fn test_all_clips_within_timeline_duration() {
        for template in [Template::MultiCam, Template::SingleCam, Template::Animated] {
            let project = template.build_project("Duration Test");
            let max_end = project.timeline.total_duration_ms;
            for track in &project.timeline.tracks {
                for clip in &track.clips {
                    assert!(
                        clip.time_range.end_ms <= max_end,
                        "clip '{}' exceeds timeline: {} > {}",
                        clip.name,
                        clip.time_range.end_ms,
                        max_end,
                    );
                }
            }
        }
    }

    #[test]
    fn test_template_scenes_infer_correctly() {
        let project = Template::MultiCam.build_project("Scene Test");
        let scenes = project.timeline.infer_scenes();
        assert!(!scenes.is_empty(), "multicam template should produce inferred scenes");
    }
}
