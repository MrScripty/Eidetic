use super::*;

fn scene(start: u64, end: u64) -> StoryNode {
    StoryNode::new(
        "Scene",
        StoryLevel::Scene,
        TimeRange::new(start, end).unwrap(),
    )
}

fn boundaries(
    timeline: &Timeline,
    minimum: u64,
) -> Vec<(u64, u64, Option<NodeId>, Option<NodeId>)> {
    timeline
        .find_gaps(StoryLevel::Scene, minimum)
        .into_iter()
        .map(|gap| {
            (
                gap.time_range.start_ms,
                gap.time_range.end_ms,
                gap.preceding_node_id,
                gap.following_node_id,
            )
        })
        .collect()
}

#[test]
fn nested_overlapping_and_touching_clips_only_leave_unoccupied_gaps() {
    let mut timeline = Timeline::new(100, EpisodeStructure::standard_30_min());
    let outer = scene(10, 50);
    let outer_id = outer.id;
    let overlap = scene(40, 60);
    let touching = scene(60, 70);
    let touching_id = touching.id;
    let last = scene(80, 90);
    let last_id = last.id;
    timeline.nodes = vec![outer, scene(20, 30), overlap, touching, scene(65, 68), last];

    let expected = vec![
        (0, 10, None, Some(outer_id)),
        (70, 80, Some(touching_id), Some(last_id)),
        (90, 100, Some(last_id), None),
    ];
    assert_eq!(boundaries(&timeline, 10), expected);
    assert!(boundaries(&timeline, 11).is_empty());
    timeline.nodes.reverse();
    assert_eq!(boundaries(&timeline, 10), expected);
}

#[test]
fn nested_final_clip_does_not_expose_occupied_tail() {
    let mut timeline = Timeline::new(100, EpisodeStructure::standard_30_min());
    let outer = scene(0, 90);
    let outer_id = outer.id;
    timeline.nodes = vec![outer, scene(20, 30)];
    assert_eq!(
        boundaries(&timeline, 0),
        vec![(90, 100, Some(outer_id), None)]
    );
    // Other hierarchy levels do not fill this track's gaps.
    timeline.nodes[0].level = StoryLevel::Act;
    timeline.nodes[1].level = StoryLevel::Beat;
    assert_eq!(boundaries(&timeline, 100), vec![(0, 100, None, None)]);
    assert!(boundaries(&timeline, 101).is_empty());
}

#[test]
fn equal_boundary_nodes_have_stable_gap_neighbors() {
    let mut timeline = Timeline::new(100, EpisodeStructure::standard_30_min());
    let first = scene(10, 90);
    let second = scene(10, 90);
    timeline.nodes = vec![first, second];
    let expected = boundaries(&timeline, 0);
    timeline.nodes.reverse();
    assert_eq!(boundaries(&timeline, 0), expected);
}
