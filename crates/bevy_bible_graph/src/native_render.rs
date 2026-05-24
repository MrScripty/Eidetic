use std::num::NonZeroU64;
use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::{Arc, Mutex};
use std::time::{Duration, Instant};

use bevy::app::AppExit;
use bevy::prelude::{
    App, Camera2d, ClearColor, Color, Commands, Component, DefaultPlugins, Entity, MessageWriter,
    Plugin, PluginGroup, Res, ResMut, Resource, Sprite, Startup, Transform, Update, Vec2, Vec3,
    With, World,
};
use bevy::window::{ExitCondition, Window, WindowPlugin, WindowResolution};
use bevy::winit::WinitPlugin;
use eidetic_core::contracts::{BibleGraphEdgeId, BibleGraphNodeId, BibleRenderGraphProjection};

use crate::build_bible_graph_visual_snapshot;

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
    pub highlighted: bool,
}

#[derive(Component, Debug, Clone, PartialEq)]
pub struct BibleGraphNativeEdgeVisual {
    pub edge_id: BibleGraphEdgeId,
    pub from_node_id: BibleGraphNodeId,
    pub to_node_id: BibleGraphNodeId,
    pub from_x: f32,
    pub from_y: f32,
    pub to_x: f32,
    pub to_y: f32,
    pub width: f32,
    pub stroke_color: &'static str,
    pub highlighted: bool,
}

impl Default for BibleGraphNativeRenderConfig {
    fn default() -> Self {
        Self {
            borderless_window: true,
        }
    }
}

impl BibleGraphNativeWindowRunnerConfig {
    pub fn minimal_smoke(run_on_any_thread: bool) -> Self {
        Self {
            title: "Eidetic Bible Graph".to_string(),
            width_px: 1280,
            height_px: 720,
            borderless_window: true,
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
        }
    }
}

pub struct BibleGraphNativeRenderPlugin;

impl Plugin for BibleGraphNativeRenderPlugin {
    fn build(&self, app: &mut App) {
        app.insert_resource(BibleGraphNativeRenderConfig::default());
        app.insert_resource(BibleGraphNativeRendererWindowScene::default());
        app.insert_resource(BibleGraphNativeRendererWindowStatus::default());
        app.insert_resource(BibleGraphNativeVisualStatus::default());
        app.insert_resource(ClearColor(Color::srgb(0.067, 0.082, 0.114)));
        app.add_systems(Startup, spawn_bible_graph_renderer_window_scene);
        app.add_systems(Startup, mark_bible_graph_native_window_ready);
        app.add_systems(Update, apply_bible_graph_native_projection);
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
    commands.spawn((Camera2d, BibleGraphNativeCamera));
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
    if !world.contains_resource::<BibleGraphNativeVisualStatus>() {
        return;
    }

    despawn_existing_native_visuals(world);
    let snapshot = build_bible_graph_visual_snapshot(projection);
    let node_count = snapshot.nodes.len();
    let edge_count = snapshot.edges.len();

    for edge in snapshot.edges {
        let edge_length = ((edge.to_position.x - edge.from_position.x).powi(2)
            + (edge.to_position.y - edge.from_position.y).powi(2))
        .sqrt()
        .max(1.0);
        let edge_angle = (edge.to_position.y - edge.from_position.y)
            .atan2(edge.to_position.x - edge.from_position.x);
        let edge_midpoint_x = (edge.from_position.x + edge.to_position.x) / 2.0;
        let edge_midpoint_y = (edge.from_position.y + edge.to_position.y) / 2.0;
        world.spawn((
            BibleGraphNativeVisualEntity,
            BibleGraphNativeEdgeVisual {
                edge_id: edge.edge_id,
                from_node_id: edge.from_node_id,
                to_node_id: edge.to_node_id,
                from_x: edge.from_position.x,
                from_y: edge.from_position.y,
                to_x: edge.to_position.x,
                to_y: edge.to_position.y,
                width: edge.width,
                stroke_color: edge.stroke_color,
                highlighted: edge.highlighted,
            },
            Sprite::from_color(
                graph_color(edge.stroke_color),
                Vec2::new(edge_length, edge.width.max(1.0)),
            ),
            Transform {
                translation: Vec3::new(edge_midpoint_x, edge_midpoint_y, 0.0),
                rotation: bevy::prelude::Quat::from_rotation_z(edge_angle),
                ..Default::default()
            },
        ));
    }

    for node in snapshot.nodes {
        world.spawn((
            BibleGraphNativeVisualEntity,
            BibleGraphNativeNodeVisual {
                node_id: node.node_id,
                x: node.position.x,
                y: node.position.y,
                z: node.position.z,
                radius: node.radius,
                fill_color: node.fill_color,
                outline_color: node.outline_color,
                highlighted: node.highlighted,
            },
            Sprite::from_color(
                graph_color(node.fill_color),
                Vec2::splat((node.radius * 2.0).max(1.0)),
            ),
            Transform::from_translation(Vec3::new(
                node.position.x,
                node.position.y,
                node.position.z + 1.0,
            )),
        ));
    }

    let mut status = world.resource_mut::<BibleGraphNativeVisualStatus>();
    status.node_count = node_count;
    status.edge_count = edge_count;
}

pub fn update_bible_graph_renderer_window_bounds(world: &mut World, width_px: u32, height_px: u32) {
    if let Some(mut status) = world.get_resource_mut::<BibleGraphNativeRendererWindowStatus>() {
        status.bounds = BibleGraphNativeRendererWindowBounds {
            width_px,
            height_px,
        };
    }
}

fn graph_color(color: &str) -> Color {
    match color {
        "#f2c94c" => Color::srgb(0.949, 0.788, 0.298),
        "#253041" => Color::srgb(0.145, 0.188, 0.255),
        "#11151d" => Color::srgb(0.067, 0.082, 0.114),
        _ => Color::srgb(0.8, 0.84, 0.9),
    }
}

fn despawn_existing_native_visuals(world: &mut World) {
    let entities: Vec<Entity> = world
        .query_filtered::<Entity, With<BibleGraphNativeVisualEntity>>()
        .iter(world)
        .collect();

    for entity in entities {
        let _ = world.despawn(entity);
    }
}
