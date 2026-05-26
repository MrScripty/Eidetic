use eidetic_core::contracts::{
    AffectConfidence, AffectProvenance, AffectValueId, Arousal, EmotionalIntensity, MoodLabel,
    TimelineRenderAffectSample, TimelineRenderRelationship, Valence,
};
use eidetic_core::timeline::node::{ContentStatus, NodeId, StoryLevel};
use eidetic_core::timeline::relationship::{RelationshipId, RelationshipType};
use eidetic_core::timeline::track::TrackId;

use super::{projection_with_node, projection_with_two_tracks};

#[cfg(feature = "native_render")]
#[test]
fn controlled_native_window_app_builds_scene_from_initial_projection() {
    let node_id = NodeId::new();
    let mut app = bevy::prelude::App::new();
    let config = crate::TimelineNativeWindowRunnerConfig::minimal_smoke(true)
        .with_initial_projection(projection_with_node(node_id));

    app.add_plugins(crate::native_render::TimelineNativeRenderPlugin);
    crate::native_render::seed_initial_timeline_native_render_scene(
        &mut app,
        config.initial_projection.as_ref(),
    );

    let stats = app.world().resource::<crate::TimelineSceneStats>();
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
    let control = crate::TimelineNativeWindowControlHandle::new();
    let mut app = bevy::prelude::App::new();

    app.add_plugins(crate::native_render::TimelineNativeRenderPlugin);
    app.insert_resource(crate::TimelineNativeWindowControl::from(&control));

    control
        .request_projection_update(projection_with_node(node_id))
        .unwrap();
    crate::native_render::apply_timeline_native_projection_updates(app.world_mut());

    let stats = app.world().resource::<crate::TimelineSceneStats>();
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
    projection.relationships = vec![TimelineRenderRelationship {
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
