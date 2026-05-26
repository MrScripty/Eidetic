use eidetic_core::timeline::node::NodeId;

use crate::{TimelinePlayhead, TimelineRendererError};

use super::projection_with_node;

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
