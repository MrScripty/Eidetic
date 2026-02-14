use serde::{Deserialize, Serialize};
use uuid::Uuid;

use crate::project::Project;
use crate::timeline::clip::BeatType;

/// Severity of a progression issue.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum Severity {
    Warning,
    Error,
}

/// A specific issue found in an arc's progression.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ProgressionIssue {
    pub severity: Severity,
    pub message: String,
}

/// Analysis result for a single story arc's progression.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ArcProgression {
    pub arc_id: Uuid,
    pub arc_name: String,
    pub beat_count: usize,
    pub has_setup: bool,
    pub has_resolution: bool,
    pub coverage_percent: f64,
    pub longest_gap_ms: u64,
    pub issues: Vec<ProgressionIssue>,
}

/// Analyze all arcs in a project for structural completeness.
pub fn analyze_all_arcs(project: &Project) -> Vec<ArcProgression> {
    let total_duration = project.timeline.total_duration_ms;

    project
        .arcs
        .iter()
        .map(|arc| {
            let track = project
                .timeline
                .tracks
                .iter()
                .find(|t| t.arc_id == arc.id);

            let clips = match track {
                Some(t) => &t.clips,
                None => {
                    return ArcProgression {
                        arc_id: arc.id.0,
                        arc_name: arc.name.clone(),
                        beat_count: 0,
                        has_setup: false,
                        has_resolution: false,
                        coverage_percent: 0.0,
                        longest_gap_ms: total_duration,
                        issues: vec![ProgressionIssue {
                            severity: Severity::Error,
                            message: "No track exists for this arc".into(),
                        }],
                    };
                }
            };

            let beat_count = clips.len();
            let has_setup = clips.iter().any(|c| c.beat_type == BeatType::Setup);
            let has_resolution = clips
                .iter()
                .any(|c| c.beat_type == BeatType::Resolution || c.beat_type == BeatType::Payoff);

            // Calculate coverage.
            let covered_ms: u64 = clips
                .iter()
                .map(|c| c.time_range.end_ms.saturating_sub(c.time_range.start_ms))
                .sum();
            let coverage_percent = if total_duration > 0 {
                (covered_ms as f64 / total_duration as f64) * 100.0
            } else {
                0.0
            };

            // Find longest gap.
            let mut sorted_clips = clips.iter().collect::<Vec<_>>();
            sorted_clips.sort_by_key(|c| c.time_range.start_ms);

            let mut longest_gap_ms = 0u64;
            let mut cursor = 0u64;
            for clip in &sorted_clips {
                if clip.time_range.start_ms > cursor {
                    let gap = clip.time_range.start_ms - cursor;
                    longest_gap_ms = longest_gap_ms.max(gap);
                }
                cursor = clip.time_range.end_ms;
            }
            if cursor < total_duration {
                longest_gap_ms = longest_gap_ms.max(total_duration - cursor);
            }

            // Build issues list.
            let mut issues = Vec::new();

            if beat_count < 3 {
                issues.push(ProgressionIssue {
                    severity: Severity::Warning,
                    message: format!(
                        "Only {beat_count} beat(s) — minimum 3 recommended for a complete arc"
                    ),
                });
            }

            if !has_setup {
                issues.push(ProgressionIssue {
                    severity: Severity::Warning,
                    message: "Missing a Setup beat".into(),
                });
            }

            if !has_resolution {
                issues.push(ProgressionIssue {
                    severity: Severity::Warning,
                    message: "Missing a Resolution or Payoff beat".into(),
                });
            }

            if longest_gap_ms > 300_000 {
                // 5 minutes
                issues.push(ProgressionIssue {
                    severity: Severity::Warning,
                    message: format!(
                        "Longest gap is {:.1} minutes — may lose audience engagement",
                        longest_gap_ms as f64 / 60_000.0
                    ),
                });
            }

            if coverage_percent < 5.0 && beat_count > 0 {
                issues.push(ProgressionIssue {
                    severity: Severity::Warning,
                    message: format!("Very low coverage ({coverage_percent:.0}%) of episode runtime"),
                });
            }

            ArcProgression {
                arc_id: arc.id.0,
                arc_name: arc.name.clone(),
                beat_count,
                has_setup,
                has_resolution,
                coverage_percent,
                longest_gap_ms,
                issues,
            }
        })
        .collect()
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::story::arc::{ArcType, Color, StoryArc};
    use crate::timeline::clip::{BeatClip, BeatType};
    use crate::timeline::structure::EpisodeStructure;
    use crate::timeline::timing::TimeRange;
    use crate::timeline::track::ArcTrack;
    use crate::timeline::Timeline;

    fn make_test_project() -> Project {
        let arc = StoryArc::new("Test Arc", ArcType::APlot, Color::A_PLOT);
        let mut track = ArcTrack::new(arc.id);
        track.clips.push(BeatClip::new(
            "Setup",
            BeatType::Setup,
            TimeRange { start_ms: 0, end_ms: 120_000 },
        ));
        track.clips.push(BeatClip::new(
            "Complication",
            BeatType::Complication,
            TimeRange { start_ms: 200_000, end_ms: 400_000 },
        ));
        track.clips.push(BeatClip::new(
            "Resolution",
            BeatType::Resolution,
            TimeRange { start_ms: 900_000, end_ms: 1_100_000 },
        ));

        let mut timeline = Timeline::new(1_320_000, EpisodeStructure::standard_30_min());
        timeline.tracks.push(track);

        let mut project = Project::new("Test", timeline);
        project.arcs.push(arc);
        project
    }

    #[test]
    fn analyze_complete_arc_no_errors() {
        let project = make_test_project();
        let results = analyze_all_arcs(&project);
        assert_eq!(results.len(), 1);
        let r = &results[0];
        assert_eq!(r.beat_count, 3);
        assert!(r.has_setup);
        assert!(r.has_resolution);
        assert!(
            r.issues.iter().all(|i| i.severity != Severity::Error),
            "unexpected errors: {:?}",
            r.issues
        );
    }

    #[test]
    fn analyze_incomplete_arc_flags_issues() {
        let arc = StoryArc::new("Incomplete", ArcType::BPlot, Color::B_PLOT);
        let track = ArcTrack::new(arc.id);

        let mut timeline = Timeline::new(1_320_000, EpisodeStructure::standard_30_min());
        timeline.tracks.push(track);

        let mut project = Project::new("Test", timeline);
        project.arcs.push(arc);

        let results = analyze_all_arcs(&project);
        let r = &results[0];
        assert_eq!(r.beat_count, 0);
        assert!(!r.has_setup);
        assert!(!r.has_resolution);
        assert!(!r.issues.is_empty());
    }
}
