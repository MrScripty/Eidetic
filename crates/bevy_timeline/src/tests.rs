use super::*;
use eidetic_core::contracts::{
    AffectConfidence, AffectProvenance, AffectValueId, Arousal, EmotionalIntensity, MoodLabel,
    TimelineRenderAffectSample, TimelineRenderClip, TimelineRenderTrack, Valence,
};
use eidetic_core::timeline::node::{ContentStatus, StoryLevel};
use eidetic_core::timeline::relationship::{RelationshipId, RelationshipType};
use eidetic_core::timeline::track::TrackId;

mod split;

#[test]
fn crate_keeps_bevy_features_leaf_and_minimal() {
    let manifest = include_str!("../Cargo.toml");

    assert!(manifest.contains("bevy = { version = \"0.18.1\""));
    assert!(manifest.contains("default-features = false"));
    assert!(manifest.contains("features = [\"std\"]"));
    assert!(manifest.contains("[features]"));
    assert!(manifest.contains("default = []"));
    assert!(manifest.contains("native_render = ["));
    assert!(manifest.contains("\"bevy/2d_bevy_render\""));
    assert!(manifest.contains("\"bevy/bevy_winit\""));
    assert!(manifest.contains("\"bevy/bevy_window\""));
    assert!(manifest.contains("\"bevy/wayland\""));
    assert!(manifest.contains("\"bevy/x11\""));
    assert!(!manifest.contains("\"bevy_text\""));
    assert!(!manifest.contains("\"bevy_ui\""));
}

#[cfg(feature = "native_render")]
#[test]
fn native_window_runner_config_records_minimal_smoke_window_intent() {
    let config = TimelineNativeWindowRunnerConfig::minimal_smoke(true);

    assert_eq!(config.title, "Eidetic Timeline");
    assert_eq!(config.width_px, 1280);
    assert_eq!(config.height_px, 360);
    assert!(!config.borderless_window);
    assert!(config.run_on_any_thread);
    assert_eq!(config.auto_close_after_ms, None);
    assert_eq!(config.initial_projection, None);

    let auto_close_ms = std::num::NonZeroU64::new(250).unwrap();
    let config = config.with_auto_close_after_ms(auto_close_ms);

    assert_eq!(config.auto_close_after_ms, Some(auto_close_ms));
}

#[cfg(feature = "native_render")]
#[test]
fn controlled_native_window_app_builds_scene_from_initial_projection() {
    let node_id = NodeId::new();
    let mut app = bevy::prelude::App::new();
    let config = TimelineNativeWindowRunnerConfig::minimal_smoke(true)
        .with_initial_projection(projection_with_node(node_id));

    app.add_plugins(crate::native_render::TimelineNativeRenderPlugin);
    crate::native_render::seed_initial_timeline_native_render_scene(
        &mut app,
        config.initial_projection.as_ref(),
    );

    let stats = app.world().resource::<TimelineSceneStats>();
    assert_eq!(stats.track_count, 1);
    assert_eq!(stats.clip_count, 1);
    assert_eq!(stats.relationship_count, 0);

    let mut visuals = app
        .world_mut()
        .query::<&crate::native_render::TimelineNativeClipVisual>();
    let clips: Vec<_> = visuals.iter(app.world()).collect();
    assert_eq!(clips.len(), 1);
    assert_eq!(clips[0].node_id, node_id);
    assert_eq!(clips[0].level, StoryLevel::Scene);
    assert_eq!(clips[0].content_status, ContentStatus::NotesOnly);
    assert!(!clips[0].locked);
    assert!(!clips[0].selected);
    assert_eq!(clips[0].color_rgb, [0.342, 0.655, 0.691]);
    assert!(clips[0].width_px > 0.0);
}

#[cfg(feature = "native_render")]
#[test]
fn controlled_native_window_app_applies_projection_updates() {
    let node_id = NodeId::new();
    let control = TimelineNativeWindowControlHandle::new();
    let mut app = bevy::prelude::App::new();

    app.add_plugins(crate::native_render::TimelineNativeRenderPlugin);
    app.insert_resource(TimelineNativeWindowControl::from(&control));

    control
        .request_projection_update(projection_with_node(node_id))
        .unwrap();
    crate::native_render::apply_timeline_native_projection_updates(app.world_mut());

    let stats = app.world().resource::<TimelineSceneStats>();
    assert_eq!(stats.track_count, 1);
    assert_eq!(stats.clip_count, 1);

    let mut visuals = app
        .world_mut()
        .query::<&crate::native_render::TimelineNativeClipVisual>();
    assert_eq!(visuals.iter(app.world()).count(), 1);
}

#[cfg(feature = "native_render")]
#[test]
fn controlled_native_window_replaces_projection_derived_clip_visuals() {
    let node_id = NodeId::new();
    let mut app = bevy::prelude::App::new();

    app.add_plugins(crate::native_render::TimelineNativeRenderPlugin);
    crate::native_render::rebuild_timeline_native_visuals(
        app.world_mut(),
        &projection_with_node(node_id),
    );

    let mut empty_projection = projection_with_node(node_id);
    empty_projection.clips.clear();
    crate::native_render::rebuild_timeline_native_visuals(app.world_mut(), &empty_projection);

    let mut visuals = app
        .world_mut()
        .query::<&crate::native_render::TimelineNativeClipVisual>();
    assert_eq!(visuals.iter(app.world()).count(), 0);
}

#[cfg(feature = "native_render")]
#[test]
fn controlled_native_window_highlights_projected_selected_clip() {
    let node_id = NodeId::new();
    let mut projection = projection_with_node(node_id);
    projection.selected_node_id = Some(node_id);
    let mut app = bevy::prelude::App::new();

    app.add_plugins(crate::native_render::TimelineNativeRenderPlugin);
    crate::native_render::seed_initial_timeline_native_render_scene(&mut app, Some(&projection));

    let mut visuals = app
        .world_mut()
        .query::<&crate::native_render::TimelineNativeClipVisual>();
    let clips: Vec<_> = visuals.iter(app.world()).collect();
    assert_eq!(clips.len(), 1);
    assert!(clips[0].selected);
    assert_eq!(clips[0].color_rgb, [0.957, 0.769, 0.188]);
    assert!(clips[0].height_px > 42.0);
}

#[cfg(feature = "native_render")]
#[test]
fn controlled_native_window_clip_visuals_use_projection_status_colors() {
    assert_eq!(
        crate::native_style::native_clip_color_rgb(
            StoryLevel::Scene,
            false,
            ContentStatus::NotesOnly,
            false,
        ),
        [0.342, 0.655, 0.691]
    );
    assert_eq!(
        crate::native_style::native_clip_color_rgb(
            StoryLevel::Beat,
            false,
            ContentStatus::Empty,
            false,
        ),
        [0.188, 0.227, 0.298]
    );
    assert_eq!(
        crate::native_style::native_clip_color_rgb(
            StoryLevel::Act,
            false,
            ContentStatus::Generating,
            false,
        ),
        [0.937, 0.706, 0.294]
    );
    assert_eq!(
        crate::native_style::native_clip_color_rgb(
            StoryLevel::Sequence,
            false,
            ContentStatus::HasContent,
            false,
        ),
        [0.282, 0.686, 0.424]
    );
    assert_eq!(
        crate::native_style::native_clip_color_rgb(
            StoryLevel::Premise,
            true,
            ContentStatus::Empty,
            false,
        ),
        [0.431, 0.455, 0.502]
    );
    assert_eq!(
        crate::native_style::native_clip_color_rgb(
            StoryLevel::Premise,
            true,
            ContentStatus::Empty,
            true,
        ),
        [0.957, 0.769, 0.188]
    );
}

#[cfg(feature = "native_render")]
#[test]
fn controlled_native_window_renders_projection_relationship_visuals() {
    let upper_node_id = NodeId::new();
    let lower_node_id = NodeId::new();
    let upper_track_id = TrackId::new();
    let lower_track_id = TrackId::new();
    let relationship_id = RelationshipId::new();
    let mut projection =
        projection_with_two_tracks(upper_node_id, upper_track_id, lower_node_id, lower_track_id);
    projection.relationships = vec![eidetic_core::contracts::TimelineRenderRelationship {
        relationship_id,
        from_node_id: upper_node_id,
        to_node_id: lower_node_id,
        relationship_type: RelationshipType::Causal,
    }];
    let mut app = bevy::prelude::App::new();

    app.add_plugins(crate::native_render::TimelineNativeRenderPlugin);
    crate::native_render::seed_initial_timeline_native_render_scene(&mut app, Some(&projection));

    let mut visuals = app
        .world_mut()
        .query::<&crate::native_render::TimelineNativeRelationshipVisual>();
    let relationship_visuals: Vec<_> = visuals.iter(app.world()).collect();
    assert_eq!(relationship_visuals.len(), 1);
    assert_eq!(relationship_visuals[0].relationship_id, relationship_id);
    assert_eq!(relationship_visuals[0].from_node_id, upper_node_id);
    assert_eq!(relationship_visuals[0].to_node_id, lower_node_id);
    assert_eq!(
        relationship_visuals[0].relationship_type,
        RelationshipType::Causal
    );
    assert_eq!(relationship_visuals[0].color_rgb, [0.937, 0.384, 0.314]);
    assert!(relationship_visuals[0].length_px > 1.0);
}

#[cfg(feature = "native_render")]
#[test]
fn controlled_native_window_relationship_visuals_use_projection_type_colors() {
    assert_eq!(
        crate::native_style::native_relationship_color_rgb(&RelationshipType::Causal),
        [0.937, 0.384, 0.314]
    );
    assert_eq!(
        crate::native_style::native_relationship_color_rgb(&RelationshipType::Thematic),
        [0.933, 0.831, 0.455]
    );
    assert_eq!(
        crate::native_style::native_relationship_color_rgb(&RelationshipType::Convergence {
            arc_ids: Vec::new()
        }),
        [0.655, 0.463, 0.914]
    );
}

#[cfg(feature = "native_render")]
#[test]
fn controlled_native_window_renders_projection_affect_overlay_visuals() {
    let node_id = NodeId::new();
    let affect_id = AffectValueId::new();
    let mut projection = projection_with_node(node_id);
    projection.affect_overlays = vec![TimelineRenderAffectSample {
        affect_id,
        node_id,
        start_ms: 1_500,
        end_ms: 3_500,
        valence: Valence::new(-400).unwrap(),
        arousal: Arousal::new(700).unwrap(),
        intensity: EmotionalIntensity::new(500).unwrap(),
        confidence: AffectConfidence::new(900).unwrap(),
        mood_labels: vec![MoodLabel::new("tense").unwrap()],
        provenance: AffectProvenance::UserAuthored,
    }];
    let mut app = bevy::prelude::App::new();

    app.add_plugins(crate::native_render::TimelineNativeRenderPlugin);
    crate::native_render::seed_initial_timeline_native_render_scene(&mut app, Some(&projection));

    let mut visuals = app
        .world_mut()
        .query::<&crate::native_render::TimelineNativeAffectOverlayVisual>();
    let affect_visuals: Vec<_> = visuals.iter(app.world()).collect();
    assert_eq!(affect_visuals.len(), 1);
    assert_eq!(affect_visuals[0].affect_id, affect_id);
    assert_eq!(affect_visuals[0].node_id, node_id);
    assert_eq!(affect_visuals[0].start_ms, 1_500);
    assert_eq!(affect_visuals[0].end_ms, 3_500);
    assert_eq!(affect_visuals[0].color_rgb, [0.376, 0.592, 0.827]);
    assert!(affect_visuals[0].width_px > 1.0);
    assert_eq!(affect_visuals[0].height_px, 7.0);
}

#[cfg(feature = "native_render")]
#[test]
fn controlled_native_window_affect_overlay_visuals_use_projection_colors() {
    assert_eq!(
        crate::native_style::native_affect_color_rgb(Valence::new(-400).unwrap()),
        [0.376, 0.592, 0.827]
    );
    assert_eq!(
        crate::native_style::native_affect_color_rgb(Valence::new(0).unwrap()),
        [0.933, 0.831, 0.455]
    );
    assert_eq!(
        crate::native_style::native_affect_color_rgb(Valence::new(400).unwrap()),
        [0.282, 0.686, 0.424]
    );
    assert_eq!(
        crate::native_style::native_affect_height_px(EmotionalIntensity::new(0).unwrap()),
        4.0
    );
    assert_eq!(
        crate::native_style::native_affect_height_px(EmotionalIntensity::new(1_000).unwrap()),
        10.0
    );
}

#[cfg(feature = "native_render")]
#[test]
fn controlled_native_window_clip_visuals_use_transient_viewport() {
    let node_id = NodeId::new();
    let projection = projection_with_node(node_id);
    let mut app = bevy::prelude::App::new();

    app.add_plugins(crate::native_render::TimelineNativeRenderPlugin);
    crate::native_render::seed_initial_timeline_native_render_scene(&mut app, Some(&projection));
    crate::native_render::set_timeline_native_viewport(app.world_mut(), 4_000, 8_000).unwrap();
    crate::native_render::rebuild_timeline_native_visuals(app.world_mut(), &projection);

    let mut visuals = app
        .world_mut()
        .query::<&crate::native_render::TimelineNativeClipVisual>();
    assert_eq!(visuals.iter(app.world()).count(), 0);
}

#[cfg(feature = "native_render")]
#[test]
fn controlled_native_window_rejects_invalid_transient_viewport() {
    let node_id = NodeId::new();
    let mut app = bevy::prelude::App::new();

    app.add_plugins(crate::native_render::TimelineNativeRenderPlugin);
    crate::native_render::seed_initial_timeline_native_render_scene(
        &mut app,
        Some(&projection_with_node(node_id)),
    );

    assert_eq!(
        crate::native_render::set_timeline_native_viewport(app.world_mut(), 8_000, 4_000),
        Err(TimelineRendererError::InvalidViewportRange {
            start_ms: 8_000,
            end_ms: 4_000,
            duration_ms: 10_000,
        })
    );
}

#[cfg(feature = "native_render")]
#[test]
fn controlled_native_window_pans_transient_viewport_and_rebuilds_visuals() {
    let node_id = NodeId::new();
    let projection = projection_with_node(node_id);
    let mut app = bevy::prelude::App::new();

    app.add_plugins(crate::native_render::TimelineNativeRenderPlugin);
    crate::native_render::seed_initial_timeline_native_render_scene(&mut app, Some(&projection));
    crate::native_render::set_timeline_native_viewport(app.world_mut(), 0, 5_000).unwrap();

    let viewport = crate::native_render::pan_timeline_native_viewport(app.world_mut(), 1_000)
        .expect("viewport pan");

    assert_eq!(viewport.start_ms, 1_000);
    assert_eq!(viewport.end_ms, 6_000);
    let mut visuals = app
        .world_mut()
        .query::<&crate::native_render::TimelineNativeClipVisual>();
    assert_eq!(visuals.iter(app.world()).count(), 1);
}

#[cfg(feature = "native_render")]
#[test]
fn controlled_native_window_zooms_transient_viewport_and_rebuilds_visuals() {
    let node_id = NodeId::new();
    let projection = projection_with_node(node_id);
    let mut app = bevy::prelude::App::new();

    app.add_plugins(crate::native_render::TimelineNativeRenderPlugin);
    crate::native_render::seed_initial_timeline_native_render_scene(&mut app, Some(&projection));

    let viewport = crate::native_render::zoom_timeline_native_viewport(app.world_mut(), 2.0)
        .expect("viewport zoom");

    assert_eq!(viewport.width_ms(), 5_000);
    let mut visuals = app
        .world_mut()
        .query::<&crate::native_render::TimelineNativeClipVisual>();
    assert_eq!(visuals.iter(app.world()).count(), 1);
}

#[cfg(feature = "native_render")]
#[test]
fn controlled_native_window_renders_transient_playhead_visual() {
    let node_id = NodeId::new();
    let projection = projection_with_node(node_id);
    let mut app = bevy::prelude::App::new();

    app.add_plugins(crate::native_render::TimelineNativeRenderPlugin);
    crate::native_render::seed_initial_timeline_native_render_scene(&mut app, Some(&projection));

    let playhead =
        crate::native_render::set_timeline_native_playhead(app.world_mut(), 5_000).unwrap();

    assert_eq!(
        playhead,
        TimelinePlayhead {
            position_ms: 5_000,
            duration_ms: 10_000,
        }
    );
    let mut visuals = app
        .world_mut()
        .query::<&crate::native_render::TimelineNativePlayheadVisual>();
    let playhead_visuals: Vec<_> = visuals.iter(app.world()).collect();
    assert_eq!(playhead_visuals.len(), 1);
    assert_eq!(playhead_visuals[0].position_ms, 5_000);
    assert_eq!(playhead_visuals[0].x_px, 0.0);
    assert!(playhead_visuals[0].height_px > 0.0);
}

#[cfg(feature = "native_render")]
#[test]
fn controlled_native_window_rejects_invalid_transient_playhead_position() {
    let node_id = NodeId::new();
    let mut app = bevy::prelude::App::new();

    app.add_plugins(crate::native_render::TimelineNativeRenderPlugin);
    crate::native_render::seed_initial_timeline_native_render_scene(
        &mut app,
        Some(&projection_with_node(node_id)),
    );

    assert_eq!(
        crate::native_render::set_timeline_native_playhead(app.world_mut(), 12_000),
        Err(TimelineRendererError::InvalidPlayheadPosition {
            position_ms: 12_000,
            duration_ms: 10_000,
        })
    );
}

#[cfg(feature = "native_render")]
#[test]
fn controlled_native_window_nudges_transient_playhead_and_rebuilds_visual() {
    let node_id = NodeId::new();
    let projection = projection_with_node(node_id);
    let mut app = bevy::prelude::App::new();

    app.add_plugins(crate::native_render::TimelineNativeRenderPlugin);
    crate::native_render::seed_initial_timeline_native_render_scene(&mut app, Some(&projection));

    let playhead = crate::native_render::nudge_timeline_native_playhead(app.world_mut(), 2_500);

    assert_eq!(
        playhead,
        TimelinePlayhead {
            position_ms: 2_500,
            duration_ms: 10_000,
        }
    );
    let mut visuals = app
        .world_mut()
        .query::<&crate::native_render::TimelineNativePlayheadVisual>();
    let playhead_visuals: Vec<_> = visuals.iter(app.world()).collect();
    assert_eq!(playhead_visuals.len(), 1);
    assert_eq!(playhead_visuals[0].position_ms, 2_500);
    assert!(playhead_visuals[0].x_px < 0.0);

    let playhead = crate::native_render::nudge_timeline_native_playhead(app.world_mut(), -10_000);
    assert_eq!(playhead.position_ms, 0);
}

#[cfg(feature = "native_render")]
#[test]
fn controlled_native_window_nudges_transient_playhead_inside_duration() {
    let node_id = NodeId::new();
    let projection = projection_with_node(node_id);
    let mut app = bevy::prelude::App::new();

    app.add_plugins(crate::native_render::TimelineNativeRenderPlugin);
    crate::native_render::seed_initial_timeline_native_render_scene(&mut app, Some(&projection));

    let playhead = crate::native_render::nudge_timeline_native_playhead(app.world_mut(), 12_000);

    assert_eq!(playhead.position_ms, 10_000);
    assert_eq!(playhead.duration_ms, 10_000);
}

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
fn controlled_native_window_emits_validated_create_node_command() {
    let parent_id = NodeId::new();
    let node_id = NodeId::new();
    let control = TimelineNativeWindowControlHandle::new();
    let window_control = TimelineNativeWindowControl::from(&control);

    crate::native_command::emit_timeline_native_create_node_request(
        &window_control,
        &projection_with_node(parent_id),
        node_id,
        Some(parent_id),
        StoryLevel::Act,
        "Inserted act".to_string(),
        2_000,
        5_000,
        None,
    )
    .unwrap();

    assert_eq!(
        control.drain_commands(),
        vec![TimelineRendererCommand::CreateNode {
            node_id,
            parent_id: Some(parent_id),
            level: StoryLevel::Act,
            name: "Inserted act".to_string(),
            start_ms: 2_000,
            end_ms: 5_000,
            beat_type: None,
        }]
    );
}

#[cfg(feature = "native_render")]
#[test]
fn controlled_native_window_rejects_create_node_command_with_unknown_parent() {
    let known_node_id = NodeId::new();
    let parent_id = NodeId::new();
    let node_id = NodeId::new();
    let control = TimelineNativeWindowControlHandle::new();
    let window_control = TimelineNativeWindowControl::from(&control);

    assert_eq!(
        crate::native_command::emit_timeline_native_create_node_request(
            &window_control,
            &projection_with_node(known_node_id),
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
    assert!(control.drain_commands().is_empty());
}

#[cfg(feature = "native_render")]
#[test]
fn native_window_control_handle_records_close_requests() {
    let control = TimelineNativeWindowControlHandle::new();

    assert!(!control.close_requested());
    assert!(!control.ready());
    assert!(!control.visible());

    control.request_close();
    control.mark_ready();
    control.mark_visible(true);

    assert!(control.close_requested());
    assert!(control.ready());
    assert!(control.visible());
}

#[cfg(feature = "native_render")]
#[test]
fn controlled_native_window_app_installs_close_control_resource() {
    let control = TimelineNativeWindowControlHandle::new();
    let mut app = bevy::prelude::App::new();

    configure_controlled_minimal_timeline_native_window_app(
        &mut app,
        TimelineNativeWindowRunnerConfig::minimal_smoke(true),
        control.clone(),
    );

    assert!(
        app.world()
            .contains_resource::<TimelineNativeWindowControl>()
    );
    assert!(!control.close_requested());
    assert!(!control.ready());

    control.request_close();

    assert!(control.close_requested());
}

#[cfg(feature = "native_render")]
#[test]
fn controlled_native_window_os_close_requests_shutdown() {
    let control = TimelineNativeWindowControlHandle::new();
    let window_control = TimelineNativeWindowControl::from(&control);

    control.mark_visible(true);
    window_control.request_close_from_os_window();

    assert!(control.close_requested());
    assert!(!control.visible());
}

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
fn relationship_curves_serialize_for_host_bridge() {
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

pub(super) fn projection_with_node(node_id: NodeId) -> TimelineRenderProjection {
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
        selected_node_id: None,
        structure_segments: Vec::new(),
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
        gaps: Vec::new(),
        affect_overlays: Vec::new(),
    }
}

fn projection_with_two_tracks(
    upper_node_id: NodeId,
    upper_track_id: TrackId,
    lower_node_id: NodeId,
    lower_track_id: TrackId,
) -> TimelineRenderProjection {
    TimelineRenderProjection {
        total_duration_ms: 10_000,
        selected_node_id: None,
        structure_segments: Vec::new(),
        tracks: vec![
            TimelineRenderTrack {
                track_id: lower_track_id,
                level: StoryLevel::Beat,
                label: "Beats".to_string(),
                sort_order: 20,
                collapsed: false,
            },
            TimelineRenderTrack {
                track_id: upper_track_id,
                level: StoryLevel::Scene,
                label: "Scenes".to_string(),
                sort_order: 10,
                collapsed: false,
            },
        ],
        clips: vec![
            TimelineRenderClip {
                node_id: upper_node_id,
                parent_id: None,
                track_id: upper_track_id,
                level: StoryLevel::Scene,
                name: "Opening scene".to_string(),
                start_ms: 1_000,
                end_ms: 4_000,
                sort_order: 10,
                locked: false,
                content_status: ContentStatus::NotesOnly,
                beat_type: None,
                arc_ids: Vec::new(),
            },
            TimelineRenderClip {
                node_id: lower_node_id,
                parent_id: Some(upper_node_id),
                track_id: lower_track_id,
                level: StoryLevel::Beat,
                name: "Reversal".to_string(),
                start_ms: 2_000,
                end_ms: 5_000,
                sort_order: 20,
                locked: false,
                content_status: ContentStatus::NotesOnly,
                beat_type: None,
                arc_ids: Vec::new(),
            },
        ],
        relationships: Vec::new(),
        gaps: Vec::new(),
        affect_overlays: Vec::new(),
    }
}
