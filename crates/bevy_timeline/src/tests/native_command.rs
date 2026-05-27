use eidetic_core::timeline::node::NodeId;
use eidetic_core::timeline::relationship::{RelationshipId, RelationshipType};

use crate::{
    TimelineNativeWindowControl, TimelineNativeWindowControlHandle, TimelineRendererCommand,
    TimelineRendererError, TimelineViewport, TimelineViewportGeometry, TimelineViewportPoint,
};

use super::projection_with_node;

#[cfg(feature = "native_render")]
#[test]
fn controlled_native_window_emits_validated_clip_selection_commands() {
    let node_id = NodeId::new();
    let control = TimelineNativeWindowControlHandle::new();
    let window_control = TimelineNativeWindowControl::from(&control);

    let selected_node_id = crate::native_command::emit_timeline_native_clip_selection(
        &window_control,
        &projection_with_node(node_id),
        TimelineViewport::from_duration(10_000),
        TimelineViewportGeometry::new(1_000, 300, 60),
        TimelineViewportPoint::new(250, 10),
    )
    .unwrap();

    assert_eq!(selected_node_id, Some(node_id));
    assert_eq!(
        control.drain_commands(),
        vec![TimelineRendererCommand::SelectNode { node_id }]
    );
    assert!(control.drain_commands().is_empty());
}

#[cfg(feature = "native_render")]
#[test]
fn controlled_native_window_emits_validated_node_range_command() {
    let node_id = NodeId::new();
    let control = TimelineNativeWindowControlHandle::new();
    let window_control = TimelineNativeWindowControl::from(&control);

    crate::native_command::emit_timeline_native_node_range_request(
        &window_control,
        &projection_with_node(node_id),
        node_id,
        2_000,
        5_000,
    )
    .unwrap();

    assert_eq!(
        control.drain_commands(),
        vec![TimelineRendererCommand::SetNodeRange {
            node_id,
            start_ms: 2_000,
            end_ms: 5_000,
        }]
    );
}

#[cfg(feature = "native_render")]
#[test]
fn controlled_native_window_rejects_invalid_node_range_command() {
    let node_id = NodeId::new();
    let control = TimelineNativeWindowControlHandle::new();
    let window_control = TimelineNativeWindowControl::from(&control);

    assert_eq!(
        crate::native_command::emit_timeline_native_node_range_request(
            &window_control,
            &projection_with_node(node_id),
            node_id,
            8_000,
            4_000,
        ),
        Err(TimelineRendererError::InvalidNodeRange {
            start_ms: 8_000,
            end_ms: 4_000,
            duration_ms: 10_000,
        })
    );
    assert!(control.drain_commands().is_empty());
}

#[cfg(feature = "native_render")]
#[test]
fn controlled_native_window_emits_selected_nudge_command_from_projection() {
    let node_id = NodeId::new();
    let control = TimelineNativeWindowControlHandle::new();
    let window_control = TimelineNativeWindowControl::from(&control);
    let mut projection = projection_with_node(node_id);
    projection.selected_node_id = Some(node_id);

    let nudged_range = crate::native_command::emit_timeline_native_selected_nudge_request(
        &window_control,
        &projection,
        500,
    )
    .unwrap();

    assert_eq!(nudged_range, Some((node_id, 1_500, 4_500)));
    assert_eq!(
        control.drain_commands(),
        vec![TimelineRendererCommand::SetNodeRange {
            node_id,
            start_ms: 1_500,
            end_ms: 4_500,
        }]
    );
}

#[cfg(feature = "native_render")]
#[test]
fn controlled_native_window_clamps_selected_nudge_to_projection_bounds() {
    let node_id = NodeId::new();
    let control = TimelineNativeWindowControlHandle::new();
    let window_control = TimelineNativeWindowControl::from(&control);
    let mut projection = projection_with_node(node_id);
    projection.selected_node_id = Some(node_id);

    let nudged_range = crate::native_command::emit_timeline_native_selected_nudge_request(
        &window_control,
        &projection,
        -5_000,
    )
    .unwrap();

    assert_eq!(nudged_range, Some((node_id, 0, 3_000)));
    assert_eq!(
        control.drain_commands(),
        vec![TimelineRendererCommand::SetNodeRange {
            node_id,
            start_ms: 0,
            end_ms: 3_000,
        }]
    );
}

#[cfg(feature = "native_render")]
#[test]
fn controlled_native_window_ignores_selected_nudge_without_projection_selection() {
    let node_id = NodeId::new();
    let control = TimelineNativeWindowControlHandle::new();
    let window_control = TimelineNativeWindowControl::from(&control);

    let nudged_range = crate::native_command::emit_timeline_native_selected_nudge_request(
        &window_control,
        &projection_with_node(node_id),
        500,
    )
    .unwrap();

    assert_eq!(nudged_range, None);
    assert!(control.drain_commands().is_empty());
}

#[cfg(feature = "native_render")]
#[test]
fn controlled_native_window_emits_selected_start_resize_command_from_projection() {
    let node_id = NodeId::new();
    let control = TimelineNativeWindowControlHandle::new();
    let window_control = TimelineNativeWindowControl::from(&control);
    let mut projection = projection_with_node(node_id);
    projection.selected_node_id = Some(node_id);

    let resized_range = crate::native_command::emit_timeline_native_selected_resize_request(
        &window_control,
        &projection,
        crate::native_command::TimelineNativeResizeEdge::Start,
        500,
    )
    .unwrap();

    assert_eq!(resized_range, Some((node_id, 1_500, 4_000)));
    assert_eq!(
        control.drain_commands(),
        vec![TimelineRendererCommand::SetNodeRange {
            node_id,
            start_ms: 1_500,
            end_ms: 4_000,
        }]
    );
}

#[cfg(feature = "native_render")]
#[test]
fn controlled_native_window_emits_selected_end_resize_command_from_projection() {
    let node_id = NodeId::new();
    let control = TimelineNativeWindowControlHandle::new();
    let window_control = TimelineNativeWindowControl::from(&control);
    let mut projection = projection_with_node(node_id);
    projection.selected_node_id = Some(node_id);

    let resized_range = crate::native_command::emit_timeline_native_selected_resize_request(
        &window_control,
        &projection,
        crate::native_command::TimelineNativeResizeEdge::End,
        500,
    )
    .unwrap();

    assert_eq!(resized_range, Some((node_id, 1_000, 4_500)));
    assert_eq!(
        control.drain_commands(),
        vec![TimelineRendererCommand::SetNodeRange {
            node_id,
            start_ms: 1_000,
            end_ms: 4_500,
        }]
    );
}

#[cfg(feature = "native_render")]
#[test]
fn controlled_native_window_clamps_selected_resize_to_projection_bounds() {
    let node_id = NodeId::new();
    let control = TimelineNativeWindowControlHandle::new();
    let window_control = TimelineNativeWindowControl::from(&control);
    let mut projection = projection_with_node(node_id);
    projection.selected_node_id = Some(node_id);

    let resized_range = crate::native_command::emit_timeline_native_selected_resize_request(
        &window_control,
        &projection,
        crate::native_command::TimelineNativeResizeEdge::End,
        10_000,
    )
    .unwrap();

    assert_eq!(resized_range, Some((node_id, 1_000, 10_000)));
}

#[cfg(feature = "native_render")]
#[test]
fn controlled_native_window_ignores_selected_resize_without_projection_selection() {
    let node_id = NodeId::new();
    let control = TimelineNativeWindowControlHandle::new();
    let window_control = TimelineNativeWindowControl::from(&control);

    let resized_range = crate::native_command::emit_timeline_native_selected_resize_request(
        &window_control,
        &projection_with_node(node_id),
        crate::native_command::TimelineNativeResizeEdge::End,
        500,
    )
    .unwrap();

    assert_eq!(resized_range, None);
    assert!(control.drain_commands().is_empty());
}

#[cfg(feature = "native_render")]
#[test]
fn controlled_native_window_emits_validated_delete_node_command() {
    let node_id = NodeId::new();
    let control = TimelineNativeWindowControlHandle::new();
    let window_control = TimelineNativeWindowControl::from(&control);

    crate::native_command::emit_timeline_native_delete_node_request(
        &window_control,
        &projection_with_node(node_id),
        node_id,
    )
    .unwrap();

    assert_eq!(
        control.drain_commands(),
        vec![TimelineRendererCommand::DeleteNode { node_id }]
    );
}

#[cfg(feature = "native_render")]
#[test]
fn controlled_native_window_emits_selected_delete_command_from_projection() {
    let node_id = NodeId::new();
    let control = TimelineNativeWindowControlHandle::new();
    let window_control = TimelineNativeWindowControl::from(&control);
    let mut projection = projection_with_node(node_id);
    projection.selected_node_id = Some(node_id);

    let deleted_node_id = crate::native_command::emit_timeline_native_selected_delete_request(
        &window_control,
        &projection,
    )
    .unwrap();

    assert_eq!(deleted_node_id, Some(node_id));
    assert_eq!(
        control.drain_commands(),
        vec![TimelineRendererCommand::DeleteNode { node_id }]
    );
}

#[cfg(feature = "native_render")]
#[test]
fn controlled_native_window_ignores_selected_delete_without_projection_selection() {
    let node_id = NodeId::new();
    let control = TimelineNativeWindowControlHandle::new();
    let window_control = TimelineNativeWindowControl::from(&control);

    let deleted_node_id = crate::native_command::emit_timeline_native_selected_delete_request(
        &window_control,
        &projection_with_node(node_id),
    )
    .unwrap();

    assert_eq!(deleted_node_id, None);
    assert!(control.drain_commands().is_empty());
}

#[cfg(feature = "native_render")]
#[test]
fn controlled_native_window_rejects_delete_node_command_for_unknown_node() {
    let known_node_id = NodeId::new();
    let unknown_node_id = NodeId::new();
    let control = TimelineNativeWindowControlHandle::new();
    let window_control = TimelineNativeWindowControl::from(&control);

    assert_eq!(
        crate::native_command::emit_timeline_native_delete_node_request(
            &window_control,
            &projection_with_node(known_node_id),
            unknown_node_id,
        ),
        Err(TimelineRendererError::UnknownNode {
            node_id: unknown_node_id,
        })
    );
    assert!(control.drain_commands().is_empty());
}

#[cfg(feature = "native_render")]
#[test]
fn controlled_native_window_emits_validated_split_node_command() {
    let node_id = NodeId::new();
    let left_node_id = NodeId::new();
    let right_node_id = NodeId::new();
    let control = TimelineNativeWindowControlHandle::new();
    let window_control = TimelineNativeWindowControl::from(&control);

    crate::native_command::emit_timeline_native_split_node_request(
        &window_control,
        &projection_with_node(node_id),
        node_id,
        2_500,
        left_node_id,
        right_node_id,
    )
    .unwrap();

    assert_eq!(
        control.drain_commands(),
        vec![TimelineRendererCommand::SplitNode {
            node_id,
            at_ms: 2_500,
            left_node_id,
            right_node_id,
        }]
    );
}

#[cfg(feature = "native_render")]
#[test]
fn controlled_native_window_emits_selected_split_command_from_projection() {
    let node_id = NodeId::new();
    let left_node_id = NodeId::new();
    let right_node_id = NodeId::new();
    let control = TimelineNativeWindowControlHandle::new();
    let window_control = TimelineNativeWindowControl::from(&control);
    let mut projection = projection_with_node(node_id);
    projection.selected_node_id = Some(node_id);

    let split_node_id = crate::native_command::emit_timeline_native_selected_split_request(
        &window_control,
        &projection,
        2_500,
        left_node_id,
        right_node_id,
    )
    .unwrap();

    assert_eq!(split_node_id, Some(node_id));
    assert_eq!(
        control.drain_commands(),
        vec![TimelineRendererCommand::SplitNode {
            node_id,
            at_ms: 2_500,
            left_node_id,
            right_node_id,
        }]
    );
}

#[cfg(feature = "native_render")]
#[test]
fn controlled_native_window_ignores_selected_split_without_projection_selection() {
    let node_id = NodeId::new();
    let control = TimelineNativeWindowControlHandle::new();
    let window_control = TimelineNativeWindowControl::from(&control);

    let split_node_id = crate::native_command::emit_timeline_native_selected_split_request(
        &window_control,
        &projection_with_node(node_id),
        2_500,
        NodeId::new(),
        NodeId::new(),
    )
    .unwrap();

    assert_eq!(split_node_id, None);
    assert!(control.drain_commands().is_empty());
}

#[cfg(feature = "native_render")]
#[test]
fn controlled_native_window_rejects_invalid_split_node_command() {
    let node_id = NodeId::new();
    let left_node_id = NodeId::new();
    let right_node_id = NodeId::new();
    let control = TimelineNativeWindowControlHandle::new();
    let window_control = TimelineNativeWindowControl::from(&control);

    assert_eq!(
        crate::native_command::emit_timeline_native_split_node_request(
            &window_control,
            &projection_with_node(node_id),
            node_id,
            10_000,
            left_node_id,
            right_node_id,
        ),
        Err(TimelineRendererError::InvalidNodeSplit {
            at_ms: 10_000,
            start_ms: 1_000,
            end_ms: 4_000,
        })
    );
    assert!(control.drain_commands().is_empty());
}

#[cfg(feature = "native_render")]
#[test]
fn controlled_native_window_emits_validated_create_child_from_parent_command() {
    let parent_id = NodeId::new();
    let node_id = NodeId::new();
    let control = TimelineNativeWindowControlHandle::new();
    let window_control = TimelineNativeWindowControl::from(&control);

    crate::native_command::emit_timeline_native_create_child_from_parent_request(
        &window_control,
        &projection_with_node(parent_id),
        node_id,
        parent_id,
    )
    .unwrap();

    assert_eq!(
        control.drain_commands(),
        vec![TimelineRendererCommand::CreateChildFromParent { node_id, parent_id }]
    );
}

#[cfg(feature = "native_render")]
#[test]
fn controlled_native_window_emits_selected_create_child_from_projection() {
    let parent_id = NodeId::new();
    let node_id = NodeId::new();
    let control = TimelineNativeWindowControlHandle::new();
    let window_control = TimelineNativeWindowControl::from(&control);
    let mut projection = projection_with_node(parent_id);
    projection.selected_node_id = Some(parent_id);

    let created_parent_id =
        crate::native_command::emit_timeline_native_selected_create_child_from_parent_request(
            &window_control,
            &projection,
            node_id,
        )
        .unwrap();

    assert_eq!(created_parent_id, Some(parent_id));
    assert_eq!(
        control.drain_commands(),
        vec![TimelineRendererCommand::CreateChildFromParent { node_id, parent_id }]
    );
}

#[cfg(feature = "native_render")]
#[test]
fn controlled_native_window_ignores_selected_create_child_without_projection_selection() {
    let node_id = NodeId::new();
    let control = TimelineNativeWindowControlHandle::new();
    let window_control = TimelineNativeWindowControl::from(&control);

    let created_parent_id =
        crate::native_command::emit_timeline_native_selected_create_child_from_parent_request(
            &window_control,
            &projection_with_node(NodeId::new()),
            node_id,
        )
        .unwrap();

    assert_eq!(created_parent_id, None);
    assert!(control.drain_commands().is_empty());
}

#[cfg(feature = "native_render")]
#[test]
fn controlled_native_window_rejects_create_child_intent_with_unknown_parent() {
    let known_node_id = NodeId::new();
    let parent_id = NodeId::new();
    let node_id = NodeId::new();
    let control = TimelineNativeWindowControlHandle::new();
    let window_control = TimelineNativeWindowControl::from(&control);

    assert_eq!(
        crate::native_command::emit_timeline_native_create_child_from_parent_request(
            &window_control,
            &projection_with_node(known_node_id),
            node_id,
            parent_id,
        ),
        Err(TimelineRendererError::UnknownNode { node_id: parent_id })
    );
    assert!(control.drain_commands().is_empty());
}

#[cfg(feature = "native_render")]
#[test]
fn controlled_native_window_emits_validated_create_relationship_command() {
    let from_node_id = NodeId::new();
    let to_node_id = NodeId::new();
    let relationship_id = RelationshipId::new();
    let control = TimelineNativeWindowControlHandle::new();
    let window_control = TimelineNativeWindowControl::from(&control);
    let mut projection = projection_with_node(from_node_id);
    let mut to_clip = projection.clips[0].clone();
    to_clip.node_id = to_node_id;
    projection.clips.push(to_clip);

    crate::native_command::emit_timeline_native_create_relationship_request(
        &window_control,
        &projection,
        relationship_id,
        from_node_id,
        to_node_id,
        RelationshipType::Thematic,
    )
    .unwrap();

    assert_eq!(
        control.drain_commands(),
        vec![TimelineRendererCommand::CreateRelationship {
            relationship_id,
            from_node_id,
            to_node_id,
            relationship_type: RelationshipType::Thematic,
        }]
    );
}

#[cfg(feature = "native_render")]
#[test]
fn controlled_native_window_rejects_create_relationship_with_unknown_endpoint() {
    let from_node_id = NodeId::new();
    let to_node_id = NodeId::new();
    let relationship_id = RelationshipId::new();
    let control = TimelineNativeWindowControlHandle::new();
    let window_control = TimelineNativeWindowControl::from(&control);

    assert_eq!(
        crate::native_command::emit_timeline_native_create_relationship_request(
            &window_control,
            &projection_with_node(from_node_id),
            relationship_id,
            from_node_id,
            to_node_id,
            RelationshipType::Thematic,
        ),
        Err(TimelineRendererError::UnknownNode {
            node_id: to_node_id,
        })
    );
    assert!(control.drain_commands().is_empty());
}

#[cfg(feature = "native_render")]
#[test]
fn controlled_native_window_rejects_create_relationship_with_same_endpoint() {
    let node_id = NodeId::new();
    let relationship_id = RelationshipId::new();
    let control = TimelineNativeWindowControlHandle::new();
    let window_control = TimelineNativeWindowControl::from(&control);

    assert_eq!(
        crate::native_command::emit_timeline_native_create_relationship_request(
            &window_control,
            &projection_with_node(node_id),
            relationship_id,
            node_id,
            node_id,
            RelationshipType::Thematic,
        ),
        Err(TimelineRendererError::InvalidRelationshipEndpoints {
            from_node_id: node_id,
            to_node_id: node_id,
        })
    );
    assert!(control.drain_commands().is_empty());
}

#[cfg(feature = "native_render")]
#[test]
fn controlled_native_window_emits_validated_playhead_command() {
    let node_id = NodeId::new();
    let control = TimelineNativeWindowControlHandle::new();
    let window_control = TimelineNativeWindowControl::from(&control);

    crate::native_command::emit_timeline_native_playhead_request(
        &window_control,
        &projection_with_node(node_id),
        4_250,
    )
    .unwrap();

    assert_eq!(
        control.drain_commands(),
        vec![TimelineRendererCommand::SetPlayhead { position_ms: 4_250 }]
    );
}

#[cfg(feature = "native_render")]
#[test]
fn controlled_native_window_rejects_invalid_playhead_command() {
    let node_id = NodeId::new();
    let control = TimelineNativeWindowControlHandle::new();
    let window_control = TimelineNativeWindowControl::from(&control);

    assert_eq!(
        crate::native_command::emit_timeline_native_playhead_request(
            &window_control,
            &projection_with_node(node_id),
            12_000,
        ),
        Err(TimelineRendererError::InvalidPlayheadPosition {
            position_ms: 12_000,
            duration_ms: 10_000,
        })
    );
    assert!(control.drain_commands().is_empty());
}
