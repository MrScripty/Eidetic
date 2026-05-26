use eidetic_core::timeline::node::{NodeId, StoryLevel};

use crate::{TimelineRendererApp, TimelineRendererCommand, TimelineRendererError};

use super::projection_with_node;

#[test]
fn renderer_app_emits_validated_node_range_command() {
    let node_id = NodeId::new();
    let mut renderer = TimelineRendererApp::new();
    renderer.set_projection(projection_with_node(node_id));

    assert_eq!(renderer.request_node_range(node_id, 2_000, 5_000), Ok(()));
    assert_eq!(
        renderer.drain_commands(),
        vec![TimelineRendererCommand::SetNodeRange {
            node_id,
            start_ms: 2_000,
            end_ms: 5_000
        }]
    );
}

#[test]
fn renderer_app_rejects_invalid_node_range_command() {
    let node_id = NodeId::new();
    let mut renderer = TimelineRendererApp::new();
    renderer.set_projection(projection_with_node(node_id));

    assert_eq!(
        renderer.request_node_range(node_id, 5_000, 2_000),
        Err(TimelineRendererError::InvalidNodeRange {
            start_ms: 5_000,
            end_ms: 2_000,
            duration_ms: 10_000
        })
    );
    assert!(renderer.drain_commands().is_empty());
}

#[test]
fn renderer_app_emits_validated_create_node_command() {
    let parent_id = NodeId::new();
    let node_id = NodeId::new();
    let mut renderer = TimelineRendererApp::new();
    renderer.set_projection(projection_with_node(parent_id));

    assert_eq!(
        renderer.request_create_node(
            node_id,
            Some(parent_id),
            StoryLevel::Act,
            "Inserted act".to_string(),
            2_000,
            5_000,
            None,
        ),
        Ok(())
    );
    assert_eq!(
        renderer.drain_commands(),
        vec![TimelineRendererCommand::CreateNode {
            node_id,
            parent_id: Some(parent_id),
            level: StoryLevel::Act,
            name: "Inserted act".to_string(),
            start_ms: 2_000,
            end_ms: 5_000,
            beat_type: None
        }]
    );
}

#[test]
fn renderer_app_rejects_create_node_command_with_unknown_parent() {
    let known_node_id = NodeId::new();
    let parent_id = NodeId::new();
    let node_id = NodeId::new();
    let mut renderer = TimelineRendererApp::new();
    renderer.set_projection(projection_with_node(known_node_id));

    assert_eq!(
        renderer.request_create_node(
            node_id,
            Some(parent_id),
            StoryLevel::Act,
            "Inserted act".to_string(),
            2_000,
            5_000,
            None,
        ),
        Err(TimelineRendererError::UnknownNode { node_id: parent_id })
    );
    assert!(renderer.drain_commands().is_empty());
}

#[test]
fn renderer_app_emits_validated_delete_node_command() {
    let node_id = NodeId::new();
    let mut renderer = TimelineRendererApp::new();
    renderer.set_projection(projection_with_node(node_id));

    assert_eq!(renderer.request_delete_node(node_id), Ok(()));
    assert_eq!(
        renderer.drain_commands(),
        vec![TimelineRendererCommand::DeleteNode { node_id }]
    );
}

#[test]
fn renderer_app_rejects_delete_node_command_for_unknown_node() {
    let known_node_id = NodeId::new();
    let unknown_node_id = NodeId::new();
    let mut renderer = TimelineRendererApp::new();
    renderer.set_projection(projection_with_node(known_node_id));

    assert_eq!(
        renderer.request_delete_node(unknown_node_id),
        Err(TimelineRendererError::UnknownNode {
            node_id: unknown_node_id
        })
    );
    assert!(renderer.drain_commands().is_empty());
}
