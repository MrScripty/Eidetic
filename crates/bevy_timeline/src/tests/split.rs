use eidetic_core::timeline::node::NodeId;

use crate::{TimelineRendererApp, TimelineRendererCommand, TimelineRendererError};

use super::projection_with_node;

#[test]
fn renderer_app_emits_validated_split_node_command() {
    let node_id = NodeId::new();
    let left_node_id = NodeId::new();
    let right_node_id = NodeId::new();
    let mut renderer = TimelineRendererApp::new();
    renderer.set_projection(projection_with_node(node_id));

    assert_eq!(
        renderer.request_split_node(node_id, 2_500, left_node_id, right_node_id),
        Ok(())
    );
    assert_eq!(
        renderer.drain_commands(),
        vec![TimelineRendererCommand::SplitNode {
            node_id,
            at_ms: 2_500,
            left_node_id,
            right_node_id
        }]
    );
}

#[test]
fn renderer_app_rejects_split_node_command_outside_clip() {
    let node_id = NodeId::new();
    let left_node_id = NodeId::new();
    let right_node_id = NodeId::new();
    let mut renderer = TimelineRendererApp::new();
    renderer.set_projection(projection_with_node(node_id));

    assert_eq!(
        renderer.request_split_node(node_id, 4_000, left_node_id, right_node_id),
        Err(TimelineRendererError::InvalidNodeSplit {
            at_ms: 4_000,
            start_ms: 1_000,
            end_ms: 4_000
        })
    );
    assert!(renderer.drain_commands().is_empty());
}

#[test]
fn renderer_app_rejects_split_node_command_with_existing_output_ids() {
    let node_id = NodeId::new();
    let right_node_id = NodeId::new();
    let mut renderer = TimelineRendererApp::new();
    renderer.set_projection(projection_with_node(node_id));

    assert_eq!(
        renderer.request_split_node(node_id, 2_500, node_id, right_node_id),
        Err(TimelineRendererError::InvalidSplitOutputNodeIds {
            left_node_id: node_id,
            right_node_id,
        })
    );
    assert!(renderer.drain_commands().is_empty());
}
