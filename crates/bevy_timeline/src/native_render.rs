use std::num::NonZeroU64;
use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::{Arc, Mutex, mpsc};
use std::time::{Duration, Instant};

use bevy::app::AppExit;
use bevy::prelude::{
    App, ButtonInput, Camera2d, ClearColor, Color, Commands, Component, Entity, KeyCode,
    MessageWriter, MouseButton, Plugin, PluginGroup, Query, Res, Resource, Startup, Transform,
    Update, Vec2, Vec3, Window, World,
};
use bevy::sprite::Sprite;
use bevy::window::{
    ExitCondition, PrimaryWindow, WindowCloseRequested, WindowPlugin, WindowResolution,
};
use bevy::winit::WinitPlugin;
use eidetic_core::contracts::TimelineRenderProjection;
use eidetic_core::timeline::node::{BeatType, NodeId, StoryLevel};
use eidetic_core::timeline::track::TrackId;

use crate::scene::{TimelineSceneEntity, TimelineSceneStats, rebuild_timeline_scene};
use crate::{
    TimelinePlayhead, TimelineRendererCommand, TimelineRendererError, TimelineViewport,
    TimelineViewportGeometry, TimelineViewportPoint, hit_test_projection_clip_at_point,
};

const TIMELINE_NATIVE_PROJECTION_QUEUE_CAPACITY: usize = 8;
const TIMELINE_NATIVE_COMMAND_QUEUE_CAPACITY: usize = 128;
const TIMELINE_NATIVE_CLIP_HEIGHT_PX: f32 = 42.0;
const TIMELINE_NATIVE_TRACK_GAP_PX: f32 = 16.0;
const TIMELINE_NATIVE_HORIZONTAL_PADDING_PX: f32 = 48.0;
const TIMELINE_NATIVE_TOP_PADDING_PX: f32 = 48.0;
const TIMELINE_NATIVE_PLAYHEAD_WIDTH_PX: f32 = 3.0;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Resource)]
pub struct TimelineNativeRenderConfig {
    pub borderless_window: bool,
}

#[derive(Debug, Clone, Copy, PartialEq, Resource)]
pub struct TimelineNativeRenderLayout {
    pub width_px: f32,
    pub height_px: f32,
    pub clip_height_px: f32,
    pub track_gap_px: f32,
    pub horizontal_padding_px: f32,
    pub top_padding_px: f32,
}

#[derive(Resource, Default)]
struct TimelineNativeProjectionState {
    projection: Option<TimelineRenderProjection>,
    viewport: TimelineViewport,
    playhead: TimelinePlayhead,
}

#[derive(Component)]
pub struct TimelineNativeVisualEntity;

#[derive(Component)]
pub struct TimelineNativeCamera;

#[derive(Component, Debug, Clone, PartialEq)]
pub struct TimelineNativeClipVisual {
    pub node_id: NodeId,
    pub track_id: TrackId,
    pub x_px: f32,
    pub y_px: f32,
    pub width_px: f32,
    pub height_px: f32,
    pub start_ms: u64,
    pub end_ms: u64,
}

#[derive(Component, Debug, Clone, PartialEq)]
pub struct TimelineNativePlayheadVisual {
    pub position_ms: u64,
    pub x_px: f32,
    pub height_px: f32,
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

#[derive(Debug, Clone)]
pub struct TimelineNativeWindowControlHandle {
    shutdown_requested: Arc<AtomicBool>,
    show_requested: Arc<AtomicBool>,
    hide_requested: Arc<AtomicBool>,
    visible: Arc<AtomicBool>,
    ready: Arc<AtomicBool>,
    projection_sender: mpsc::SyncSender<TimelineRenderProjection>,
    projection_receiver: Arc<Mutex<mpsc::Receiver<TimelineRenderProjection>>>,
    command_sender: mpsc::SyncSender<TimelineRendererCommand>,
    command_receiver: Arc<Mutex<mpsc::Receiver<TimelineRendererCommand>>>,
}

#[derive(Debug, Clone, Resource)]
pub struct TimelineNativeWindowControl {
    shutdown_requested: Arc<AtomicBool>,
    show_requested: Arc<AtomicBool>,
    hide_requested: Arc<AtomicBool>,
    visible: Arc<AtomicBool>,
    ready: Arc<AtomicBool>,
    projection_receiver: Arc<Mutex<mpsc::Receiver<TimelineRenderProjection>>>,
    command_sender: mpsc::SyncSender<TimelineRendererCommand>,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum TimelineNativeWindowProjectionUpdateError {
    QueueFull,
    WindowClosed,
}

impl Default for TimelineNativeRenderConfig {
    fn default() -> Self {
        Self {
            borderless_window: false,
        }
    }
}

impl TimelineNativeRenderLayout {
    pub fn from_window(width_px: u32, height_px: u32) -> Self {
        Self {
            width_px: width_px.max(1) as f32,
            height_px: height_px.max(1) as f32,
            clip_height_px: TIMELINE_NATIVE_CLIP_HEIGHT_PX,
            track_gap_px: TIMELINE_NATIVE_TRACK_GAP_PX,
            horizontal_padding_px: TIMELINE_NATIVE_HORIZONTAL_PADDING_PX,
            top_padding_px: TIMELINE_NATIVE_TOP_PADDING_PX,
        }
    }

    fn usable_width_px(self) -> f32 {
        (self.width_px - (self.horizontal_padding_px * 2.0)).max(1.0)
    }

    fn left_px(self) -> f32 {
        -(self.width_px / 2.0) + self.horizontal_padding_px
    }

    fn top_px(self) -> f32 {
        (self.height_px / 2.0) - self.top_padding_px
    }
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

impl Default for TimelineNativeWindowControlHandle {
    fn default() -> Self {
        Self::new()
    }
}

impl TimelineNativeWindowControlHandle {
    pub fn new() -> Self {
        let (projection_sender, projection_receiver) =
            mpsc::sync_channel(TIMELINE_NATIVE_PROJECTION_QUEUE_CAPACITY);
        let (command_sender, command_receiver) =
            mpsc::sync_channel(TIMELINE_NATIVE_COMMAND_QUEUE_CAPACITY);
        Self {
            shutdown_requested: Arc::new(AtomicBool::new(false)),
            show_requested: Arc::new(AtomicBool::new(false)),
            hide_requested: Arc::new(AtomicBool::new(false)),
            visible: Arc::new(AtomicBool::new(false)),
            ready: Arc::new(AtomicBool::new(false)),
            projection_sender,
            projection_receiver: Arc::new(Mutex::new(projection_receiver)),
            command_sender,
            command_receiver: Arc::new(Mutex::new(command_receiver)),
        }
    }

    pub fn request_close(&self) {
        self.shutdown_requested.store(true, Ordering::Release);
    }

    pub fn close_requested(&self) -> bool {
        self.shutdown_requested.load(Ordering::Acquire)
    }

    pub fn request_show(&self) {
        self.show_requested.store(true, Ordering::Release);
    }

    pub fn request_hide(&self) {
        self.hide_requested.store(true, Ordering::Release);
    }

    pub fn visible(&self) -> bool {
        self.visible.load(Ordering::Acquire)
    }

    pub fn mark_visible(&self, visible: bool) {
        self.visible.store(visible, Ordering::Release);
    }

    pub fn mark_ready(&self) {
        self.ready.store(true, Ordering::Release);
    }

    pub fn ready(&self) -> bool {
        self.ready.load(Ordering::Acquire)
    }

    pub fn request_projection_update(
        &self,
        projection: TimelineRenderProjection,
    ) -> Result<(), TimelineNativeWindowProjectionUpdateError> {
        match self.projection_sender.try_send(projection) {
            Ok(()) => Ok(()),
            Err(mpsc::TrySendError::Full(_)) => {
                Err(TimelineNativeWindowProjectionUpdateError::QueueFull)
            }
            Err(mpsc::TrySendError::Disconnected(_)) => {
                Err(TimelineNativeWindowProjectionUpdateError::WindowClosed)
            }
        }
    }

    pub fn drain_commands(&self) -> Vec<TimelineRendererCommand> {
        let Ok(receiver) = self.command_receiver.lock() else {
            return Vec::new();
        };
        let mut commands = Vec::new();
        while let Ok(command) = receiver.try_recv() {
            commands.push(command);
        }
        commands
    }
}

impl From<&TimelineNativeWindowControlHandle> for TimelineNativeWindowControl {
    fn from(handle: &TimelineNativeWindowControlHandle) -> Self {
        Self {
            shutdown_requested: Arc::clone(&handle.shutdown_requested),
            show_requested: Arc::clone(&handle.show_requested),
            hide_requested: Arc::clone(&handle.hide_requested),
            visible: Arc::clone(&handle.visible),
            ready: Arc::clone(&handle.ready),
            projection_receiver: Arc::clone(&handle.projection_receiver),
            command_sender: handle.command_sender.clone(),
        }
    }
}

impl TimelineNativeWindowControl {
    pub fn request_close_from_os_window(&self) {
        self.shutdown_requested.store(true, Ordering::Release);
        self.visible.store(false, Ordering::Release);
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
        app.add_systems(Startup, spawn_timeline_native_camera);
        app.add_systems(Update, mark_timeline_native_window_ready);
        app.add_systems(Update, apply_timeline_native_projection_updates);
        app.add_systems(Update, emit_timeline_native_click_selection);
        app.add_systems(Update, navigate_timeline_native_viewport);
        app.add_systems(Update, navigate_timeline_native_playhead);
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

fn spawn_timeline_native_camera(mut commands: Commands) {
    commands.spawn((Camera2d, TimelineNativeCamera));
}

fn emit_timeline_native_click_selection(
    buttons: Option<Res<ButtonInput<MouseButton>>>,
    windows: Query<&Window, bevy::prelude::With<PrimaryWindow>>,
    control: Option<Res<TimelineNativeWindowControl>>,
    projection_state: Res<TimelineNativeProjectionState>,
) {
    let Some(buttons) = buttons else {
        return;
    };
    if !buttons.just_pressed(MouseButton::Left) {
        return;
    }
    let Some(control) = control else {
        return;
    };
    let Ok(window) = windows.single() else {
        return;
    };
    let Some(cursor_position) = window.cursor_position() else {
        return;
    };
    let Some(projection) = projection_state.projection.as_ref() else {
        return;
    };
    let geometry = TimelineViewportGeometry::new(
        window.width().max(1.0) as u32,
        window.height().max(1.0) as u32,
        native_track_height_px() as u32,
    );
    let point = TimelineViewportPoint::new(
        cursor_position.x.max(0.0) as u32,
        (window.height() - cursor_position.y).max(0.0) as u32,
    );
    let _ = emit_timeline_native_clip_selection(
        &control,
        projection,
        projection_state.viewport,
        geometry,
        point,
    );
}

fn navigate_timeline_native_viewport(world: &mut World) {
    let Some(keys) = world.get_resource::<ButtonInput<KeyCode>>() else {
        return;
    };
    let pan_left = keys.just_pressed(KeyCode::KeyA) || keys.just_pressed(KeyCode::ArrowLeft);
    let pan_right = keys.just_pressed(KeyCode::KeyD) || keys.just_pressed(KeyCode::ArrowRight);
    let zoom_out = keys.just_pressed(KeyCode::KeyQ) || keys.just_pressed(KeyCode::Minus);
    let zoom_in = keys.just_pressed(KeyCode::KeyE) || keys.just_pressed(KeyCode::Equal);

    let viewport_width_ms = world
        .get_resource::<TimelineNativeProjectionState>()
        .map(|state| state.viewport.width_ms())
        .unwrap_or(1);
    let pan_step_ms = (viewport_width_ms / 10).max(1) as i64;

    if pan_left {
        let _ = pan_timeline_native_viewport(world, -pan_step_ms);
    }
    if pan_right {
        let _ = pan_timeline_native_viewport(world, pan_step_ms);
    }
    if zoom_out {
        let _ = zoom_timeline_native_viewport(world, 0.8);
    }
    if zoom_in {
        let _ = zoom_timeline_native_viewport(world, 1.25);
    }
}

fn navigate_timeline_native_playhead(world: &mut World) {
    let Some(keys) = world.get_resource::<ButtonInput<KeyCode>>() else {
        return;
    };
    let nudge_left = keys.just_pressed(KeyCode::KeyJ);
    let nudge_right = keys.just_pressed(KeyCode::KeyL);

    if !nudge_left && !nudge_right {
        return;
    }

    let viewport_width_ms = world
        .get_resource::<TimelineNativeProjectionState>()
        .map(|state| state.viewport.width_ms())
        .unwrap_or(1);
    let nudge_step_ms = (viewport_width_ms / 100).max(1) as i64;

    if nudge_left {
        nudge_timeline_native_playhead(world, -nudge_step_ms);
    }
    if nudge_right {
        nudge_timeline_native_playhead(world, nudge_step_ms);
    }
}

pub(crate) fn emit_timeline_native_clip_selection(
    control: &TimelineNativeWindowControl,
    projection: &TimelineRenderProjection,
    viewport: TimelineViewport,
    geometry: TimelineViewportGeometry,
    point: TimelineViewportPoint,
) -> Result<Option<NodeId>, TimelineRendererError> {
    let node_id = hit_test_projection_clip_at_point(projection, viewport, geometry, point)?;
    if let Some(node_id) = node_id {
        let _ = control
            .command_sender
            .try_send(TimelineRendererCommand::SelectNode { node_id });
    }
    Ok(node_id)
}

pub fn emit_timeline_native_node_range_request(
    control: &TimelineNativeWindowControl,
    projection: &TimelineRenderProjection,
    node_id: NodeId,
    start_ms: u64,
    end_ms: u64,
) -> Result<(), TimelineRendererError> {
    if !projection.clips.iter().any(|clip| clip.node_id == node_id) {
        return Err(TimelineRendererError::UnknownNode { node_id });
    }
    if start_ms >= end_ms || end_ms > projection.total_duration_ms {
        return Err(TimelineRendererError::InvalidNodeRange {
            start_ms,
            end_ms,
            duration_ms: projection.total_duration_ms,
        });
    }

    let _ = control
        .command_sender
        .try_send(TimelineRendererCommand::SetNodeRange {
            node_id,
            start_ms,
            end_ms,
        });
    Ok(())
}

pub fn emit_timeline_native_delete_node_request(
    control: &TimelineNativeWindowControl,
    projection: &TimelineRenderProjection,
    node_id: NodeId,
) -> Result<(), TimelineRendererError> {
    if !projection.clips.iter().any(|clip| clip.node_id == node_id) {
        return Err(TimelineRendererError::UnknownNode { node_id });
    }

    let _ = control
        .command_sender
        .try_send(TimelineRendererCommand::DeleteNode { node_id });
    Ok(())
}

pub fn emit_timeline_native_split_node_request(
    control: &TimelineNativeWindowControl,
    projection: &TimelineRenderProjection,
    node_id: NodeId,
    at_ms: u64,
    left_node_id: NodeId,
    right_node_id: NodeId,
) -> Result<(), TimelineRendererError> {
    let Some(clip) = projection.clips.iter().find(|clip| clip.node_id == node_id) else {
        return Err(TimelineRendererError::UnknownNode { node_id });
    };
    if at_ms <= clip.start_ms || at_ms >= clip.end_ms {
        return Err(TimelineRendererError::InvalidNodeSplit {
            at_ms,
            start_ms: clip.start_ms,
            end_ms: clip.end_ms,
        });
    }
    let output_ids_are_available = left_node_id != right_node_id
        && !projection
            .clips
            .iter()
            .any(|clip| clip.node_id == left_node_id || clip.node_id == right_node_id);
    if !output_ids_are_available {
        return Err(TimelineRendererError::InvalidSplitOutputNodeIds {
            left_node_id,
            right_node_id,
        });
    }

    let _ = control
        .command_sender
        .try_send(TimelineRendererCommand::SplitNode {
            node_id,
            at_ms,
            left_node_id,
            right_node_id,
        });
    Ok(())
}

pub fn emit_timeline_native_create_node_request(
    control: &TimelineNativeWindowControl,
    projection: &TimelineRenderProjection,
    node_id: NodeId,
    parent_id: Option<NodeId>,
    level: StoryLevel,
    name: String,
    start_ms: u64,
    end_ms: u64,
    beat_type: Option<BeatType>,
) -> Result<(), TimelineRendererError> {
    if let Some(parent_id) = parent_id
        && !projection
            .clips
            .iter()
            .any(|clip| clip.node_id == parent_id)
    {
        return Err(TimelineRendererError::UnknownNode { node_id: parent_id });
    }
    if start_ms >= end_ms || end_ms > projection.total_duration_ms {
        return Err(TimelineRendererError::InvalidNodeRange {
            start_ms,
            end_ms,
            duration_ms: projection.total_duration_ms,
        });
    }

    let _ = control
        .command_sender
        .try_send(TimelineRendererCommand::CreateNode {
            node_id,
            parent_id,
            level,
            name,
            start_ms,
            end_ms,
            beat_type,
        });
    Ok(())
}

fn native_track_height_px() -> u32 {
    (TIMELINE_NATIVE_CLIP_HEIGHT_PX + TIMELINE_NATIVE_TRACK_GAP_PX) as u32
}

pub(crate) fn rebuild_timeline_native_visuals(
    world: &mut World,
    projection: &TimelineRenderProjection,
) {
    despawn_existing_timeline_native_visuals(world);

    let layout = world
        .get_resource::<TimelineNativeRenderLayout>()
        .copied()
        .unwrap_or_else(|| TimelineNativeRenderLayout::from_window(1280, 360));
    let viewport = world
        .get_resource::<TimelineNativeProjectionState>()
        .map(|state| state.viewport)
        .unwrap_or_else(|| TimelineViewport::from_duration(projection.total_duration_ms));
    let viewport_width_ms = viewport.width_ms();
    let sorted_tracks = {
        let mut tracks = projection.tracks.clone();
        tracks.sort_by_key(|track| track.sort_order);
        tracks
    };

    for clip in &projection.clips {
        let Some(track_index) = sorted_tracks
            .iter()
            .position(|track| track.track_id == clip.track_id)
        else {
            continue;
        };
        let visible_start_ms = clip.start_ms.max(viewport.start_ms);
        let visible_end_ms = clip.end_ms.min(viewport.end_ms);
        if visible_end_ms <= visible_start_ms {
            continue;
        }
        let start_ratio =
            visible_start_ms.saturating_sub(viewport.start_ms) as f32 / viewport_width_ms as f32;
        let end_ratio =
            visible_end_ms.saturating_sub(viewport.start_ms) as f32 / viewport_width_ms as f32;
        if end_ratio <= start_ratio {
            continue;
        }
        let x_start = layout.left_px() + (start_ratio * layout.usable_width_px());
        let x_end = layout.left_px() + (end_ratio * layout.usable_width_px());
        let width_px = (x_end - x_start).max(1.0);
        let x_px = x_start + (width_px / 2.0);
        let y_px =
            layout.top_px() - (track_index as f32 * (layout.clip_height_px + layout.track_gap_px));

        world.spawn((
            TimelineSceneEntity,
            TimelineNativeVisualEntity,
            TimelineNativeClipVisual {
                node_id: clip.node_id,
                track_id: clip.track_id,
                x_px,
                y_px,
                width_px,
                height_px: layout.clip_height_px,
                start_ms: clip.start_ms,
                end_ms: clip.end_ms,
            },
            Sprite::from_color(
                Color::srgb(0.342, 0.655, 0.691),
                Vec2::new(width_px, layout.clip_height_px),
            ),
            Transform::from_translation(Vec3::new(x_px, y_px, 0.0)),
        ));
    }

    if let Some(playhead) = world
        .get_resource::<TimelineNativeProjectionState>()
        .map(|state| state.playhead)
    {
        spawn_timeline_native_playhead_visual(world, layout, viewport, playhead);
    }
}

fn spawn_timeline_native_playhead_visual(
    world: &mut World,
    layout: TimelineNativeRenderLayout,
    viewport: TimelineViewport,
    playhead: TimelinePlayhead,
) {
    if playhead.position_ms < viewport.start_ms || playhead.position_ms > viewport.end_ms {
        return;
    }

    let viewport_width_ms = viewport.width_ms();
    let position_ratio =
        playhead.position_ms.saturating_sub(viewport.start_ms) as f32 / viewport_width_ms as f32;
    let x_px = layout.left_px() + (position_ratio * layout.usable_width_px());
    let height_px = (layout.height_px - (layout.top_padding_px * 2.0)).max(layout.clip_height_px);
    let y_px = layout.top_px() - (height_px / 2.0) + (layout.clip_height_px / 2.0);

    world.spawn((
        TimelineSceneEntity,
        TimelineNativeVisualEntity,
        TimelineNativePlayheadVisual {
            position_ms: playhead.position_ms,
            x_px,
            height_px,
        },
        Sprite::from_color(
            Color::srgb(0.937, 0.267, 0.267),
            Vec2::new(TIMELINE_NATIVE_PLAYHEAD_WIDTH_PX, height_px),
        ),
        Transform::from_translation(Vec3::new(x_px, y_px, 1.0)),
    ));
}

fn despawn_existing_timeline_native_visuals(world: &mut World) {
    let entities: Vec<Entity> = world
        .query_filtered::<Entity, bevy::prelude::With<TimelineNativeVisualEntity>>()
        .iter(world)
        .collect();

    for entity in entities {
        let _ = world.despawn(entity);
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
