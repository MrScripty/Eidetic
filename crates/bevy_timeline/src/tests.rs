use super::*;
use eidetic_core::contracts::{TimelineRenderClip, TimelineRenderTrack};
use eidetic_core::timeline::node::{ContentStatus, StoryLevel};
use eidetic_core::timeline::track::TrackId;

#[test]
fn renderer_app_receives_projection_and_emits_validated_selection_command() {
    let node_id = NodeId::new();
    let mut renderer = TimelineRendererApp::new();

    renderer.set_projection(projection_with_node(node_id));

    assert_eq!(renderer.projection_clip_count(), 1);
    assert_eq!(renderer.select_node(node_id), Ok(()));
    assert_eq!(
        renderer.drain_commands(),
        vec![TimelineRendererCommand::SelectNode { node_id }]
    );
    assert!(renderer.drain_commands().is_empty());
}

#[test]
fn renderer_app_rebuilds_scene_entities_from_projection() {
    let node_id = NodeId::new();
    let mut renderer = TimelineRendererApp::new();

    renderer.set_projection(projection_with_node(node_id));

    assert_eq!(renderer.scene_counts(), (1, 1));
    assert_eq!(renderer.scene_relationship_count(), 0);

    renderer.set_projection(TimelineRenderProjection {
        total_duration_ms: 10_000,
        tracks: Vec::new(),
        clips: Vec::new(),
        relationships: Vec::new(),
    });

    assert_eq!(renderer.scene_counts(), (0, 0));
    assert_eq!(renderer.scene_relationship_count(), 0);
}

#[test]
fn renderer_app_rebuilds_relationship_entities_from_projection() {
    let node_id = NodeId::new();
    let track_id = TrackId::new();
    let mut projection = projection_with_clip(node_id, track_id, 1_000, 4_000);
    projection.relationships = vec![eidetic_core::contracts::TimelineRenderRelationship {
        relationship_id: eidetic_core::timeline::relationship::RelationshipId::new(),
        from_node_id: node_id,
        to_node_id: node_id,
        relationship_type: eidetic_core::timeline::relationship::RelationshipType::Thematic,
    }];
    let mut renderer = TimelineRendererApp::new();

    renderer.set_projection(projection);

    assert_eq!(renderer.scene_counts(), (1, 1));
    assert_eq!(renderer.scene_relationship_count(), 1);
}

#[test]
fn renderer_app_derives_relationship_curves_from_projection() {
    let node_id = NodeId::new();
    let track_id = TrackId::new();
    let relationship_id = eidetic_core::timeline::relationship::RelationshipId::new();
    let mut projection = projection_with_clip(node_id, track_id, 1_000, 5_000);
    projection.relationships = vec![eidetic_core::contracts::TimelineRenderRelationship {
        relationship_id,
        from_node_id: node_id,
        to_node_id: node_id,
        relationship_type: eidetic_core::timeline::relationship::RelationshipType::Thematic,
    }];
    let mut renderer = TimelineRendererApp::new();

    renderer.set_projection(projection);
    let curves = renderer.relationship_curves().expect("relationship curves");

    assert_eq!(curves.len(), 1);
    assert_eq!(curves[0].relationship_id, relationship_id);
    assert_eq!(curves[0].start.x_ms, 3_000.0);
    assert_eq!(curves[0].end.x_ms, 3_000.0);
}

#[test]
fn relationship_curves_serialize_for_wasm_bridge() {
    let node_id = NodeId::new();
    let track_id = TrackId::new();
    let relationship_id = eidetic_core::timeline::relationship::RelationshipId::new();
    let mut projection = projection_with_clip(node_id, track_id, 1_000, 5_000);
    projection.relationships = vec![eidetic_core::contracts::TimelineRenderRelationship {
        relationship_id,
        from_node_id: node_id,
        to_node_id: node_id,
        relationship_type: eidetic_core::timeline::relationship::RelationshipType::Thematic,
    }];

    let serialized =
        serde_json::to_value(relationship_curves(&projection).expect("relationship curves"))
            .expect("relationship curves serialize");

    assert_eq!(
        serialized[0]["relationship_id"],
        relationship_id.0.to_string()
    );
    assert_eq!(serialized[0]["from_node_id"], node_id.0.to_string());
    assert_eq!(serialized[0]["to_node_id"], node_id.0.to_string());
    assert_eq!(serialized[0]["relationship_type"], "Thematic");
    assert_eq!(serialized[0]["start"]["x_ms"], 3_000.0);
    assert_eq!(serialized[0]["end"]["y_track"], 0.0);
}

#[test]
fn renderer_app_rejects_relationship_curve_with_unknown_endpoint() {
    let node_id = NodeId::new();
    let missing_node_id = NodeId::new();
    let track_id = TrackId::new();
    let mut projection = projection_with_clip(node_id, track_id, 1_000, 5_000);
    projection.relationships = vec![eidetic_core::contracts::TimelineRenderRelationship {
        relationship_id: eidetic_core::timeline::relationship::RelationshipId::new(),
        from_node_id: node_id,
        to_node_id: missing_node_id,
        relationship_type: eidetic_core::timeline::relationship::RelationshipType::Thematic,
    }];
    let mut renderer = TimelineRendererApp::new();

    renderer.set_projection(projection);

    assert_eq!(
        renderer.relationship_curves(),
        Err(TimelineRendererError::UnknownRelationshipEndpoint {
            node_id: missing_node_id
        })
    );
}

#[test]
fn renderer_app_rejects_selection_before_projection_load() {
    let mut renderer = TimelineRendererApp::new();
    let node_id = NodeId::new();

    assert_eq!(
        renderer.select_node(node_id),
        Err(TimelineRendererError::MissingProjection)
    );
    assert!(renderer.drain_commands().is_empty());
}

#[test]
fn renderer_app_rejects_unknown_node_selection() {
    let mut renderer = TimelineRendererApp::new();
    let known_node_id = NodeId::new();
    let unknown_node_id = NodeId::new();
    renderer.set_projection(projection_with_node(known_node_id));

    assert_eq!(
        renderer.select_node(unknown_node_id),
        Err(TimelineRendererError::UnknownNode {
            node_id: unknown_node_id
        })
    );
    assert!(renderer.drain_commands().is_empty());
}

#[test]
fn renderer_app_hit_tests_and_selects_clip_by_track_and_time() {
    let node_id = NodeId::new();
    let track_id = TrackId::new();
    let mut renderer = TimelineRendererApp::new();
    renderer.set_projection(projection_with_clip(node_id, track_id, 1_000, 4_000));

    assert_eq!(
        renderer.hit_test_clip_at_time(track_id, 2_000),
        Ok(Some(node_id))
    );
    assert_eq!(renderer.select_clip_at_time(track_id, 2_000), Ok(()));
    assert_eq!(
        renderer.drain_commands(),
        vec![TimelineRendererCommand::SelectNode { node_id }]
    );
}

#[test]
fn renderer_app_hit_test_misses_empty_time() {
    let node_id = NodeId::new();
    let track_id = TrackId::new();
    let mut renderer = TimelineRendererApp::new();
    renderer.set_projection(projection_with_clip(node_id, track_id, 1_000, 4_000));

    assert_eq!(renderer.hit_test_clip_at_time(track_id, 4_000), Ok(None));
    assert_eq!(
        renderer.select_clip_at_time(track_id, 4_000),
        Err(TimelineRendererError::NoClipAtTime {
            track_id,
            time_ms: 4_000
        })
    );
    assert!(renderer.drain_commands().is_empty());
}

#[test]
fn renderer_app_keeps_transient_viewport_inside_projection_duration() {
    let node_id = NodeId::new();
    let mut renderer = TimelineRendererApp::new();
    renderer.set_projection(projection_with_node(node_id));

    assert_eq!(
        renderer.viewport(),
        TimelineViewport {
            start_ms: 0,
            end_ms: 10_000,
            duration_ms: 10_000
        }
    );

    assert_eq!(renderer.set_viewport(2_000, 6_000), Ok(()));
    renderer.pan_viewport(10_000);
    assert_eq!(
        renderer.viewport(),
        TimelineViewport {
            start_ms: 6_000,
            end_ms: 10_000,
            duration_ms: 10_000
        }
    );

    renderer.pan_viewport(-10_000);
    assert_eq!(
        renderer.viewport(),
        TimelineViewport {
            start_ms: 0,
            end_ms: 4_000,
            duration_ms: 10_000
        }
    );
}

#[test]
fn renderer_app_keeps_transient_playhead_inside_projection_duration() {
    let node_id = NodeId::new();
    let mut renderer = TimelineRendererApp::new();
    renderer.set_projection(projection_with_node(node_id));

    assert_eq!(
        renderer.playhead(),
        TimelinePlayhead {
            position_ms: 0,
            duration_ms: 10_000
        }
    );

    assert_eq!(renderer.set_playhead(4_000), Ok(()));
    assert_eq!(
        renderer.playhead(),
        TimelinePlayhead {
            position_ms: 4_000,
            duration_ms: 10_000
        }
    );

    assert_eq!(
        renderer.set_playhead(12_000),
        Err(TimelineRendererError::InvalidPlayheadPosition {
            position_ms: 12_000,
            duration_ms: 10_000
        })
    );
}

#[test]
fn renderer_app_zooms_viewport_around_time() {
    let node_id = NodeId::new();
    let mut renderer = TimelineRendererApp::new();
    renderer.set_projection(projection_with_node(node_id));

    assert_eq!(renderer.zoom_viewport_around(5_000, 2.0), Ok(()));
    assert_eq!(
        renderer.viewport(),
        TimelineViewport {
            start_ms: 2_500,
            end_ms: 7_500,
            duration_ms: 10_000
        }
    );

    assert_eq!(
        renderer.zoom_viewport_around(5_000, 0.0),
        Err(TimelineRendererError::InvalidZoomFactor)
    );
}

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
fn renderer_app_emits_validated_split_node_command() {
    let node_id = NodeId::new();
    let mut renderer = TimelineRendererApp::new();
    renderer.set_projection(projection_with_node(node_id));

    assert_eq!(renderer.request_split_node(node_id, 2_500), Ok(()));
    assert_eq!(
        renderer.drain_commands(),
        vec![TimelineRendererCommand::SplitNode {
            node_id,
            at_ms: 2_500
        }]
    );
}

#[test]
fn renderer_app_rejects_split_node_command_outside_clip() {
    let node_id = NodeId::new();
    let mut renderer = TimelineRendererApp::new();
    renderer.set_projection(projection_with_node(node_id));

    assert_eq!(
        renderer.request_split_node(node_id, 4_000),
        Err(TimelineRendererError::InvalidNodeSplit {
            at_ms: 4_000,
            start_ms: 1_000,
            end_ms: 4_000
        })
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

fn projection_with_node(node_id: NodeId) -> TimelineRenderProjection {
    let track_id = TrackId::new();
    projection_with_clip(node_id, track_id, 1_000, 4_000)
}

fn projection_with_clip(
    node_id: NodeId,
    track_id: TrackId,
    start_ms: u64,
    end_ms: u64,
) -> TimelineRenderProjection {
    TimelineRenderProjection {
        total_duration_ms: 10_000,
        tracks: vec![TimelineRenderTrack {
            track_id,
            level: StoryLevel::Scene,
            label: "Scenes".to_string(),
            sort_order: 30,
            collapsed: false,
        }],
        clips: vec![TimelineRenderClip {
            node_id,
            parent_id: None,
            track_id,
            level: StoryLevel::Scene,
            name: "Beach argument".to_string(),
            start_ms,
            end_ms,
            sort_order: 10,
            locked: false,
            content_status: ContentStatus::NotesOnly,
            beat_type: None,
            arc_ids: Vec::new(),
        }],
        relationships: Vec::new(),
    }
}
