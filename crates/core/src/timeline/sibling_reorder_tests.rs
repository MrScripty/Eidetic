use super::*;
use crate::timeline::{
    node::{StoryLevel, StoryNode},
    structure::EpisodeStructure,
};
fn fixture() -> (Timeline, NodeId, NodeId, NodeId) {
    let mut t = Timeline::new(1000, EpisodeStructure::standard_30_min());
    let p = StoryNode::new(
        "Parent",
        StoryLevel::Premise,
        TimeRange::new(0, 1000).unwrap(),
    );
    let parent = p.id;
    t.add_node(p).unwrap();
    let mut a = StoryNode::new_child(
        "A",
        StoryLevel::Act,
        TimeRange::new(100, 300).unwrap(),
        parent,
    );
    a.sort_order = 1;
    a.locked = true;
    a.content.notes = "  exact 雨\n".into();
    let first = a.id;
    t.add_node(a).unwrap();
    let mut b = StoryNode::new_child(
        "B",
        StoryLevel::Act,
        TimeRange::new(400, 700).unwrap(),
        parent,
    );
    b.sort_order = 2;
    let second = b.id;
    t.add_node(b).unwrap();
    let c = StoryNode::new_child(
        "Child",
        StoryLevel::Sequence,
        TimeRange::new(150, 250).unwrap(),
        first,
    );
    let child = c.id;
    t.add_node(c).unwrap();
    (t, first, second, child)
}
#[test]
fn swap_translates_children_preserves_content_and_is_reversible() {
    let (mut t, a, b, c) = fixture();
    let before = serde_json::to_value(&t).unwrap();
    t.reorder_adjacent_nodes(a, b).unwrap();
    assert_eq!(
        t.node(a).unwrap().time_range,
        TimeRange::new(500, 700).unwrap()
    );
    assert_eq!(
        t.node(b).unwrap().time_range,
        TimeRange::new(100, 400).unwrap()
    );
    assert_eq!(
        t.node(c).unwrap().time_range,
        TimeRange::new(550, 650).unwrap()
    );
    assert!(t.node(a).unwrap().locked);
    assert_eq!(t.node(a).unwrap().content.notes, "  exact 雨\n");
    assert_eq!(
        t.children_of(t.node(a).unwrap().parent_id.unwrap())[0].id,
        b
    );
    t.reorder_adjacent_nodes(a, b).unwrap();
    assert_eq!(serde_json::to_value(&t).unwrap(), before);
}
#[test]
fn equal_sort_order_uses_chronological_tie_break() {
    let (mut t, a, b, _) = fixture();
    t.node_mut(b).unwrap().sort_order = 1;
    t.reorder_adjacent_nodes(a, b).unwrap();
    assert_eq!(t.siblings_of(a)[0].id, b);
}
#[test]
fn malformed_overlap_child_and_order_refuse_without_mutation() {
    for case in 0..4 {
        let (mut t, a, b, c) = fixture();
        match case {
            0 => t.node_mut(b).unwrap().time_range.start_ms = 200,
            1 => t.node_mut(c).unwrap().time_range.end_ms = 350,
            2 => t.node_mut(b).unwrap().sort_order = 0,
            _ => t.node_mut(a).unwrap().time_range.end_ms = 50,
        };
        let before = serde_json::to_value(&t).unwrap();
        assert!(t.reorder_adjacent_nodes(a, b).is_err());
        assert_eq!(serde_json::to_value(&t).unwrap(), before);
    }
}
#[test]
fn extreme_ranges_are_exact_without_float_rounding() {
    let (a, b) = swapped_adjacent_ranges(
        TimeRange::new(1, u64::MAX / 2).unwrap(),
        TimeRange::new(u64::MAX / 2 + 7, u64::MAX).unwrap(),
    )
    .unwrap();
    assert_eq!(a.duration_ms(), u64::MAX / 2 - 1);
    assert_eq!(b.duration_ms(), u64::MAX - (u64::MAX / 2 + 7));
    assert_eq!(a.start_ms - b.end_ms, 7);
}
