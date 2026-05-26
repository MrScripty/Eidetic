use eidetic_core::contracts::{
    AffectConfidence, AffectProvenance, AffectValueId, Arousal, EmotionalIntensity, MoodLabel,
    TimelineRenderAffectSample, TimelineRenderProjection, TimelineRenderRelationship, Valence,
};
use eidetic_core::timeline::node::NodeId;
use eidetic_core::timeline::relationship::{RelationshipId, RelationshipType};
use eidetic_core::timeline::track::TrackId;

use crate::{
    TimelinePlayhead, TimelineRendererApp, TimelineRendererCommand, TimelineRendererError,
    TimelineViewport, TimelineViewportGeometry, TimelineViewportPoint, relationship_curves,
};

use super::{projection_with_clip, projection_with_node, projection_with_two_tracks};

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
    assert_eq!(renderer.scene_affect_overlay_count(), 0);

    renderer.set_projection(TimelineRenderProjection {
        total_duration_ms: 10_000,
        selected_node_id: None,
        structure_segments: Vec::new(),
        tracks: Vec::new(),
        clips: Vec::new(),
        relationships: Vec::new(),
        gaps: Vec::new(),
        affect_overlays: Vec::new(),
    });

    assert_eq!(renderer.scene_counts(), (0, 0));
    assert_eq!(renderer.scene_relationship_count(), 0);
    assert_eq!(renderer.scene_affect_overlay_count(), 0);
}

#[test]
fn renderer_app_rebuilds_relationship_entities_from_projection() {
    let node_id = NodeId::new();
    let track_id = TrackId::new();
    let mut projection = projection_with_clip(node_id, track_id, 1_000, 4_000);
    projection.relationships = vec![TimelineRenderRelationship {
        relationship_id: RelationshipId::new(),
        from_node_id: node_id,
        to_node_id: node_id,
        relationship_type: RelationshipType::Thematic,
    }];
    let mut renderer = TimelineRendererApp::new();

    renderer.set_projection(projection);

    assert_eq!(renderer.scene_counts(), (1, 1));
    assert_eq!(renderer.scene_relationship_count(), 1);
}

#[test]
fn renderer_app_rebuilds_affect_overlay_entities_from_projection() {
    let node_id = NodeId::new();
    let affect_id = AffectValueId::new();
    let track_id = TrackId::new();
    let mut projection = projection_with_clip(node_id, track_id, 1_000, 5_000);
    projection.affect_overlays = vec![TimelineRenderAffectSample {
        affect_id,
        node_id,
        start_ms: 1_000,
        end_ms: 5_000,
        valence: Valence::new(-250).unwrap(),
        arousal: Arousal::new(700).unwrap(),
        intensity: EmotionalIntensity::new(800).unwrap(),
        confidence: AffectConfidence::new(900).unwrap(),
        mood_labels: vec![MoodLabel::new("tense").unwrap()],
        provenance: AffectProvenance::UserAuthored,
    }];
    let mut renderer = TimelineRendererApp::new();

    renderer.set_projection(projection);

    assert_eq!(renderer.scene_counts(), (1, 1));
    assert_eq!(renderer.scene_affect_overlay_count(), 1);
}

#[test]
fn renderer_app_derives_relationship_curves_from_projection() {
    let node_id = NodeId::new();
    let track_id = TrackId::new();
    let relationship_id = RelationshipId::new();
    let mut projection = projection_with_clip(node_id, track_id, 1_000, 5_000);
    projection.relationships = vec![TimelineRenderRelationship {
        relationship_id,
        from_node_id: node_id,
        to_node_id: node_id,
        relationship_type: RelationshipType::Thematic,
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
fn relationship_curves_serialize_for_host_bridge() {
    let node_id = NodeId::new();
    let track_id = TrackId::new();
    let relationship_id = RelationshipId::new();
    let mut projection = projection_with_clip(node_id, track_id, 1_000, 5_000);
    projection.relationships = vec![TimelineRenderRelationship {
        relationship_id,
        from_node_id: node_id,
        to_node_id: node_id,
        relationship_type: RelationshipType::Thematic,
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
    projection.relationships = vec![TimelineRenderRelationship {
        relationship_id: RelationshipId::new(),
        from_node_id: node_id,
        to_node_id: missing_node_id,
        relationship_type: RelationshipType::Thematic,
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
fn renderer_app_hit_tests_clip_by_viewport_point_without_projection_ownership() {
    let upper_node_id = NodeId::new();
    let lower_node_id = NodeId::new();
    let upper_track_id = TrackId::new();
    let lower_track_id = TrackId::new();
    let mut renderer = TimelineRendererApp::new();
    renderer.set_projection(projection_with_two_tracks(
        upper_node_id,
        upper_track_id,
        lower_node_id,
        lower_track_id,
    ));

    assert_eq!(
        renderer.hit_test_clip_at_point(
            TimelineViewportGeometry::new(1_000, 80, 40),
            TimelineViewportPoint::new(300, 10),
        ),
        Ok(Some(upper_node_id))
    );
    assert_eq!(
        renderer.hit_test_clip_at_point(
            TimelineViewportGeometry::new(1_000, 80, 40),
            TimelineViewportPoint::new(300, 50),
        ),
        Ok(Some(lower_node_id))
    );
}

#[test]
fn renderer_app_hit_test_point_uses_current_transient_viewport() {
    let node_id = NodeId::new();
    let track_id = TrackId::new();
    let mut renderer = TimelineRendererApp::new();
    renderer.set_projection(projection_with_clip(node_id, track_id, 4_000, 6_000));
    renderer.set_viewport(2_000, 6_000).unwrap();

    assert_eq!(
        renderer.hit_test_clip_at_point(
            TimelineViewportGeometry::new(400, 40, 40),
            TimelineViewportPoint::new(200, 20),
        ),
        Ok(Some(node_id))
    );
    assert_eq!(
        renderer.hit_test_clip_at_point(
            TimelineViewportGeometry::new(400, 40, 40),
            TimelineViewportPoint::new(100, 20),
        ),
        Ok(None)
    );
}

#[test]
fn renderer_app_rejects_invalid_viewport_geometry_for_point_hit_testing() {
    let node_id = NodeId::new();
    let mut renderer = TimelineRendererApp::new();
    renderer.set_projection(projection_with_node(node_id));

    assert_eq!(
        renderer.hit_test_clip_at_point(
            TimelineViewportGeometry::new(0, 40, 40),
            TimelineViewportPoint::new(0, 0),
        ),
        Err(TimelineRendererError::InvalidViewportGeometry {
            width_px: 0,
            height_px: 40,
            track_height_px: 40,
        })
    );
}

#[test]
fn renderer_app_point_hit_testing_misses_outside_geometry_or_tracks() {
    let node_id = NodeId::new();
    let mut renderer = TimelineRendererApp::new();
    renderer.set_projection(projection_with_node(node_id));

    assert_eq!(
        renderer.hit_test_clip_at_point(
            TimelineViewportGeometry::new(100, 40, 40),
            TimelineViewportPoint::new(100, 20),
        ),
        Ok(None)
    );
    assert_eq!(
        renderer.hit_test_clip_at_point(
            TimelineViewportGeometry::new(100, 80, 40),
            TimelineViewportPoint::new(20, 60),
        ),
        Ok(None)
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
