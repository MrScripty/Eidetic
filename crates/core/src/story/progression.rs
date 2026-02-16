use serde::{Deserialize, Serialize};
use uuid::Uuid;

use crate::project::Project;
use crate::timeline::node::{BeatType, StoryLevel};

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
    pub node_count: usize,
    pub has_setup: bool,
    pub has_resolution: bool,
    pub coverage_percent: f64,
    pub longest_gap_ms: u64,
    pub issues: Vec<ProgressionIssue>,
}

/// Analyze all arcs in a project for structural completeness.
///
/// For each arc, gathers all nodes tagged with that arc and analyzes
/// their progression through the timeline.
pub fn analyze_all_arcs(project: &Project) -> Vec<ArcProgression> {
    let total_duration = project.timeline.total_duration_ms;

    project
        .arcs
        .iter()
        .map(|arc| {
            // Gather all nodes tagged with this arc.
            let node_ids = project.timeline.nodes_for_arc(arc.id);

            // Get the actual nodes, filtered to Scene and Beat levels.
            let mut nodes: Vec<_> = node_ids
                .iter()
                .filter_map(|id| project.timeline.node(*id).ok())
                .filter(|n| n.level == StoryLevel::Scene || n.level == StoryLevel::Beat)
                .collect();

            nodes.sort_by_key(|n| n.time_range.start_ms);

            let node_count = nodes.len();
            let has_setup = nodes.iter().any(|n| {
                n.beat_type.as_ref() == Some(&BeatType::Setup)
            });
            let has_resolution = nodes.iter().any(|n| {
                matches!(
                    n.beat_type.as_ref(),
                    Some(BeatType::Resolution) | Some(BeatType::Payoff)
                )
            });

            // Calculate coverage.
            let covered_ms: u64 = nodes
                .iter()
                .map(|n| n.time_range.end_ms.saturating_sub(n.time_range.start_ms))
                .sum();
            let coverage_percent = if total_duration > 0 {
                (covered_ms as f64 / total_duration as f64) * 100.0
            } else {
                0.0
            };

            // Find longest gap.
            let mut longest_gap_ms = 0u64;
            let mut cursor = 0u64;
            for node in &nodes {
                if node.time_range.start_ms > cursor {
                    let gap = node.time_range.start_ms - cursor;
                    longest_gap_ms = longest_gap_ms.max(gap);
                }
                cursor = node.time_range.end_ms;
            }
            if cursor < total_duration {
                longest_gap_ms = longest_gap_ms.max(total_duration - cursor);
            }

            // Build issues list.
            let mut issues = Vec::new();

            if node_count == 0 {
                issues.push(ProgressionIssue {
                    severity: Severity::Warning,
                    message: "No nodes tagged with this arc".into(),
                });
            } else if node_count < 3 {
                issues.push(ProgressionIssue {
                    severity: Severity::Warning,
                    message: format!(
                        "Only {node_count} node(s) — minimum 3 recommended for a complete arc"
                    ),
                });
            }

            if !has_setup && node_count > 0 {
                issues.push(ProgressionIssue {
                    severity: Severity::Warning,
                    message: "Missing a Setup beat".into(),
                });
            }

            if !has_resolution && node_count > 0 {
                issues.push(ProgressionIssue {
                    severity: Severity::Warning,
                    message: "Missing a Resolution or Payoff beat".into(),
                });
            }

            if longest_gap_ms > 300_000 && node_count > 0 {
                issues.push(ProgressionIssue {
                    severity: Severity::Warning,
                    message: format!(
                        "Longest gap is {:.1} minutes — may lose audience engagement",
                        longest_gap_ms as f64 / 60_000.0
                    ),
                });
            }

            if coverage_percent < 5.0 && node_count > 0 {
                issues.push(ProgressionIssue {
                    severity: Severity::Warning,
                    message: format!("Very low coverage ({coverage_percent:.0}%) of episode runtime"),
                });
            }

            ArcProgression {
                arc_id: arc.id.0,
                arc_name: arc.name.clone(),
                node_count,
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
    use crate::timeline::node::{BeatType, StoryLevel, StoryNode};
    use crate::timeline::structure::EpisodeStructure;
    use crate::timeline::timing::TimeRange;
    use crate::timeline::Timeline;

    fn make_test_project() -> Project {
        let arc = StoryArc::new("Test Arc", ArcType::APlot, Color::A_PLOT);
        let mut timeline = Timeline::new(1_320_000, EpisodeStructure::standard_30_min());

        let mut scene1 = StoryNode::new("Setup", StoryLevel::Scene, TimeRange { start_ms: 0, end_ms: 120_000 });
        scene1.beat_type = Some(BeatType::Setup);
        let mut scene2 = StoryNode::new("Complication", StoryLevel::Scene, TimeRange { start_ms: 200_000, end_ms: 400_000 });
        scene2.beat_type = Some(BeatType::Complication);
        let mut scene3 = StoryNode::new("Resolution", StoryLevel::Scene, TimeRange { start_ms: 900_000, end_ms: 1_100_000 });
        scene3.beat_type = Some(BeatType::Resolution);

        // Need to add as Act-level for hierarchy, but for this test we'll add scenes directly.
        // In the new model, Scene nodes need parent Acts. For the test, we create a parent act first.
        let act = StoryNode::new("Act 1", StoryLevel::Act, TimeRange { start_ms: 0, end_ms: 1_320_000 });
        let act_id = act.id;

        timeline.nodes.push(act);

        scene1.parent_id = Some(act_id);
        scene2.parent_id = Some(act_id);
        scene3.parent_id = Some(act_id);

        // Wait - scenes need Sequence parents in new hierarchy...
        // Let's just bypass validation and push directly for the test.
        let scene1_id = scene1.id;
        let scene2_id = scene2.id;
        let scene3_id = scene3.id;

        // Create a sequence to hold the scenes.
        let seq = StoryNode::new_child("Seq 1", StoryLevel::Sequence, TimeRange { start_ms: 0, end_ms: 1_320_000 }, act_id);
        let seq_id = seq.id;
        timeline.nodes.push(seq);

        scene1.parent_id = Some(seq_id);
        scene2.parent_id = Some(seq_id);
        scene3.parent_id = Some(seq_id);

        timeline.nodes.push(scene1);
        timeline.nodes.push(scene2);
        timeline.nodes.push(scene3);

        // Tag scenes with the arc.
        timeline.tag_node(scene1_id, arc.id);
        timeline.tag_node(scene2_id, arc.id);
        timeline.tag_node(scene3_id, arc.id);

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
        assert_eq!(r.node_count, 3);
        assert!(r.has_setup);
        assert!(r.has_resolution);
        assert!(
            r.issues.iter().all(|i| i.severity != Severity::Error),
            "unexpected errors: {:?}",
            r.issues
        );
    }

    #[test]
    fn analyze_empty_arc_flags_issues() {
        let arc = StoryArc::new("Empty", ArcType::BPlot, Color::B_PLOT);
        let timeline = Timeline::new(1_320_000, EpisodeStructure::standard_30_min());

        let mut project = Project::new("Test", timeline);
        project.arcs.push(arc);

        let results = analyze_all_arcs(&project);
        let r = &results[0];
        assert_eq!(r.node_count, 0);
        assert!(!r.has_setup);
        assert!(!r.has_resolution);
        assert!(!r.issues.is_empty());
    }
}
