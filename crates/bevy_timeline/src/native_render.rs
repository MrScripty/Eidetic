use std::num::NonZeroU64;
use std::sync::atomic::Ordering;
use std::time::{Duration, Instant};

use bevy::app::AppExit;
use bevy::prelude::{
    App, ClearColor, Color, MessageWriter, Plugin, PluginGroup, Query, Res, Resource, Startup,
    Update, Window, World,
};
use bevy::window::{
    ExitCondition, PrimaryWindow, WindowCloseRequested, WindowPlugin, WindowResolution,
};
use bevy::winit::WinitPlugin;
use eidetic_core::contracts::TimelineRenderProjection;

#[cfg(test)]
pub(crate) use crate::native_visual::{
    TimelineNativeAffectOverlayVisual, TimelineNativeClipVisual, TimelineNativePlayheadVisual,
    TimelineNativeRelationshipVisual,
};
pub(crate) use crate::native_visual::{
    TimelineNativeRenderLayout, native_track_height_px, rebuild_timeline_native_visuals,
};
pub use crate::native_window_control::{
    TimelineNativeWindowControl, TimelineNativeWindowControlHandle,
    TimelineNativeWindowProjectionUpdateError,
};
use crate::scene::{TimelineSceneStats, rebuild_timeline_scene};
use crate::{TimelinePlayhead, TimelineRendererError, TimelineViewport};

#[derive(Debug, Clone, Copy, Default, PartialEq, Eq, Resource)]
pub struct TimelineNativeRenderConfig {
    pub borderless_window: bool,
}

#[derive(Resource, Default)]
pub(crate) struct TimelineNativeProjectionState {
    pub(crate) projection: Option<TimelineRenderProjection>,
    pub(crate) viewport: TimelineViewport,
    pub(crate) playhead: TimelinePlayhead,
}

#[derive(Debug, Clone, PartialEq)]
pub struct TimelineNativeWindowRunnerConfig {
    pub title: String,
    pub width_px: u32,
    pub height_px: u32,
    pub borderless_window: bool,
    pub run_on_any_thread: bool,
    pub auto_close_after_ms: Option<NonZeroU64>,
    pub initial_projection: Option<TimelineRenderProjection>,
}

#[derive(Debug, Resource)]
struct TimelineNativeAutoClose {
    close_at: Instant,
}

impl TimelineNativeWindowRunnerConfig {
    pub fn minimal_smoke(run_on_any_thread: bool) -> Self {
        Self {
            title: "Eidetic Timeline".to_string(),
            width_px: 1280,
            height_px: 360,
            borderless_window: false,
            run_on_any_thread,
            auto_close_after_ms: None,
            initial_projection: None,
        }
    }

    pub fn with_auto_close_after_ms(mut self, auto_close_after_ms: NonZeroU64) -> Self {
        self.auto_close_after_ms = Some(auto_close_after_ms);
        self
    }

    pub fn with_initial_projection(mut self, projection: TimelineRenderProjection) -> Self {
        self.initial_projection = Some(projection);
        self
    }
}

pub struct TimelineNativeRenderPlugin;

impl Plugin for TimelineNativeRenderPlugin {
    fn build(&self, app: &mut App) {
        app.insert_resource(TimelineNativeRenderConfig::default());
        app.insert_resource(TimelineNativeRenderLayout::from_window(1280, 360));
        app.insert_resource(TimelineNativeProjectionState::default());
        app.insert_resource(ClearColor(Color::srgb(0.067, 0.082, 0.114)));
        app.insert_resource(TimelineSceneStats::default());
        app.add_systems(Startup, crate::native_visual::spawn_timeline_native_camera);
        app.add_systems(Update, mark_timeline_native_window_ready);
        app.add_systems(Update, apply_timeline_native_projection_updates);
        app.add_systems(
            Update,
            crate::native_input::emit_timeline_native_selected_relationship_create,
        );
        app.add_systems(
            Update,
            crate::native_input::emit_timeline_native_click_selection,
        );
        app.add_systems(
            Update,
            crate::native_input::emit_timeline_native_selected_delete,
        );
        app.add_systems(
            Update,
            crate::native_input::emit_timeline_native_selected_split,
        );
        app.add_systems(
            Update,
            crate::native_input::emit_timeline_native_selected_create_child,
        );
        app.add_systems(
            Update,
            crate::native_input::emit_timeline_native_selected_nudge,
        );
        app.add_systems(
            Update,
            crate::native_input::emit_timeline_native_selected_resize,
        );
        app.add_systems(
            Update,
            crate::native_input::navigate_timeline_native_viewport,
        );
        app.add_systems(
            Update,
            crate::native_input::navigate_timeline_native_playhead,
        );
    }
}

pub fn configure_minimal_timeline_native_window_app(
    app: &mut App,
    config: TimelineNativeWindowRunnerConfig,
) {
    configure_controlled_minimal_timeline_native_window_app(
        app,
        config,
        TimelineNativeWindowControlHandle::new(),
    );
}

pub fn configure_controlled_minimal_timeline_native_window_app(
    app: &mut App,
    config: TimelineNativeWindowRunnerConfig,
    control_handle: TimelineNativeWindowControlHandle,
) {
    app.add_message::<AppExit>();
    app.add_message::<WindowCloseRequested>();
    app.add_plugins(
        bevy::DefaultPlugins
            .set(WindowPlugin {
                primary_window: Some(Window {
                    title: config.title,
                    resolution: WindowResolution::new(config.width_px, config.height_px),
                    decorations: !config.borderless_window,
                    ..Default::default()
                }),
                exit_condition: ExitCondition::DontExit,
                close_when_requested: false,
                ..Default::default()
            })
            .set(WinitPlugin {
                run_on_any_thread: config.run_on_any_thread,
            }),
    );
    app.add_plugins(TimelineNativeRenderPlugin);
    app.insert_resource(TimelineNativeRenderLayout::from_window(
        config.width_px,
        config.height_px,
    ));
    app.insert_resource(TimelineNativeWindowControl::from(&control_handle));
    app.add_systems(Update, close_minimal_native_window_when_requested);
    app.add_systems(Update, close_minimal_native_window_from_os_request);
    app.add_systems(Update, apply_minimal_native_window_visibility_requests);

    seed_initial_timeline_native_render_scene(app, config.initial_projection.as_ref());

    if let Some(auto_close_after_ms) = config.auto_close_after_ms {
        app.insert_resource(TimelineNativeAutoClose {
            close_at: Instant::now() + Duration::from_millis(auto_close_after_ms.get()),
        });
        app.add_systems(Update, close_minimal_native_window_after_timer);
    }
}

pub(crate) fn seed_initial_timeline_native_render_scene(
    app: &mut App,
    projection: Option<&TimelineRenderProjection>,
) {
    let Some(projection) = projection else {
        return;
    };
    set_timeline_native_projection_state(app.world_mut(), projection);
    rebuild_timeline_scene(app.world_mut(), projection);
    rebuild_timeline_native_visuals(app.world_mut(), projection);
}

pub(crate) fn apply_timeline_native_projection_updates(world: &mut bevy::prelude::World) {
    let latest_projection = {
        let Some(control) = world.get_resource::<TimelineNativeWindowControl>() else {
            return;
        };
        let Ok(receiver) = control.projection_receiver.lock() else {
            return;
        };
        let mut latest_projection = None;
        while let Ok(projection) = receiver.try_recv() {
            latest_projection = Some(projection);
        }
        latest_projection
    };

    if let Some(projection) = latest_projection {
        set_timeline_native_projection_state(world, &projection);
        rebuild_timeline_scene(world, &projection);
        rebuild_timeline_native_visuals(world, &projection);
    }
}

fn set_timeline_native_projection_state(world: &mut World, projection: &TimelineRenderProjection) {
    let mut state = world.resource_mut::<TimelineNativeProjectionState>();
    state.viewport.set_duration(projection.total_duration_ms);
    state.playhead.set_duration(projection.total_duration_ms);
    state.projection = Some(projection.clone());
}

pub fn set_timeline_native_viewport(
    world: &mut World,
    start_ms: u64,
    end_ms: u64,
) -> Result<(), TimelineRendererError> {
    let mut state = world.resource_mut::<TimelineNativeProjectionState>();
    let duration_ms = state.viewport.duration_ms;
    if start_ms >= end_ms || end_ms > duration_ms {
        return Err(TimelineRendererError::InvalidViewportRange {
            start_ms,
            end_ms,
            duration_ms,
        });
    }
    state.viewport.set_range(start_ms, end_ms);
    rebuild_timeline_native_visuals_from_state(world);
    Ok(())
}

pub fn pan_timeline_native_viewport(
    world: &mut World,
    delta_ms: i64,
) -> Result<TimelineViewport, TimelineRendererError> {
    let viewport = {
        let mut state = world.resource_mut::<TimelineNativeProjectionState>();
        state.viewport.pan_by(delta_ms);
        state.viewport
    };
    rebuild_timeline_native_visuals_from_state(world);
    Ok(viewport)
}

pub fn zoom_timeline_native_viewport(
    world: &mut World,
    factor: f32,
) -> Result<TimelineViewport, TimelineRendererError> {
    if !factor.is_finite() || factor <= 0.0 {
        return Err(TimelineRendererError::InvalidZoomFactor);
    }
    let viewport = {
        let mut state = world.resource_mut::<TimelineNativeProjectionState>();
        let center_ms = state
            .viewport
            .start_ms
            .saturating_add(state.viewport.width_ms() / 2);
        state.viewport.zoom_around(center_ms, factor);
        state.viewport
    };
    rebuild_timeline_native_visuals_from_state(world);
    Ok(viewport)
}

pub fn set_timeline_native_playhead(
    world: &mut World,
    position_ms: u64,
) -> Result<TimelinePlayhead, TimelineRendererError> {
    let playhead = {
        let mut state = world.resource_mut::<TimelineNativeProjectionState>();
        let duration_ms = state.playhead.duration_ms;
        if position_ms > duration_ms {
            return Err(TimelineRendererError::InvalidPlayheadPosition {
                position_ms,
                duration_ms,
            });
        }
        state.playhead.set_position(position_ms);
        state.playhead
    };
    rebuild_timeline_native_visuals_from_state(world);
    Ok(playhead)
}

pub fn nudge_timeline_native_playhead(world: &mut World, delta_ms: i64) -> TimelinePlayhead {
    let playhead = {
        let mut state = world.resource_mut::<TimelineNativeProjectionState>();
        let next_position_ms = if delta_ms.is_negative() {
            state
                .playhead
                .position_ms
                .saturating_sub(delta_ms.unsigned_abs())
        } else {
            state
                .playhead
                .position_ms
                .saturating_add(delta_ms as u64)
                .min(state.playhead.duration_ms)
        };
        state.playhead.set_position(next_position_ms);
        state.playhead
    };
    rebuild_timeline_native_visuals_from_state(world);
    playhead
}

fn rebuild_timeline_native_visuals_from_state(world: &mut World) {
    let projection = world
        .resource::<TimelineNativeProjectionState>()
        .projection
        .clone();
    if let Some(projection) = projection {
        rebuild_timeline_native_visuals(world, &projection);
    }
}

pub fn run_minimal_timeline_native_window(config: TimelineNativeWindowRunnerConfig) {
    run_controlled_minimal_timeline_native_window(config, TimelineNativeWindowControlHandle::new());
}

pub fn run_controlled_minimal_timeline_native_window(
    config: TimelineNativeWindowRunnerConfig,
    control_handle: TimelineNativeWindowControlHandle,
) {
    let mut app = App::new();
    configure_controlled_minimal_timeline_native_window_app(&mut app, config, control_handle);
    app.run();
}

fn mark_timeline_native_window_ready(control: Option<Res<TimelineNativeWindowControl>>) {
    let Some(control) = control else {
        return;
    };
    control.ready.store(true, Ordering::Release);
    control.visible.store(true, Ordering::Release);
}

fn close_minimal_native_window_after_timer(
    auto_close: Res<TimelineNativeAutoClose>,
    mut app_exit: MessageWriter<AppExit>,
) {
    if Instant::now() >= auto_close.close_at {
        app_exit.write(AppExit::Success);
    }
}

fn close_minimal_native_window_when_requested(
    control: Option<Res<TimelineNativeWindowControl>>,
    mut app_exit: MessageWriter<AppExit>,
) {
    let Some(control) = control else {
        return;
    };
    if control.shutdown_requested.load(Ordering::Acquire) {
        app_exit.write(AppExit::Success);
    }
}

fn close_minimal_native_window_from_os_request(
    control: Option<Res<TimelineNativeWindowControl>>,
    mut requests: bevy::prelude::MessageReader<WindowCloseRequested>,
) {
    let Some(control) = control else {
        return;
    };
    for _ in requests.read() {
        control.request_close_from_os_window();
    }
}

fn apply_minimal_native_window_visibility_requests(
    control: Res<TimelineNativeWindowControl>,
    mut windows: Query<&mut Window, bevy::prelude::With<PrimaryWindow>>,
) {
    if control.hide_requested.swap(false, Ordering::AcqRel) {
        for mut window in &mut windows {
            window.visible = false;
        }
        control.visible.store(false, Ordering::Release);
    }

    if control.show_requested.swap(false, Ordering::AcqRel) {
        for mut window in &mut windows {
            window.visible = true;
        }
        control.visible.store(true, Ordering::Release);
    }
}
