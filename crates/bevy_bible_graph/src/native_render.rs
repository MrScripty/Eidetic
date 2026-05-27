use std::collections::HashMap;
use std::num::NonZeroU64;
use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::{Arc, Mutex};
use std::time::{Duration, Instant};

use bevy::app::AppExit;
use bevy::input::mouse::{MouseMotion, MouseWheel};
use bevy::prelude::{
    App, Assets, ButtonInput, Camera, Camera3d, ClearColor, Color, Commands, Component, Cuboid,
    DefaultPlugins, Entity, GlobalTransform, Handle, Justify, KeyCode, Mesh, Mesh3d,
    MeshMaterial3d, MessageReader, MessageWriter, MouseButton, Plugin, PluginGroup, PointLight,
    Quat, Query, Ray3d, Res, ResMut, Resource, Sphere, StandardMaterial, Startup, Text2d,
    TextColor, TextFont, TextLayout, Time, Transform, Update, Vec2, Vec3, Visibility, Window, With,
    Without, World,
};
use bevy::window::{
    ExitCondition, PrimaryWindow, WindowCloseRequested, WindowPlugin, WindowResolution,
};
use bevy::winit::WinitPlugin;
use eidetic_core::contracts::{
    BibleGraphEdgeId, BibleGraphNodeId, BibleRenderGraphProjection, ContextInfluenceId,
};

use crate::{
    BIBLE_GRAPH_RENDERER_COMMAND_QUEUE_CAPACITY, BibleGraphCameraCommand,
    BibleGraphRendererCommand, BibleGraphRendererError, BibleGraphVisual3dEdgeClass,
    build_bible_graph_visual_3d_snapshot,
};

#[derive(Debug, Clone, Copy, PartialEq, Eq, Resource)]
pub struct BibleGraphNativeRenderConfig {
    pub borderless_window: bool,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct BibleGraphNativeWindowRunnerConfig {
    pub title: String,
    pub width_px: u32,
    pub height_px: u32,
    pub borderless_window: bool,
    pub run_on_any_thread: bool,
    pub auto_close_after_ms: Option<NonZeroU64>,
}

#[derive(Debug, Clone, PartialEq, Eq, Resource)]
pub struct BibleGraphNativeRendererWindowScene {
    pub background_color: &'static str,
    pub grid_color: &'static str,
    pub accent_color: &'static str,
}

#[derive(Debug, Resource)]
struct BibleGraphNativeAutoClose {
    close_at: Instant,
}

#[derive(Debug, Clone)]
pub struct BibleGraphNativeWindowControlHandle {
    shutdown_requested: Arc<AtomicBool>,
    show_requested: Arc<AtomicBool>,
    hide_requested: Arc<AtomicBool>,
    visible: Arc<AtomicBool>,
    ready: Arc<AtomicBool>,
    projection: Arc<Mutex<Option<BibleRenderGraphProjection>>>,
    native_visual_counts: Arc<Mutex<BibleGraphNativeVisualStatus>>,
    commands: Arc<Mutex<Vec<BibleGraphRendererCommand>>>,
    camera_commands: Arc<Mutex<Vec<BibleGraphCameraCommand>>>,
}

#[derive(Debug, Clone, Resource)]
pub struct BibleGraphNativeWindowControl {
    shutdown_requested: Arc<AtomicBool>,
    show_requested: Arc<AtomicBool>,
    hide_requested: Arc<AtomicBool>,
    visible: Arc<AtomicBool>,
    ready: Arc<AtomicBool>,
    projection: Arc<Mutex<Option<BibleRenderGraphProjection>>>,
    native_visual_counts: Arc<Mutex<BibleGraphNativeVisualStatus>>,
    commands: Arc<Mutex<Vec<BibleGraphRendererCommand>>>,
    camera_commands: Arc<Mutex<Vec<BibleGraphCameraCommand>>>,
}

#[derive(Debug, Clone, Copy, Default, PartialEq, Eq, Resource)]
pub struct BibleGraphNativeRendererWindowStatus {
    pub camera_count: usize,
    pub bounds: BibleGraphNativeRendererWindowBounds,
}

#[derive(Debug, Clone, Copy, Default, PartialEq, Eq, Resource)]
pub struct BibleGraphNativeVisualStatus {
    pub node_count: usize,
    pub edge_count: usize,
}

#[derive(Debug, Default, Resource)]
struct BibleGraphNativeAssetCache {
    node_meshes: HashMap<u32, Handle<Mesh>>,
    edge_meshes: HashMap<(u32, u32), Handle<Mesh>>,
    materials: HashMap<BibleGraphNativeMaterialKey, Handle<StandardMaterial>>,
}

#[derive(Debug, Clone, PartialEq, Eq, Hash)]
struct BibleGraphNativeMaterialKey {
    color: &'static str,
    selected: bool,
    highlighted: bool,
    dimmed: bool,
}

#[derive(Debug, Clone, Copy, Default, PartialEq, Eq)]
pub struct BibleGraphNativeRendererWindowBounds {
    pub width_px: u32,
    pub height_px: u32,
}

impl Default for BibleGraphNativeRendererWindowScene {
    fn default() -> Self {
        Self {
            background_color: "#11151d",
            grid_color: "#253041",
            accent_color: "#f2c94c",
        }
    }
}

#[derive(Component)]
pub struct BibleGraphNativeCamera;

#[derive(Component, Debug, Clone, PartialEq)]
pub struct BibleGraphNativeVisualEntity;

#[derive(Component, Debug, Clone, PartialEq)]
pub struct BibleGraphNativeNodeVisual {
    pub node_id: BibleGraphNodeId,
    pub x: f32,
    pub y: f32,
    pub z: f32,
    pub radius: f32,
    pub fill_color: &'static str,
    pub outline_color: &'static str,
    pub selected: bool,
    pub highlighted: bool,
    pub dimmed: bool,
    pub label_visible: bool,
}

#[derive(Component, Debug, Clone, PartialEq)]
pub struct BibleGraphNativeNodeLabelVisual {
    pub node_id: BibleGraphNodeId,
    pub label: String,
}

#[derive(Component, Debug, Clone, PartialEq)]
pub struct BibleGraphNativeLabelBillboard;

#[derive(Component, Debug, Clone, PartialEq)]
pub struct BibleGraphNativeEdgeVisual {
    pub edge_id: BibleGraphEdgeId,
    pub edge_class: BibleGraphVisual3dEdgeClass,
    pub from_node_id: BibleGraphNodeId,
    pub to_node_id: BibleGraphNodeId,
    pub from_x: f32,
    pub from_y: f32,
    pub from_z: f32,
    pub to_x: f32,
    pub to_y: f32,
    pub to_z: f32,
    pub width: f32,
    pub stroke_color: &'static str,
    pub selected: bool,
    pub highlighted: bool,
    pub dimmed: bool,
}

#[derive(Component, Debug, Clone, PartialEq)]
pub struct BibleGraphNativeInfluenceVisual {
    pub influence_id: ContextInfluenceId,
    pub bible_node_id: Option<BibleGraphNodeId>,
    pub bible_edge_id: Option<BibleGraphEdgeId>,
}

impl Default for BibleGraphNativeRenderConfig {
    fn default() -> Self {
        Self {
            borderless_window: false,
        }
    }
}

impl BibleGraphNativeWindowRunnerConfig {
    pub fn minimal_smoke(run_on_any_thread: bool) -> Self {
        Self {
            title: "Eidetic Bible Graph".to_string(),
            width_px: 1280,
            height_px: 720,
            borderless_window: false,
            run_on_any_thread,
            auto_close_after_ms: None,
        }
    }

    pub fn with_auto_close_after_ms(mut self, auto_close_after_ms: NonZeroU64) -> Self {
        self.auto_close_after_ms = Some(auto_close_after_ms);
        self
    }
}

impl Default for BibleGraphNativeWindowControlHandle {
    fn default() -> Self {
        Self::new()
    }
}

impl BibleGraphNativeWindowControlHandle {
    pub fn new() -> Self {
        Self {
            shutdown_requested: Arc::new(AtomicBool::new(false)),
            show_requested: Arc::new(AtomicBool::new(false)),
            hide_requested: Arc::new(AtomicBool::new(false)),
            visible: Arc::new(AtomicBool::new(false)),
            ready: Arc::new(AtomicBool::new(false)),
            projection: Arc::new(Mutex::new(None)),
            native_visual_counts: Arc::new(Mutex::new(BibleGraphNativeVisualStatus::default())),
            commands: Arc::new(Mutex::new(Vec::new())),
            camera_commands: Arc::new(Mutex::new(Vec::new())),
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

    pub fn set_projection(&self, projection: BibleRenderGraphProjection) {
        *self
            .projection
            .lock()
            .unwrap_or_else(|error| error.into_inner()) = Some(projection);
    }

    pub fn native_visual_counts(&self) -> BibleGraphNativeVisualStatus {
        *self
            .native_visual_counts
            .lock()
            .unwrap_or_else(|error| error.into_inner())
    }

    pub fn drain_commands(&self) -> Vec<BibleGraphRendererCommand> {
        std::mem::take(
            &mut self
                .commands
                .lock()
                .unwrap_or_else(|error| error.into_inner()),
        )
    }

    pub fn push_camera_command(
        &self,
        command: BibleGraphCameraCommand,
    ) -> Result<(), BibleGraphRendererError> {
        let mut commands = self
            .camera_commands
            .lock()
            .unwrap_or_else(|error| error.into_inner());
        if commands.len() >= BIBLE_GRAPH_RENDERER_COMMAND_QUEUE_CAPACITY {
            return Err(BibleGraphRendererError::CommandQueueFull);
        }
        commands.push(command);
        Ok(())
    }

    pub fn drain_camera_commands(&self) -> Vec<BibleGraphCameraCommand> {
        std::mem::take(
            &mut self
                .camera_commands
                .lock()
                .unwrap_or_else(|error| error.into_inner()),
        )
    }
}

impl From<&BibleGraphNativeWindowControlHandle> for BibleGraphNativeWindowControl {
    fn from(handle: &BibleGraphNativeWindowControlHandle) -> Self {
        Self {
            shutdown_requested: Arc::clone(&handle.shutdown_requested),
            show_requested: Arc::clone(&handle.show_requested),
            hide_requested: Arc::clone(&handle.hide_requested),
            visible: Arc::clone(&handle.visible),
            ready: Arc::clone(&handle.ready),
            projection: Arc::clone(&handle.projection),
            native_visual_counts: Arc::clone(&handle.native_visual_counts),
            commands: Arc::clone(&handle.commands),
            camera_commands: Arc::clone(&handle.camera_commands),
        }
    }
}

impl BibleGraphNativeWindowControl {
    pub fn request_close_from_os_window(&self) {
        self.shutdown_requested.store(true, Ordering::Release);
        self.visible.store(false, Ordering::Release);
    }

    fn drain_camera_commands(&self) -> Vec<BibleGraphCameraCommand> {
        std::mem::take(
            &mut self
                .camera_commands
                .lock()
                .unwrap_or_else(|error| error.into_inner()),
        )
    }
}

pub struct BibleGraphNativeRenderPlugin;

impl Plugin for BibleGraphNativeRenderPlugin {
    fn build(&self, app: &mut App) {
        app.init_resource::<Assets<Mesh>>();
        app.init_resource::<Assets<StandardMaterial>>();
        app.insert_resource(BibleGraphNativeRenderConfig::default());
        app.insert_resource(BibleGraphNativeRendererWindowScene::default());
        app.insert_resource(BibleGraphNativeRendererWindowStatus::default());
        app.insert_resource(BibleGraphNativeVisualStatus::default());
        app.insert_resource(BibleGraphNativeAssetCache::default());
        app.insert_resource(ClearColor(Color::srgb(0.067, 0.082, 0.114)));
        app.add_systems(Startup, spawn_bible_graph_renderer_window_scene);
        app.add_systems(Startup, mark_bible_graph_native_window_ready);
        app.add_systems(Update, apply_bible_graph_native_projection);
        app.add_systems(Update, emit_bible_graph_native_click_selection);
        app.add_systems(Update, emit_bible_graph_native_keyboard_commands);
        app.add_systems(Update, billboard_bible_graph_native_labels);
        app.add_systems(Update, navigate_bible_graph_native_camera);
        app.add_systems(Update, drag_bible_graph_native_camera);
        app.add_systems(Update, scroll_bible_graph_native_camera);
        app.add_systems(Update, orbit_bible_graph_native_camera);
        app.add_systems(Update, recover_bible_graph_native_camera);
        app.add_systems(Update, frame_bible_graph_native_camera_on_selected);
        app.add_systems(Update, apply_bible_graph_native_camera_commands);
    }
}

pub fn configure_minimal_bible_graph_native_window_app(
    app: &mut App,
    config: BibleGraphNativeWindowRunnerConfig,
) {
    configure_controlled_minimal_bible_graph_native_window_app(
        app,
        config,
        BibleGraphNativeWindowControlHandle::new(),
    );
}

pub fn configure_controlled_minimal_bible_graph_native_window_app(
    app: &mut App,
    config: BibleGraphNativeWindowRunnerConfig,
    control_handle: BibleGraphNativeWindowControlHandle,
) {
    app.add_message::<AppExit>();
    app.add_message::<WindowCloseRequested>();
    app.add_plugins(
        DefaultPlugins
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
    app.add_plugins(BibleGraphNativeRenderPlugin);
    app.insert_resource(BibleGraphNativeWindowControl::from(&control_handle));
    app.add_systems(Update, close_minimal_native_window_when_requested);
    app.add_systems(Update, close_minimal_native_window_from_os_request);
    app.add_systems(Update, apply_minimal_native_window_visibility_requests);

    if let Some(auto_close_after_ms) = config.auto_close_after_ms {
        app.insert_resource(BibleGraphNativeAutoClose {
            close_at: Instant::now() + Duration::from_millis(auto_close_after_ms.get()),
        });
        app.add_systems(Update, close_minimal_native_window_after_timer);
    }
}

pub fn run_minimal_bible_graph_native_window(config: BibleGraphNativeWindowRunnerConfig) {
    run_controlled_minimal_bible_graph_native_window(
        config,
        BibleGraphNativeWindowControlHandle::new(),
    );
}

pub fn run_controlled_minimal_bible_graph_native_window(
    config: BibleGraphNativeWindowRunnerConfig,
    control_handle: BibleGraphNativeWindowControlHandle,
) {
    let mut app = App::new();
    configure_controlled_minimal_bible_graph_native_window_app(&mut app, config, control_handle);
    app.run();
}

fn spawn_bible_graph_renderer_window_scene(
    mut commands: Commands,
    mut status: ResMut<BibleGraphNativeRendererWindowStatus>,
) {
    commands.spawn((
        Camera3d::default(),
        Transform::from_xyz(0.0, 0.0, 900.0).looking_at(Vec3::ZERO, Vec3::Y),
        BibleGraphNativeCamera,
    ));
    commands.spawn((PointLight::default(), Transform::from_xyz(0.0, 0.0, 700.0)));
    status.camera_count = 1;
}

fn mark_bible_graph_native_window_ready(control: Option<Res<BibleGraphNativeWindowControl>>) {
    let Some(control) = control else {
        return;
    };
    control.ready.store(true, Ordering::Release);
    control.visible.store(true, Ordering::Release);
}

fn close_minimal_native_window_after_timer(
    auto_close: Res<BibleGraphNativeAutoClose>,
    mut app_exit: MessageWriter<AppExit>,
) {
    if Instant::now() >= auto_close.close_at {
        app_exit.write(AppExit::Success);
    }
}

fn close_minimal_native_window_when_requested(
    control: Res<BibleGraphNativeWindowControl>,
    mut app_exit: MessageWriter<AppExit>,
) {
    if control.shutdown_requested.load(Ordering::Acquire) {
        app_exit.write(AppExit::Success);
    }
}

fn close_minimal_native_window_from_os_request(
    control: Res<BibleGraphNativeWindowControl>,
    close_requests: Option<bevy::prelude::MessageReader<WindowCloseRequested>>,
    mut app_exit: MessageWriter<AppExit>,
) {
    let Some(mut close_requests) = close_requests else {
        return;
    };
    if close_requests.read().next().is_some() {
        control.request_close_from_os_window();
        app_exit.write(AppExit::Success);
    }
}

fn apply_minimal_native_window_visibility_requests(
    control: Res<BibleGraphNativeWindowControl>,
    mut windows: bevy::prelude::Query<&mut Window>,
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

fn emit_bible_graph_native_click_selection(
    buttons: Option<Res<ButtonInput<MouseButton>>>,
    windows: Query<&Window, With<PrimaryWindow>>,
    cameras: Query<(&Camera, &GlobalTransform), With<BibleGraphNativeCamera>>,
    nodes: Query<&BibleGraphNativeNodeVisual>,
    edges: Query<&BibleGraphNativeEdgeVisual>,
    control: Option<Res<BibleGraphNativeWindowControl>>,
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
    let Ok((camera, camera_transform)) = cameras.single() else {
        return;
    };
    let Ok(ray) = camera.viewport_to_world(camera_transform, cursor_position) else {
        return;
    };

    if let Some(node_id) = nearest_native_node_on_ray(nodes.iter(), ray) {
        let _ = push_native_command_to_control(
            &control,
            BibleGraphRendererCommand::SelectNode { node_id },
        );
        return;
    }

    if let Some(edge_id) = nearest_selectable_native_edge_on_ray(edges.iter(), ray) {
        let _ = push_native_command_to_control(
            &control,
            BibleGraphRendererCommand::SelectEdge { edge_id },
        );
    }
}

fn emit_bible_graph_native_keyboard_commands(
    keys: Option<Res<ButtonInput<KeyCode>>>,
    nodes: Query<&BibleGraphNativeNodeVisual>,
    control: Option<Res<BibleGraphNativeWindowControl>>,
) {
    let Some(keys) = keys else {
        return;
    };
    let Some(control) = control else {
        return;
    };

    if keys.just_pressed(KeyCode::Escape) {
        let _ = push_native_command_to_control(&control, BibleGraphRendererCommand::ClearSelection);
    }
    if keys.just_pressed(KeyCode::Delete) || keys.just_pressed(KeyCode::Backspace) {
        if let Some(selected_node) = nodes.iter().find(|node| node.selected) {
            let _ = push_native_command_to_control(
                &control,
                BibleGraphRendererCommand::DeleteNode {
                    node_id: selected_node.node_id.clone(),
                },
            );
        }
    }
    if keys.just_pressed(KeyCode::Insert) {
        if let Some(selected_node) = nodes.iter().find(|node| node.selected) {
            let _ = push_native_command_to_control(
                &control,
                BibleGraphRendererCommand::CreateConnectedNode {
                    parent_id: selected_node.node_id.clone(),
                },
            );
        }
    }
}

fn navigate_bible_graph_native_camera(
    keys: Option<Res<ButtonInput<KeyCode>>>,
    time: Option<Res<Time>>,
    mut cameras: Query<&mut Transform, With<BibleGraphNativeCamera>>,
) {
    let Some(keys) = keys else {
        return;
    };
    let Some(time) = time else {
        return;
    };
    let Ok(mut camera_transform) = cameras.single_mut() else {
        return;
    };
    let translation_delta = native_camera_navigation_delta(
        keys.pressed(KeyCode::KeyA) || keys.pressed(KeyCode::ArrowLeft),
        keys.pressed(KeyCode::KeyD) || keys.pressed(KeyCode::ArrowRight),
        keys.pressed(KeyCode::KeyW) || keys.pressed(KeyCode::ArrowUp),
        keys.pressed(KeyCode::KeyS) || keys.pressed(KeyCode::ArrowDown),
        keys.pressed(KeyCode::KeyQ) || keys.pressed(KeyCode::Minus),
        keys.pressed(KeyCode::KeyE) || keys.pressed(KeyCode::Equal),
        time.delta_secs(),
    );

    if translation_delta == Vec3::ZERO {
        return;
    }

    camera_transform.translation += translation_delta;
    camera_transform.translation.z = camera_transform.translation.z.max(120.0);
}

fn frame_bible_graph_native_camera_on_selected(
    keys: Option<Res<ButtonInput<KeyCode>>>,
    nodes: Query<&BibleGraphNativeNodeVisual>,
    mut cameras: Query<&mut Transform, With<BibleGraphNativeCamera>>,
) {
    let Some(keys) = keys else {
        return;
    };
    if !keys.just_pressed(KeyCode::KeyF) && !keys.just_pressed(KeyCode::Period) {
        return;
    }
    let Some(selected_node) = nodes.iter().find(|node| node.selected) else {
        return;
    };
    let Ok(mut camera_transform) = cameras.single_mut() else {
        return;
    };

    camera_transform.translation =
        native_camera_frame_selected_translation(camera_transform.translation, selected_node);
}

fn drag_bible_graph_native_camera(
    keys: Option<Res<ButtonInput<KeyCode>>>,
    buttons: Option<Res<ButtonInput<MouseButton>>>,
    mut motion_events: Option<MessageReader<MouseMotion>>,
    nodes: Query<&BibleGraphNativeNodeVisual>,
    mut cameras: Query<&mut Transform, With<BibleGraphNativeCamera>>,
) {
    let Some(buttons) = buttons else {
        return;
    };
    let Some(mut motion_events) = motion_events.take() else {
        return;
    };
    if !buttons.pressed(MouseButton::Middle) {
        motion_events.clear();
        return;
    }
    let total_delta = motion_events
        .read()
        .fold(Vec2::ZERO, |total, event| total + event.delta);
    if total_delta == Vec2::ZERO {
        return;
    }
    let Ok(mut camera_transform) = cameras.single_mut() else {
        return;
    };
    let shift_pressed = keys
        .as_ref()
        .is_some_and(|keys| keys.pressed(KeyCode::ShiftLeft) || keys.pressed(KeyCode::ShiftRight));

    if shift_pressed {
        let pan_delta = native_camera_drag_pan_delta(total_delta, &camera_transform);
        camera_transform.translation += pan_delta;
        return;
    }

    let orbit_target = native_camera_view_orbit_target(&camera_transform).unwrap_or_else(|| {
        nodes
            .iter()
            .find(|node| node.selected)
            .map(|node| Vec3::new(node.x, node.y, node.z))
            .unwrap_or(Vec3::ZERO)
    });
    native_camera_drag_orbit_transform(&mut camera_transform, orbit_target, total_delta);
}

fn scroll_bible_graph_native_camera(
    mut scroll_events: Option<MessageReader<MouseWheel>>,
    mut cameras: Query<&mut Transform, With<BibleGraphNativeCamera>>,
) {
    let Some(mut scroll_events) = scroll_events.take() else {
        return;
    };
    let scroll_y = scroll_events
        .read()
        .fold(0.0, |total, event| total + event.y);
    if scroll_y == 0.0 {
        return;
    }
    let Ok(mut camera_transform) = cameras.single_mut() else {
        return;
    };

    camera_transform.translation.z =
        (camera_transform.translation.z + native_camera_scroll_zoom_delta(scroll_y)).max(120.0);
}

fn recover_bible_graph_native_camera(
    keys: Option<Res<ButtonInput<KeyCode>>>,
    nodes: Query<&BibleGraphNativeNodeVisual>,
    mut cameras: Query<&mut Transform, With<BibleGraphNativeCamera>>,
) {
    let Some(keys) = keys else {
        return;
    };
    let Some(command) = native_camera_recovery_command(
        keys.just_pressed(KeyCode::KeyR),
        keys.just_pressed(KeyCode::Digit0) || keys.just_pressed(KeyCode::Numpad0),
    ) else {
        return;
    };
    let Some(transform) = native_camera_recovery_transform(command, nodes.iter()) else {
        return;
    };
    let Ok(mut camera_transform) = cameras.single_mut() else {
        return;
    };

    *camera_transform = transform;
}

fn apply_bible_graph_native_camera_commands(world: &mut World) {
    let Some(control) = world
        .get_resource::<BibleGraphNativeWindowControl>()
        .cloned()
    else {
        return;
    };

    for command in control.drain_camera_commands() {
        apply_bible_graph_native_camera_command(world, command);
    }
}

pub fn apply_bible_graph_native_camera_command(
    world: &mut World,
    command: BibleGraphCameraCommand,
) {
    let Some(transform) = native_camera_transform_for_command(world, &command) else {
        return;
    };
    let mut cameras = world.query_filtered::<&mut Transform, With<BibleGraphNativeCamera>>();
    let Ok(mut camera_transform) = cameras.single_mut(world) else {
        return;
    };
    *camera_transform = transform;
}

fn native_camera_transform_for_command(
    world: &mut World,
    command: &BibleGraphCameraCommand,
) -> Option<Transform> {
    match command {
        BibleGraphCameraCommand::ResetCamera => Some(native_camera_reset_transform()),
        BibleGraphCameraCommand::FitGraph => {
            let mut nodes = world.query::<&BibleGraphNativeNodeVisual>();
            native_camera_fit_nodes_transform(nodes.iter(world))
        }
        BibleGraphCameraCommand::FrameNode { node_id }
        | BibleGraphCameraCommand::NavigateToNode { node_id }
        | BibleGraphCameraCommand::NavigateToNeighborhood { node_id } => {
            let mut nodes = world.query::<&BibleGraphNativeNodeVisual>();
            let node = nodes.iter(world).find(|node| &node.node_id == node_id)?;
            Some(native_camera_frame_node_transform(node))
        }
        BibleGraphCameraCommand::FrameEdge { edge_id } => {
            let mut edges = world.query::<&BibleGraphNativeEdgeVisual>();
            let edge = edges.iter(world).find(|edge| &edge.edge_id == edge_id)?;
            Some(native_camera_frame_edge_transform(edge))
        }
        BibleGraphCameraCommand::FrameInfluence { influence_id } => {
            let mut influences = world.query::<&BibleGraphNativeInfluenceVisual>();
            let influence = influences
                .iter(world)
                .find(|influence| &influence.influence_id == influence_id)?;
            if let Some(node_id) = &influence.bible_node_id {
                return native_camera_transform_for_command(
                    world,
                    &BibleGraphCameraCommand::FrameNode {
                        node_id: node_id.clone(),
                    },
                );
            }
            let edge_id = influence.bible_edge_id.as_ref()?;
            native_camera_transform_for_command(
                world,
                &BibleGraphCameraCommand::FrameEdge {
                    edge_id: edge_id.clone(),
                },
            )
        }
    }
}

fn orbit_bible_graph_native_camera(
    keys: Option<Res<ButtonInput<KeyCode>>>,
    time: Option<Res<Time>>,
    nodes: Query<&BibleGraphNativeNodeVisual>,
    mut cameras: Query<&mut Transform, With<BibleGraphNativeCamera>>,
) {
    let Some(keys) = keys else {
        return;
    };
    let Some(time) = time else {
        return;
    };
    let orbit_left = keys.pressed(KeyCode::KeyZ);
    let orbit_right = keys.pressed(KeyCode::KeyC);
    if orbit_left == orbit_right {
        return;
    }
    let Ok(mut camera_transform) = cameras.single_mut() else {
        return;
    };
    let orbit_target = nodes
        .iter()
        .find(|node| node.selected)
        .map(|node| Vec3::new(node.x, node.y, node.z))
        .unwrap_or(Vec3::ZERO);
    let direction = if orbit_left { 1.0 } else { -1.0 };
    let angle_delta = direction * 0.85 * time.delta_secs();

    camera_transform.translation =
        native_camera_orbit_translation(camera_transform.translation, orbit_target, angle_delta);
    camera_transform.look_at(orbit_target, Vec3::Y);
}

pub(crate) fn native_camera_navigation_delta(
    pan_left: bool,
    pan_right: bool,
    pan_up: bool,
    pan_down: bool,
    zoom_out: bool,
    zoom_in: bool,
    delta_seconds: f32,
) -> Vec3 {
    let mut pan_direction = Vec2::ZERO;
    let mut zoom_direction = 0.0;

    if pan_left {
        pan_direction.x -= 1.0;
    }
    if pan_right {
        pan_direction.x += 1.0;
    }
    if pan_up {
        pan_direction.y += 1.0;
    }
    if pan_down {
        pan_direction.y -= 1.0;
    }
    if zoom_out {
        zoom_direction += 1.0;
    }
    if zoom_in {
        zoom_direction -= 1.0;
    }

    let pan_speed = 420.0;
    let zoom_speed = 650.0;
    let pan_delta = if pan_direction == Vec2::ZERO {
        Vec2::ZERO
    } else {
        pan_direction.normalize() * pan_speed * delta_seconds
    };

    Vec3::new(
        pan_delta.x,
        pan_delta.y,
        zoom_direction * zoom_speed * delta_seconds,
    )
}

pub(crate) fn native_camera_reset_transform() -> Transform {
    Transform::from_xyz(0.0, 0.0, 900.0).looking_at(Vec3::ZERO, Vec3::Y)
}

pub(crate) fn native_camera_recovery_command(
    reset_pressed: bool,
    fit_pressed: bool,
) -> Option<BibleGraphCameraCommand> {
    if reset_pressed {
        return Some(BibleGraphCameraCommand::ResetCamera);
    }
    if fit_pressed {
        return Some(BibleGraphCameraCommand::FitGraph);
    }
    None
}

fn native_camera_recovery_transform<'a>(
    command: BibleGraphCameraCommand,
    nodes: impl Iterator<Item = &'a BibleGraphNativeNodeVisual>,
) -> Option<Transform> {
    match command {
        BibleGraphCameraCommand::ResetCamera => Some(native_camera_reset_transform()),
        BibleGraphCameraCommand::FitGraph => native_camera_fit_nodes_transform(nodes),
        _ => None,
    }
}

pub(crate) fn native_camera_fit_nodes_transform<'a>(
    nodes: impl Iterator<Item = &'a BibleGraphNativeNodeVisual>,
) -> Option<Transform> {
    let mut min = Vec3::splat(f32::INFINITY);
    let mut max = Vec3::splat(f32::NEG_INFINITY);
    let mut node_count = 0;
    for node in nodes {
        let radius = node.radius.max(1.0);
        let position = Vec3::new(node.x, node.y, node.z);
        min = min.min(position - Vec3::splat(radius));
        max = max.max(position + Vec3::splat(radius));
        node_count += 1;
    }
    if node_count == 0 {
        return Some(native_camera_reset_transform());
    }

    let center = (min + max) * 0.5;
    let extent = (max - min).length().max(220.0);
    Some(native_camera_frame_target_transform(center, extent * 1.6))
}

pub(crate) fn native_camera_frame_node_transform(node: &BibleGraphNativeNodeVisual) -> Transform {
    native_camera_frame_target_transform(
        Vec3::new(node.x, node.y, node.z),
        (node.radius * 8.0).max(220.0),
    )
}

pub(crate) fn native_camera_frame_edge_transform(edge: &BibleGraphNativeEdgeVisual) -> Transform {
    let from = Vec3::new(edge.from_x, edge.from_y, edge.from_z);
    let to = Vec3::new(edge.to_x, edge.to_y, edge.to_z);
    native_camera_frame_target_transform((from + to) * 0.5, from.distance(to).max(220.0) * 1.4)
}

fn native_camera_frame_target_transform(center: Vec3, distance: f32) -> Transform {
    Transform::from_translation(Vec3::new(
        center.x,
        center.y,
        center.z + distance.max(220.0),
    ))
    .looking_at(center, Vec3::Y)
}

pub(crate) fn native_edge_segment_transform(from: Vec3, to: Vec3) -> (f32, Transform) {
    let segment = to - from;
    let segment_length = segment.length();
    let edge_length = segment_length.max(1.0);
    let rotation = if segment_length <= f32::EPSILON {
        Default::default()
    } else {
        bevy::prelude::Quat::from_rotation_arc(Vec3::X, segment / segment_length)
    };

    (
        edge_length,
        Transform {
            translation: from + segment * 0.5,
            rotation,
            ..Default::default()
        },
    )
}

pub(crate) fn native_camera_frame_selected_translation(
    current_translation: Vec3,
    selected_node: &BibleGraphNativeNodeVisual,
) -> Vec3 {
    Vec3::new(
        selected_node.x,
        selected_node.y,
        current_translation.z.max(selected_node.z + 220.0),
    )
}

pub(crate) fn native_camera_drag_pan_delta(
    cursor_delta: Vec2,
    camera_transform: &Transform,
) -> Vec3 {
    let scale = native_camera_drag_pan_scale(camera_transform);
    let right = *camera_transform.right();
    let up = *camera_transform.up();
    right * (-cursor_delta.x * scale) + up * (cursor_delta.y * scale)
}

fn native_camera_drag_pan_scale(camera_transform: &Transform) -> f32 {
    let view_distance = native_camera_view_orbit_target(camera_transform)
        .map(|target| camera_transform.translation.distance(target))
        .unwrap_or_else(|| camera_transform.translation.length());
    (view_distance / 900.0).clamp(0.2, 4.0)
}

pub(crate) fn native_camera_scroll_zoom_delta(scroll_y: f32) -> f32 {
    -scroll_y * 80.0
}

pub(crate) fn native_camera_view_orbit_target(camera_transform: &Transform) -> Option<Vec3> {
    let forward = *camera_transform.forward();
    if forward.z.abs() <= f32::EPSILON {
        return None;
    }

    let distance_to_graph_plane = -camera_transform.translation.z / forward.z;
    if !distance_to_graph_plane.is_finite() || distance_to_graph_plane <= 0.0 {
        return None;
    }

    Some(camera_transform.translation + forward * distance_to_graph_plane)
}

pub(crate) fn native_camera_drag_orbit_transform(
    camera_transform: &mut Transform,
    orbit_target: Vec3,
    cursor_delta: Vec2,
) {
    let yaw_delta = -cursor_delta.x * 0.01;
    let pitch_delta = -cursor_delta.y * 0.01;

    camera_transform.rotate_around(orbit_target, Quat::from_axis_angle(Vec3::Y, yaw_delta));
    let right = *camera_transform.right();
    camera_transform.rotate_around(orbit_target, Quat::from_axis_angle(right, pitch_delta));
}

pub(crate) fn native_camera_orbit_translation(
    current_translation: Vec3,
    orbit_target: Vec3,
    angle_delta_radians: f32,
) -> Vec3 {
    let relative = current_translation - orbit_target;
    let (sin_delta, cos_delta) = angle_delta_radians.sin_cos();

    Vec3::new(
        orbit_target.x + relative.x * cos_delta + relative.z * sin_delta,
        current_translation.y,
        orbit_target.z - relative.x * sin_delta + relative.z * cos_delta,
    )
}

fn billboard_bible_graph_native_labels(
    cameras: Query<&Transform, With<BibleGraphNativeCamera>>,
    mut labels: Query<
        &mut Transform,
        (
            With<BibleGraphNativeLabelBillboard>,
            Without<BibleGraphNativeCamera>,
        ),
    >,
) {
    let Ok(camera_transform) = cameras.single() else {
        return;
    };

    for mut label_transform in &mut labels {
        label_transform.rotation = camera_transform.rotation;
    }
}

pub(crate) fn nearest_native_node_on_ray<'a>(
    nodes: impl Iterator<Item = &'a BibleGraphNativeNodeVisual>,
    ray: Ray3d,
) -> Option<BibleGraphNodeId> {
    let direction = *ray.direction;
    nodes
        .filter_map(|node| {
            let center = Vec3::new(node.x, node.y, node.z);
            let center_from_ray_origin = center - ray.origin;
            let ray_distance = center_from_ray_origin.dot(direction);
            if ray_distance < 0.0 {
                return None;
            }
            let closest_point = ray.origin + direction * ray_distance;
            let distance_squared = center.distance_squared(closest_point);
            (distance_squared <= node.radius.powi(2))
                .then_some((ray_distance, node.node_id.clone()))
        })
        .min_by(|(left_ray_distance, _), (right_ray_distance, _)| {
            left_ray_distance
                .partial_cmp(right_ray_distance)
                .unwrap_or(std::cmp::Ordering::Equal)
        })
        .map(|(_, node_id)| node_id)
}

pub(crate) fn nearest_selectable_native_edge_on_ray<'a>(
    edges: impl Iterator<Item = &'a BibleGraphNativeEdgeVisual>,
    ray: Ray3d,
) -> Option<BibleGraphEdgeId> {
    edges
        .filter(|edge| edge.edge_class == BibleGraphVisual3dEdgeClass::Semantic)
        .filter_map(|edge| {
            let from = Vec3::new(edge.from_x, edge.from_y, edge.from_z);
            let to = Vec3::new(edge.to_x, edge.to_y, edge.to_z);
            let ray_distance = ray_distance_to_segment(ray, from, to)?;
            let closest_point = ray.get_point(ray_distance);
            let segment_distance = distance_from_point_to_segment(closest_point, from, to);
            (segment_distance <= edge.width.max(8.0))
                .then_some((ray_distance, edge.edge_id.clone()))
        })
        .min_by(|(left_ray_distance, _), (right_ray_distance, _)| {
            left_ray_distance
                .partial_cmp(right_ray_distance)
                .unwrap_or(std::cmp::Ordering::Equal)
        })
        .map(|(_, edge_id)| edge_id)
}

fn ray_distance_to_segment(ray: Ray3d, segment_start: Vec3, segment_end: Vec3) -> Option<f32> {
    let ray_direction = *ray.direction;
    let segment_direction = segment_end - segment_start;
    let segment_length_squared = segment_direction.length_squared();
    if segment_length_squared <= f32::EPSILON {
        return None;
    }

    let ray_to_segment_start = ray.origin - segment_start;
    let ray_segment_dot = ray_direction.dot(segment_direction);
    let start_ray_dot = ray_to_segment_start.dot(ray_direction);
    let start_segment_dot = ray_to_segment_start.dot(segment_direction);
    let denominator = segment_length_squared - ray_segment_dot.powi(2);
    let unclamped_ray_distance = if denominator.abs() <= f32::EPSILON {
        -start_ray_dot
    } else {
        (ray_segment_dot * start_segment_dot - start_ray_dot * segment_length_squared) / denominator
    };
    let ray_distance = unclamped_ray_distance.max(0.0);
    let segment_position = ((ray_segment_dot * ray_distance + start_segment_dot)
        / segment_length_squared)
        .clamp(0.0, 1.0);
    let closest_segment_point = segment_start + segment_direction * segment_position;

    Some(
        (closest_segment_point - ray.origin)
            .dot(ray_direction)
            .max(0.0),
    )
}

fn distance_from_point_to_segment(point: Vec3, segment_start: Vec3, segment_end: Vec3) -> f32 {
    let segment_direction = segment_end - segment_start;
    let segment_length_squared = segment_direction.length_squared();
    if segment_length_squared <= f32::EPSILON {
        return point.distance(segment_start);
    }
    let segment_position =
        ((point - segment_start).dot(segment_direction) / segment_length_squared).clamp(0.0, 1.0);
    point.distance(segment_start + segment_direction * segment_position)
}

fn apply_bible_graph_native_projection(world: &mut World) {
    let Some(control) = world
        .get_resource::<BibleGraphNativeWindowControl>()
        .cloned()
    else {
        return;
    };
    let projection = control
        .projection
        .lock()
        .unwrap_or_else(|error| error.into_inner())
        .take();
    let Some(projection) = projection else {
        return;
    };

    rebuild_bible_graph_native_visuals(world, &projection);
    let status = *world.resource::<BibleGraphNativeVisualStatus>();
    *control
        .native_visual_counts
        .lock()
        .unwrap_or_else(|error| error.into_inner()) = status;
}

pub fn rebuild_bible_graph_native_visuals(
    world: &mut World,
    projection: &BibleRenderGraphProjection,
) {
    if !world.contains_resource::<BibleGraphNativeVisualStatus>()
        || !world.contains_resource::<BibleGraphNativeAssetCache>()
    {
        return;
    }

    let snapshot = build_bible_graph_visual_3d_snapshot(projection);
    let node_count = snapshot.nodes.len();
    let edge_count = snapshot.edges.len();

    let mut existing_edges = existing_native_edges(world);

    for edge in snapshot.edges {
        let (edge_length, edge_transform) = native_edge_segment_transform(
            Vec3::new(
                edge.from_position.x,
                edge.from_position.y,
                edge.from_position.z,
            ),
            Vec3::new(edge.to_position.x, edge.to_position.y, edge.to_position.z),
        );
        let edge_thickness = edge.radius.max(1.0);
        let edge_mesh = cached_native_edge_mesh(world, edge_length, edge_thickness);
        let edge_material = cached_native_material(
            world,
            edge.stroke_color,
            edge.selected,
            edge.highlighted,
            edge.dimmed,
        );
        let bundle = (
            BibleGraphNativeVisualEntity,
            BibleGraphNativeEdgeVisual {
                edge_id: edge.edge_id.clone(),
                edge_class: edge.edge_class,
                from_node_id: edge.from_node_id,
                to_node_id: edge.to_node_id,
                from_x: edge.from_position.x,
                from_y: edge.from_position.y,
                from_z: edge.from_position.z,
                to_x: edge.to_position.x,
                to_y: edge.to_position.y,
                to_z: edge.to_position.z,
                width: edge.radius,
                stroke_color: edge.stroke_color,
                selected: edge.selected,
                highlighted: edge.highlighted,
                dimmed: edge.dimmed,
            },
            Mesh3d(edge_mesh),
            MeshMaterial3d(edge_material),
            edge_transform,
        );
        if let Some(entity) = existing_edges.remove(&edge.edge_id) {
            world.entity_mut(entity).insert(bundle);
        } else {
            world.spawn(bundle);
        }
    }
    despawn_remaining_entities(world, existing_edges);

    let mut existing_nodes = existing_native_nodes(world);
    let mut existing_node_labels = existing_native_node_labels(world);

    for node in snapshot.nodes {
        let node_id = node.node_id.clone();
        let node_label = node.label.clone();
        let node_mesh = cached_native_node_mesh(world, node.radius);
        let node_material = cached_native_material(
            world,
            node.fill_color,
            node.selected,
            node.highlighted,
            node.dimmed,
        );
        let bundle = (
            BibleGraphNativeVisualEntity,
            BibleGraphNativeNodeVisual {
                node_id: node_id.clone(),
                x: node.position.x,
                y: node.position.y,
                z: node.position.z,
                radius: node.radius,
                fill_color: node.fill_color,
                outline_color: node.outline_color,
                selected: node.selected,
                highlighted: node.highlighted,
                dimmed: node.dimmed,
                label_visible: node.label_visible,
            },
            Mesh3d(node_mesh),
            MeshMaterial3d(node_material),
            Transform::from_translation(Vec3::new(
                node.position.x,
                node.position.y,
                node.position.z + 1.0,
            )),
        );
        if let Some(entity) = existing_nodes.remove(&node_id) {
            world.entity_mut(entity).insert(bundle);
        } else {
            world.spawn(bundle);
        }

        let label_bundle = (
            BibleGraphNativeVisualEntity,
            BibleGraphNativeNodeLabelVisual {
                node_id: node_id.clone(),
                label: node_label.clone(),
            },
            Text2d::new(node_label),
            TextFont::from_font_size(node.label_font_size),
            TextColor(native_color_from_hex(node.label_color)),
            TextLayout::new_with_justify(Justify::Center),
            bevy::sprite::Anchor::TOP_CENTER,
            BibleGraphNativeLabelBillboard,
            if node.label_visible {
                Visibility::Visible
            } else {
                Visibility::Hidden
            },
            Transform::from_translation(Vec3::new(
                node.position.x,
                node.position.y - node.radius - 6.0,
                node.position.z + 2.0,
            )),
        );
        if let Some(entity) = existing_node_labels.remove(&node_id) {
            world.entity_mut(entity).insert(label_bundle);
        } else {
            world.spawn(label_bundle);
        }
    }
    despawn_remaining_entities(world, existing_nodes);
    despawn_remaining_entities(world, existing_node_labels);

    let mut existing_influences = existing_native_influences(world);

    for influence in &projection.influences {
        let bundle = (
            BibleGraphNativeVisualEntity,
            BibleGraphNativeInfluenceVisual {
                influence_id: influence.influence_id,
                bible_node_id: influence.bible_node_id.clone(),
                bible_edge_id: influence.bible_edge_id.clone(),
            },
        );
        if let Some(entity) = existing_influences.remove(&influence.influence_id) {
            world.entity_mut(entity).insert(bundle);
        } else {
            world.spawn(bundle);
        }
    }
    despawn_remaining_entities(world, existing_influences);

    let mut status = world.resource_mut::<BibleGraphNativeVisualStatus>();
    status.node_count = node_count;
    status.edge_count = edge_count;
}

fn cached_native_node_mesh(world: &mut World, radius: f32) -> Handle<Mesh> {
    let radius = radius.max(1.0);
    let key = quantized_visual_scalar(radius);
    if let Some(handle) = world
        .resource::<BibleGraphNativeAssetCache>()
        .node_meshes
        .get(&key)
        .cloned()
    {
        return handle;
    }

    let handle = world
        .resource_mut::<Assets<Mesh>>()
        .add(Sphere::new(radius));
    world
        .resource_mut::<BibleGraphNativeAssetCache>()
        .node_meshes
        .insert(key, handle.clone());
    handle
}

fn cached_native_edge_mesh(
    world: &mut World,
    edge_length: f32,
    edge_thickness: f32,
) -> Handle<Mesh> {
    let edge_length = edge_length.max(1.0);
    let edge_thickness = edge_thickness.max(1.0);
    let key = (
        quantized_visual_scalar(edge_length),
        quantized_visual_scalar(edge_thickness),
    );
    if let Some(handle) = world
        .resource::<BibleGraphNativeAssetCache>()
        .edge_meshes
        .get(&key)
        .cloned()
    {
        return handle;
    }

    let handle = world.resource_mut::<Assets<Mesh>>().add(Cuboid::new(
        edge_length,
        edge_thickness,
        edge_thickness,
    ));
    world
        .resource_mut::<BibleGraphNativeAssetCache>()
        .edge_meshes
        .insert(key, handle.clone());
    handle
}

fn cached_native_material(
    world: &mut World,
    color: &'static str,
    selected: bool,
    highlighted: bool,
    dimmed: bool,
) -> Handle<StandardMaterial> {
    let key = BibleGraphNativeMaterialKey {
        color,
        selected,
        highlighted,
        dimmed,
    };
    if let Some(handle) = world
        .resource::<BibleGraphNativeAssetCache>()
        .materials
        .get(&key)
        .cloned()
    {
        return handle;
    }

    let handle = world
        .resource_mut::<Assets<StandardMaterial>>()
        .add(native_standard_material(
            color,
            selected,
            highlighted,
            dimmed,
        ));
    world
        .resource_mut::<BibleGraphNativeAssetCache>()
        .materials
        .insert(key, handle.clone());
    handle
}

fn quantized_visual_scalar(value: f32) -> u32 {
    (value.max(0.0) * 100.0).round() as u32
}

pub fn update_bible_graph_renderer_window_bounds(world: &mut World, width_px: u32, height_px: u32) {
    if let Some(mut status) = world.get_resource_mut::<BibleGraphNativeRendererWindowStatus>() {
        status.bounds = BibleGraphNativeRendererWindowBounds {
            width_px,
            height_px,
        };
    }
}

pub fn emit_bible_graph_native_node_selection(
    world: &mut World,
    node_id: BibleGraphNodeId,
) -> Result<(), BibleGraphRendererError> {
    validate_native_node(world, &node_id)?;
    push_native_command(world, BibleGraphRendererCommand::SelectNode { node_id })
}

pub fn emit_bible_graph_native_node_inspection(
    world: &mut World,
    node_id: BibleGraphNodeId,
) -> Result<(), BibleGraphRendererError> {
    validate_native_node(world, &node_id)?;
    push_native_command(world, BibleGraphRendererCommand::InspectNode { node_id })
}

pub fn emit_bible_graph_native_node_focus(
    world: &mut World,
    node_id: BibleGraphNodeId,
) -> Result<(), BibleGraphRendererError> {
    validate_native_node(world, &node_id)?;
    push_native_command(world, BibleGraphRendererCommand::FocusNode { node_id })
}

pub fn emit_bible_graph_native_node_navigation(
    world: &mut World,
    node_id: BibleGraphNodeId,
) -> Result<(), BibleGraphRendererError> {
    validate_native_node(world, &node_id)?;
    push_native_command(world, BibleGraphRendererCommand::NavigateToNode { node_id })
}

pub fn emit_bible_graph_native_node_delete(
    world: &mut World,
    node_id: BibleGraphNodeId,
) -> Result<(), BibleGraphRendererError> {
    validate_native_node(world, &node_id)?;
    push_native_command(world, BibleGraphRendererCommand::DeleteNode { node_id })
}

pub fn emit_bible_graph_native_connected_node_create(
    world: &mut World,
    parent_id: BibleGraphNodeId,
) -> Result<(), BibleGraphRendererError> {
    validate_native_node(world, &parent_id)?;
    push_native_command(
        world,
        BibleGraphRendererCommand::CreateConnectedNode { parent_id },
    )
}

pub fn emit_bible_graph_native_edge_selection(
    world: &mut World,
    edge_id: BibleGraphEdgeId,
) -> Result<(), BibleGraphRendererError> {
    validate_native_edge(world, &edge_id)?;
    push_native_command(world, BibleGraphRendererCommand::SelectEdge { edge_id })
}

pub fn emit_bible_graph_native_influence_selection(
    world: &mut World,
    influence_id: ContextInfluenceId,
) -> Result<(), BibleGraphRendererError> {
    validate_native_influence(world, influence_id)?;
    push_native_command(
        world,
        BibleGraphRendererCommand::SelectInfluence { influence_id },
    )
}

pub fn emit_bible_graph_native_clear_selection(
    world: &mut World,
) -> Result<(), BibleGraphRendererError> {
    push_native_command(world, BibleGraphRendererCommand::ClearSelection)
}

fn validate_native_node(
    world: &mut World,
    node_id: &BibleGraphNodeId,
) -> Result<(), BibleGraphRendererError> {
    if world
        .query::<&BibleGraphNativeNodeVisual>()
        .iter(world)
        .any(|node| &node.node_id == node_id)
    {
        Ok(())
    } else {
        Err(BibleGraphRendererError::UnknownNode {
            node_id: node_id.clone(),
        })
    }
}

fn validate_native_edge(
    world: &mut World,
    edge_id: &BibleGraphEdgeId,
) -> Result<(), BibleGraphRendererError> {
    if world
        .query::<&BibleGraphNativeEdgeVisual>()
        .iter(world)
        .any(|edge| &edge.edge_id == edge_id)
    {
        Ok(())
    } else {
        Err(BibleGraphRendererError::UnknownEdge {
            edge_id: edge_id.clone(),
        })
    }
}

fn validate_native_influence(
    world: &mut World,
    influence_id: ContextInfluenceId,
) -> Result<(), BibleGraphRendererError> {
    if world
        .query::<&BibleGraphNativeInfluenceVisual>()
        .iter(world)
        .any(|influence| influence.influence_id == influence_id)
    {
        Ok(())
    } else {
        Err(BibleGraphRendererError::UnknownInfluence { influence_id })
    }
}

fn push_native_command(
    world: &mut World,
    command: BibleGraphRendererCommand,
) -> Result<(), BibleGraphRendererError> {
    let Some(control) = world
        .get_resource::<BibleGraphNativeWindowControl>()
        .cloned()
    else {
        return Err(BibleGraphRendererError::MissingProjection);
    };
    let mut commands = control
        .commands
        .lock()
        .unwrap_or_else(|error| error.into_inner());
    if commands.len() >= BIBLE_GRAPH_RENDERER_COMMAND_QUEUE_CAPACITY {
        return Err(BibleGraphRendererError::CommandQueueFull);
    }

    commands.push(command);
    Ok(())
}

fn push_native_command_to_control(
    control: &BibleGraphNativeWindowControl,
    command: BibleGraphRendererCommand,
) -> Result<(), BibleGraphRendererError> {
    let mut commands = control
        .commands
        .lock()
        .unwrap_or_else(|error| error.into_inner());
    if commands.len() >= BIBLE_GRAPH_RENDERER_COMMAND_QUEUE_CAPACITY {
        return Err(BibleGraphRendererError::CommandQueueFull);
    }

    commands.push(command);
    Ok(())
}

pub(crate) fn native_visual_state_color(
    color: &str,
    selected: bool,
    highlighted: bool,
    dimmed: bool,
) -> Color {
    let (red, green, blue) = graph_color_components(color);
    if dimmed {
        return Color::srgb(red * 0.32, green * 0.32, blue * 0.32);
    }
    if selected || highlighted {
        return Color::srgb(
            (red + 0.22).min(1.0),
            (green + 0.22).min(1.0),
            (blue + 0.22).min(1.0),
        );
    }
    Color::srgb(red, green, blue)
}

pub(crate) fn native_standard_material(
    color: &str,
    selected: bool,
    highlighted: bool,
    dimmed: bool,
) -> StandardMaterial {
    StandardMaterial {
        base_color: native_visual_state_color(color, selected, highlighted, dimmed),
        unlit: true,
        ..Default::default()
    }
}

fn native_color_from_hex(color: &str) -> Color {
    let (red, green, blue) = graph_color_components(color);
    Color::srgb(red, green, blue)
}

fn graph_color_components(color: &str) -> (f32, f32, f32) {
    if let Some(components) = graph_hex_color_components(color) {
        return components;
    }

    match color {
        "#f2c94c" => (0.949, 0.788, 0.298),
        "#253041" => (0.145, 0.188, 0.255),
        "#11151d" => (0.067, 0.082, 0.114),
        "#40576f" => (0.251, 0.341, 0.435),
        "#52687f" => (0.322, 0.408, 0.498),
        "#1f6f78" => (0.122, 0.435, 0.471),
        "#2f7a6e" => (0.184, 0.478, 0.431),
        "#3f668f" => (0.247, 0.4, 0.561),
        "#7a5c8f" => (0.478, 0.361, 0.561),
        "#6f7a2f" => (0.435, 0.478, 0.184),
        "#8f4f5c" => (0.561, 0.31, 0.361),
        "#8a6f3d" => (0.541, 0.435, 0.239),
        "#8f5c3f" => (0.561, 0.361, 0.247),
        "#4f7f8f" => (0.31, 0.498, 0.561),
        "#536f88" => (0.325, 0.435, 0.533),
        "#6fc2c9" => (0.435, 0.761, 0.788),
        "#f6d977" => (0.965, 0.851, 0.467),
        "#c9f3f5" => (0.788, 0.953, 0.961),
        "#dbe3ea" => (0.859, 0.89, 0.918),
        "#34495e" => (0.204, 0.286, 0.369),
        _ => (0.8, 0.84, 0.9),
    }
}

fn graph_hex_color_components(color: &str) -> Option<(f32, f32, f32)> {
    let hex = color.strip_prefix('#')?;
    if hex.len() != 6 {
        return None;
    }

    let red = u8::from_str_radix(&hex[0..2], 16).ok()?;
    let green = u8::from_str_radix(&hex[2..4], 16).ok()?;
    let blue = u8::from_str_radix(&hex[4..6], 16).ok()?;

    Some((
        f32::from(red) / 255.0,
        f32::from(green) / 255.0,
        f32::from(blue) / 255.0,
    ))
}

fn existing_native_nodes(world: &mut World) -> HashMap<BibleGraphNodeId, Entity> {
    world
        .query::<(Entity, &BibleGraphNativeNodeVisual)>()
        .iter(world)
        .map(|(entity, node)| (node.node_id.clone(), entity))
        .collect()
}

fn existing_native_node_labels(world: &mut World) -> HashMap<BibleGraphNodeId, Entity> {
    world
        .query::<(Entity, &BibleGraphNativeNodeLabelVisual)>()
        .iter(world)
        .map(|(entity, label)| (label.node_id.clone(), entity))
        .collect()
}

fn existing_native_edges(world: &mut World) -> HashMap<BibleGraphEdgeId, Entity> {
    world
        .query::<(Entity, &BibleGraphNativeEdgeVisual)>()
        .iter(world)
        .map(|(entity, edge)| (edge.edge_id.clone(), entity))
        .collect()
}

fn existing_native_influences(world: &mut World) -> HashMap<ContextInfluenceId, Entity> {
    world
        .query::<(Entity, &BibleGraphNativeInfluenceVisual)>()
        .iter(world)
        .map(|(entity, influence)| (influence.influence_id, entity))
        .collect()
}

fn despawn_remaining_entities<T>(world: &mut World, existing_entities: HashMap<T, Entity>) {
    for entity in existing_entities.into_values() {
        let _ = world.despawn(entity);
    }
}
