use crate::project::Project;
use crate::story::arc::{ArcType, Color, StoryArc};
use crate::timeline::node::{BeatType, NodeId, StoryLevel, StoryNode};
use crate::timeline::structure::EpisodeStructure;
use crate::timeline::timing::TimeRange;
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

/// Act time boundaries: (start_ms, end_ms) for Cold Open, Act One, Act Two, Act Three, Tag.
const ACT_TIMES: [(u64, u64); 5] = [
    (0, 120_000),
    (150_000, 570_000),
    (570_000, 990_000),
    (990_000, 1_290_000),
    (1_290_000, 1_320_000),
];

/// A scene specification used during template construction.
struct SceneSpec {
    name: String,
    beat_type: BeatType,
    arc_prefix: &'static str,
    start_ms: u64,
    end_ms: u64,
}

impl Template {
    /// Build a new project from this template with pre-placed acts, scenes, and arcs.
    pub fn build_project(self, name: impl Into<String>) -> Project {
        let structure = EpisodeStructure::standard_30_min();
        let mut timeline = Timeline::new(EPISODE_DURATION_MS, structure);

        let a_arc = StoryArc::new("A-Plot", ArcType::APlot, Color::A_PLOT);
        let b_arc = StoryArc::new("B-Plot", ArcType::BPlot, Color::B_PLOT);
        let c_arc = StoryArc::new("C-Runner", ArcType::CRunner, Color::C_RUNNER);

        // Create Premise node spanning the entire timeline.
        let mut premise = StoryNode::new(
            "Episode Premise",
            StoryLevel::Premise,
            TimeRange { start_ms: 0, end_ms: EPISODE_DURATION_MS },
        );
        premise.sort_order = 0;
        let premise_id = premise.id;
        timeline.nodes.push(premise);

        // Create Act-level nodes as children of Premise.
        let acts = self.build_acts(premise_id);
        let act_ids: Vec<NodeId> = acts.iter().map(|a| a.id).collect();
        for act in acts {
            timeline.nodes.push(act);
        }

        // Create interleaved Scene-level nodes and tag them with arcs.
        // Scenes are sorted by start time so they don't overlap on the shared track.
        let scenes = self.build_interleaved_scenes();
        for (i, spec) in scenes.iter().enumerate() {
            let arc = match spec.arc_prefix {
                "A" => &a_arc,
                "B" => &b_arc,
                _ => &c_arc,
            };
            let parent_id = find_act_for_time(&act_ids, spec.start_ms);
            let mut node = StoryNode::new(
                &spec.name,
                StoryLevel::Scene,
                TimeRange { start_ms: spec.start_ms, end_ms: spec.end_ms },
            );
            node.beat_type = Some(spec.beat_type.clone());
            node.parent_id = parent_id;
            node.sort_order = i as u32;
            timeline.tag_node(node.id, arc.id);
            timeline.nodes.push(node);
        }

        let mut project = Project::new(name, timeline);
        project.arcs.push(a_arc);
        project.arcs.push(b_arc);
        project.arcs.push(c_arc);
        project
    }

    fn build_acts(self, premise_id: NodeId) -> Vec<StoryNode> {
        let names = ["Cold Open", "Act One", "Act Two", "Act Three", "Tag"];
        ACT_TIMES
            .iter()
            .zip(names.iter())
            .enumerate()
            .map(|(i, ((start, end), name))| {
                let mut node = StoryNode::new(*name, StoryLevel::Act, TimeRange { start_ms: *start, end_ms: *end });
                node.parent_id = Some(premise_id);
                node.sort_order = i as u32;
                node
            })
            .collect()
    }

    /// Build all scenes across A/B/C arcs, interleaved so no two overlap.
    ///
    /// Scenes are placed sequentially within each act, alternating between arcs
    /// to create the cross-cutting rhythm typical of TV comedy structure.
    fn build_interleaved_scenes(self) -> Vec<SceneSpec> {
        match self {
            // Multi-cam: rapid A/B cross-cutting with brief C-runner inserts.
            //
            // Cold Open  [0 ─────────────── 120k]  gap [120k ── 150k]
            //   A: Setup     [0 ─── 60k]
            //   C: Beat      [60k ─ 120k]
            //
            // Act One    [150k ────────────────────── 570k]
            //   B: Setup     [150k ── 270k]
            //   A: Comp.     [270k ── 450k]
            //   C: Beat      [450k ── 510k]
            //   B: Comp.     [510k ── 570k]
            //
            // Act Two    [570k ────────────────────── 990k]
            //   A: Escal.    [570k ── 750k]
            //   B: Payoff    [750k ── 900k]
            //   A: (cont)    [900k ── 990k]
            //
            // Act Three  [990k ────────────────────── 1290k]
            //   A: Climax    [990k ── 1140k]
            //   C: Callback  [1140k ─ 1200k]
            //   A: Resolut.  [1200k ─ 1290k]
            //
            // Tag         [1290k ── 1320k]
            Self::MultiCam => vec![
                // Cold Open
                SceneSpec { name: "A: Setup".into(),         beat_type: BeatType::Setup,        arc_prefix: "A", start_ms: 0,         end_ms: 60_000 },
                SceneSpec { name: "C: Beat".into(),          beat_type: BeatType::Setup,        arc_prefix: "C", start_ms: 60_000,    end_ms: 120_000 },
                // Act One
                SceneSpec { name: "B: Setup".into(),         beat_type: BeatType::Setup,        arc_prefix: "B", start_ms: 150_000,   end_ms: 270_000 },
                SceneSpec { name: "A: Complication".into(),   beat_type: BeatType::Complication, arc_prefix: "A", start_ms: 270_000,   end_ms: 450_000 },
                SceneSpec { name: "C: Beat".into(),          beat_type: BeatType::Callback,     arc_prefix: "C", start_ms: 450_000,   end_ms: 510_000 },
                SceneSpec { name: "B: Complication".into(),   beat_type: BeatType::Complication, arc_prefix: "B", start_ms: 510_000,   end_ms: 570_000 },
                // Act Two
                SceneSpec { name: "A: Escalation".into(),    beat_type: BeatType::Escalation,   arc_prefix: "A", start_ms: 570_000,   end_ms: 750_000 },
                SceneSpec { name: "B: Payoff".into(),        beat_type: BeatType::Payoff,       arc_prefix: "B", start_ms: 750_000,   end_ms: 900_000 },
                SceneSpec { name: "A: Escalation 2".into(),  beat_type: BeatType::Escalation,   arc_prefix: "A", start_ms: 900_000,   end_ms: 990_000 },
                // Act Three
                SceneSpec { name: "A: Climax".into(),        beat_type: BeatType::Climax,       arc_prefix: "A", start_ms: 990_000,   end_ms: 1_140_000 },
                SceneSpec { name: "C: Callback".into(),      beat_type: BeatType::Payoff,       arc_prefix: "C", start_ms: 1_140_000, end_ms: 1_200_000 },
                SceneSpec { name: "A: Resolution".into(),    beat_type: BeatType::Resolution,   arc_prefix: "A", start_ms: 1_200_000, end_ms: 1_290_000 },
            ],

            // Single-cam: longer flowing scenes with more gradual transitions.
            Self::SingleCam => vec![
                // Cold Open
                SceneSpec { name: "A: Setup".into(),         beat_type: BeatType::Setup,        arc_prefix: "A", start_ms: 0,         end_ms: 120_000 },
                // Act One
                SceneSpec { name: "B: Setup".into(),         beat_type: BeatType::Setup,        arc_prefix: "B", start_ms: 150_000,   end_ms: 300_000 },
                SceneSpec { name: "A: Complication".into(),   beat_type: BeatType::Complication, arc_prefix: "A", start_ms: 300_000,   end_ms: 480_000 },
                SceneSpec { name: "C: Beat".into(),          beat_type: BeatType::Setup,        arc_prefix: "C", start_ms: 480_000,   end_ms: 540_000 },
                SceneSpec { name: "B: Complication".into(),   beat_type: BeatType::Complication, arc_prefix: "B", start_ms: 540_000,   end_ms: 570_000 },
                // Act Two
                SceneSpec { name: "A: Escalation".into(),    beat_type: BeatType::Escalation,   arc_prefix: "A", start_ms: 570_000,   end_ms: 750_000 },
                SceneSpec { name: "B: Payoff".into(),        beat_type: BeatType::Payoff,       arc_prefix: "B", start_ms: 750_000,   end_ms: 880_000 },
                SceneSpec { name: "C: Beat".into(),          beat_type: BeatType::Callback,     arc_prefix: "C", start_ms: 880_000,   end_ms: 940_000 },
                SceneSpec { name: "A: Escalation 2".into(),  beat_type: BeatType::Escalation,   arc_prefix: "A", start_ms: 940_000,   end_ms: 990_000 },
                // Act Three
                SceneSpec { name: "A: Climax".into(),        beat_type: BeatType::Climax,       arc_prefix: "A", start_ms: 990_000,   end_ms: 1_150_000 },
                SceneSpec { name: "C: Callback".into(),      beat_type: BeatType::Payoff,       arc_prefix: "C", start_ms: 1_150_000, end_ms: 1_210_000 },
                SceneSpec { name: "A: Resolution".into(),    beat_type: BeatType::Resolution,   arc_prefix: "A", start_ms: 1_210_000, end_ms: 1_290_000 },
            ],

            // Animated: more C-runner beats, playful cross-cutting.
            Self::Animated => vec![
                // Cold Open
                SceneSpec { name: "A: Setup".into(),         beat_type: BeatType::Setup,        arc_prefix: "A", start_ms: 0,         end_ms: 80_000 },
                SceneSpec { name: "C: Beat".into(),          beat_type: BeatType::Setup,        arc_prefix: "C", start_ms: 80_000,    end_ms: 120_000 },
                // Act One
                SceneSpec { name: "B: Setup".into(),         beat_type: BeatType::Setup,        arc_prefix: "B", start_ms: 150_000,   end_ms: 280_000 },
                SceneSpec { name: "A: Complication".into(),   beat_type: BeatType::Complication, arc_prefix: "A", start_ms: 280_000,   end_ms: 430_000 },
                SceneSpec { name: "C: Beat".into(),          beat_type: BeatType::Callback,     arc_prefix: "C", start_ms: 430_000,   end_ms: 490_000 },
                SceneSpec { name: "B: Complication".into(),   beat_type: BeatType::Complication, arc_prefix: "B", start_ms: 490_000,   end_ms: 570_000 },
                // Act Two
                SceneSpec { name: "A: Escalation".into(),    beat_type: BeatType::Escalation,   arc_prefix: "A", start_ms: 570_000,   end_ms: 720_000 },
                SceneSpec { name: "C: Beat".into(),          beat_type: BeatType::Callback,     arc_prefix: "C", start_ms: 720_000,   end_ms: 780_000 },
                SceneSpec { name: "B: Payoff".into(),        beat_type: BeatType::Payoff,       arc_prefix: "B", start_ms: 780_000,   end_ms: 910_000 },
                SceneSpec { name: "A: Escalation 2".into(),  beat_type: BeatType::Escalation,   arc_prefix: "A", start_ms: 910_000,   end_ms: 990_000 },
                // Act Three
                SceneSpec { name: "A: Climax".into(),        beat_type: BeatType::Climax,       arc_prefix: "A", start_ms: 990_000,   end_ms: 1_130_000 },
                SceneSpec { name: "C: Callback".into(),      beat_type: BeatType::Payoff,       arc_prefix: "C", start_ms: 1_130_000, end_ms: 1_190_000 },
                SceneSpec { name: "A: Resolution".into(),    beat_type: BeatType::Resolution,   arc_prefix: "A", start_ms: 1_190_000, end_ms: 1_290_000 },
            ],
        }
    }
}

/// Find which act a time position falls into.
fn find_act_for_time(act_ids: &[NodeId], start_ms: u64) -> Option<NodeId> {
    for (i, (act_start, act_end)) in ACT_TIMES.iter().enumerate() {
        if start_ms >= *act_start && start_ms < *act_end {
            return act_ids.get(i).copied();
        }
    }
    act_ids.first().copied()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_multicam_template_builds_valid_project() {
        let project = Template::MultiCam.build_project("Test Episode");
        assert_eq!(project.name, "Test Episode");
        assert_eq!(project.arcs.len(), 3);

        // Should have 5 tracks (one per level: Premise, Act, Sequence, Scene, Beat).
        assert_eq!(project.timeline.tracks.len(), 5);

        // Should have 1 premise.
        let premises = project.timeline.nodes_at_level(StoryLevel::Premise);
        assert_eq!(premises.len(), 1);

        // Should have 5 acts.
        let acts = project.timeline.nodes_at_level(StoryLevel::Act);
        assert_eq!(acts.len(), 5);

        // All acts should be children of the premise.
        let premise_id = premises[0].id;
        for act in &acts {
            assert_eq!(act.parent_id, Some(premise_id));
        }

        // MultiCam has 12 scenes.
        let scenes = project.timeline.nodes_at_level(StoryLevel::Scene);
        assert_eq!(scenes.len(), 12);
    }

    #[test]
    fn test_animated_template_c_runner_has_four_scenes() {
        let project = Template::Animated.build_project("Animated Test");
        let c_arc = &project.arcs[2];
        let c_scene_count = project.timeline.nodes_for_arc(c_arc.id).len();
        assert_eq!(c_scene_count, 4);
    }

    #[test]
    fn test_all_nodes_within_timeline_duration() {
        for template in [Template::MultiCam, Template::SingleCam, Template::Animated] {
            let project = template.build_project("Duration Test");
            let max_end = project.timeline.total_duration_ms;
            for node in &project.timeline.nodes {
                assert!(
                    node.time_range.end_ms <= max_end,
                    "node '{}' exceeds timeline: {} > {}",
                    node.name,
                    node.time_range.end_ms,
                    max_end,
                );
            }
        }
    }

    #[test]
    fn test_scene_nodes_have_parent_ids() {
        let project = Template::MultiCam.build_project("Parent Test");
        for node in &project.timeline.nodes {
            if node.level == StoryLevel::Scene {
                assert!(
                    node.parent_id.is_some(),
                    "scene '{}' should have a parent_id",
                    node.name
                );
            }
        }
    }

    #[test]
    fn test_scenes_do_not_overlap() {
        for template in [Template::MultiCam, Template::SingleCam, Template::Animated] {
            let project = template.build_project("Overlap Test");
            let scenes = project.timeline.nodes_at_level(StoryLevel::Scene);
            for i in 1..scenes.len() {
                assert!(
                    scenes[i].time_range.start_ms >= scenes[i - 1].time_range.end_ms,
                    "scene '{}' [{}-{}] overlaps with '{}' [{}-{}] in {:?}",
                    scenes[i].name,
                    scenes[i].time_range.start_ms,
                    scenes[i].time_range.end_ms,
                    scenes[i - 1].name,
                    scenes[i - 1].time_range.start_ms,
                    scenes[i - 1].time_range.end_ms,
                    template,
                );
            }
        }
    }
}
