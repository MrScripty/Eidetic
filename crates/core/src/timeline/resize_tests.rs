use super::*;

fn fixture() -> (Timeline, NodeId, NodeId, NodeId) {
    let mut timeline = Timeline::new(u64::MAX, EpisodeStructure::standard_30_min());
    let parent = StoryNode::new(
        "Parent",
        StoryLevel::Premise,
        TimeRange::new(0, 1_000).unwrap(),
    );
    let parent_id = parent.id;
    timeline.add_node(parent).unwrap();
    let mut first = StoryNode::new("First", StoryLevel::Act, TimeRange::new(100, 200).unwrap());
    first.parent_id = Some(parent_id);
    let first_id = first.id;
    timeline.add_node(first).unwrap();
    let mut second = StoryNode::new("Second", StoryLevel::Act, TimeRange::new(300, 301).unwrap());
    second.parent_id = Some(parent_id);
    let second_id = second.id;
    timeline.add_node(second).unwrap();
    (timeline, parent_id, first_id, second_id)
}

#[test]
fn contraction_rejects_collapsed_descendant_without_partial_mutation() {
    let (mut timeline, parent, _, _) = fixture();
    let before = serde_json::to_value(&timeline).unwrap();
    let error = timeline
        .resize_node(parent, TimeRange::new(0, 10).unwrap())
        .unwrap_err();
    assert!(matches!(error, Error::InvalidTimeRange { .. }));
    assert_eq!(serde_json::to_value(&timeline).unwrap(), before);
}

#[test]
fn large_valid_resize_preserves_full_width_descendant_without_overflow() {
    let (mut timeline, parent, first, second) = fixture();
    timeline.node_mut(first).unwrap().time_range = TimeRange::new(0, 1_000).unwrap();
    timeline.node_mut(second).unwrap().time_range = TimeRange::new(500, 1_000).unwrap();
    let new_range = TimeRange::new(1, u64::MAX).unwrap();
    timeline.resize_node(parent, new_range).unwrap();
    assert_eq!(timeline.node(first).unwrap().time_range, new_range);
    assert_eq!(
        timeline.node(second).unwrap().time_range,
        TimeRange::new(u64::MAX / 2 + 1, u64::MAX).unwrap()
    );
    assert!(
        timeline
            .nodes
            .iter()
            .all(|node| node.time_range.validate().is_ok())
    );
}

#[test]
fn no_op_large_range_does_not_round_descendant_milliseconds() {
    let (mut timeline, parent, first, second) = fixture();
    let range = TimeRange::new(0, u64::MAX - 1).unwrap();
    timeline.node_mut(parent).unwrap().time_range = range;
    timeline.node_mut(first).unwrap().time_range = TimeRange::new(1, u64::MAX - 2).unwrap();
    timeline.node_mut(second).unwrap().time_range = TimeRange::new(2, u64::MAX - 3).unwrap();
    let before = serde_json::to_value(&timeline).unwrap();
    timeline.resize_node(parent, range).unwrap();
    assert_eq!(serde_json::to_value(&timeline).unwrap(), before);
}

#[test]
fn malformed_source_range_returns_error_without_panic_or_mutation() {
    for malformed_target in [true, false] {
        let (mut timeline, parent, first, _) = fixture();
        let malformed_id = if malformed_target { parent } else { first };
        timeline.node_mut(malformed_id).unwrap().time_range = TimeRange {
            start_ms: 200,
            end_ms: 100,
        };
        let before = serde_json::to_value(&timeline).unwrap();
        let error = timeline
            .resize_node(parent, TimeRange::new(0, 2_000).unwrap())
            .unwrap_err();
        assert!(matches!(error, Error::InvalidTimeRange { .. }));
        assert_eq!(serde_json::to_value(&timeline).unwrap(), before);
    }
}

#[test]
fn invalid_target_and_timeline_overrun_leave_all_ranges_unchanged() {
    let (mut timeline, parent, _, _) = fixture();
    timeline.total_duration_ms = 1_000;
    let before = serde_json::to_value(&timeline).unwrap();
    for range in [
        TimeRange {
            start_ms: 10,
            end_ms: 10,
        },
        TimeRange {
            start_ms: 50,
            end_ms: 10,
        },
        TimeRange {
            start_ms: 0,
            end_ms: 1_001,
        },
    ] {
        assert!(timeline.resize_node(parent, range).is_err());
        assert_eq!(serde_json::to_value(&timeline).unwrap(), before);
    }
}

#[test]
fn valid_resize_adjusts_multiple_levels_and_preserves_unrelated_nodes() {
    let (mut timeline, parent, first, second) = fixture();
    let mut sequence = StoryNode::new(
        "Sequence",
        StoryLevel::Sequence,
        TimeRange::new(120, 130).unwrap(),
    );
    sequence.parent_id = Some(first);
    let sequence_id = sequence.id;
    timeline.add_node(sequence).unwrap();
    let parent_before = timeline.node(parent).unwrap().time_range;
    let second_before = timeline.node(second).unwrap().time_range;
    timeline
        .resize_node(first, TimeRange::new(200, 400).unwrap())
        .unwrap();
    assert_eq!(
        timeline.node(sequence_id).unwrap().time_range,
        TimeRange::new(240, 260).unwrap()
    );
    assert_eq!(timeline.node(parent).unwrap().time_range, parent_before);
    assert_eq!(timeline.node(second).unwrap().time_range, second_before);
    // A one-millisecond positive range remains admissible.
    timeline
        .resize_node(second, TimeRange::new(400, 401).unwrap())
        .unwrap();
    assert_eq!(
        timeline.node(second).unwrap().time_range,
        TimeRange::new(400, 401).unwrap()
    );
}
