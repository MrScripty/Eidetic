use std::collections::HashMap;
use std::num::NonZeroU64;
use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::{Arc, Mutex};
use std::time::{Duration, Instant};

use bevy::app::AppExit;
use bevy::prelude::{
    App, Assets, ButtonInput, Camera, Camera3d, ClearColor, Color, Commands, Component, Cuboid,
    DefaultPlugins, Entity, GlobalTransform, Justify, KeyCode, Mesh, Mesh3d, MeshMaterial3d,
    MessageWriter, MouseButton, Plugin, PluginGroup, PointLight, Query, Res, ResMut, Resource,
    Sphere, StandardMaterial, Startup, Text2d, TextColor, TextFont, TextLayout, Time, Transform,
    Update, Vec2, Vec3, Window, With, World,
};
use bevy::window::{
    ExitCondition, PrimaryWindow, WindowCloseRequested, WindowPlugin, WindowResolution,
};
use bevy::winit::WinitPlugin;
use eidetic_core::contracts::{
    BibleGraphEdgeId, BibleGraphNodeId, BibleRenderGraphProjection, ContextInfluenceId,
};

use crate::{
    BIBLE_GRAPH_RENDERER_COMMAND_QUEUE_CAPACITY, BibleGraphRendererCommand,
    BibleGraphRendererError, build_bible_graph_visual_snapshot,
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
pub struct BibleGraphNativeNodeLabelVisual {
    pub node_id: BibleGraphNodeId,
    pub label: String,
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
        }
    }
}

impl BibleGraphNativeWindowControl {
    pub fn request_close_from_os_window(&self) {
        self.shutdown_requested.store(true, Ordering::Release);
        self.visible.store(false, Ordering::Release);
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
        app.insert_resource(ClearColor(Color::srgb(0.067, 0.082, 0.114)));
        app.add_systems(Startup, spawn_bible_graph_renderer_window_scene);
        app.add_systems(Startup, mark_bible_graph_native_window_ready);
        app.add_systems(Update, apply_bible_graph_native_projection);
        app.add_systems(Update, emit_bible_graph_native_click_selection);
        app.add_systems(Update, pan_bible_graph_native_camera);
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
    let Ok(world_position) = camera.viewport_to_world_2d(camera_transform, cursor_position) else {
        return;
    };
    let Some(node_id) = nearest_native_node_at_world_position(&nodes, world_position) else {
        return;
    };

    let _ =
        push_native_command_to_control(&control, BibleGraphRendererCommand::SelectNode { node_id });
}

fn pan_bible_graph_native_camera(
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
    let mut direction = Vec2::ZERO;

    if keys.pressed(KeyCode::KeyA) || keys.pressed(KeyCode::ArrowLeft) {
        direction.x -= 1.0;
    }
    if keys.pressed(KeyCode::KeyD) || keys.pressed(KeyCode::ArrowRight) {
        direction.x += 1.0;
    }
    if keys.pressed(KeyCode::KeyW) || keys.pressed(KeyCode::ArrowUp) {
        direction.y += 1.0;
    }
    if keys.pressed(KeyCode::KeyS) || keys.pressed(KeyCode::ArrowDown) {
        direction.y -= 1.0;
    }

    if direction == Vec2::ZERO {
        return;
    }

    let pan_speed = 420.0;
    camera_transform.translation +=
        (direction.normalize() * pan_speed * time.delta_secs()).extend(0.0);
}

fn nearest_native_node_at_world_position(
    nodes: &Query<&BibleGraphNativeNodeVisual>,
    world_position: Vec2,
) -> Option<BibleGraphNodeId> {
    nodes
        .iter()
        .filter_map(|node| {
            let distance_squared =
                (node.x - world_position.x).powi(2) + (node.y - world_position.y).powi(2);
            (distance_squared <= node.radius.powi(2))
                .then_some((distance_squared, node.node_id.clone()))
        })
        .min_by(|(left_distance, _), (right_distance, _)| {
            left_distance
                .partial_cmp(right_distance)
                .unwrap_or(std::cmp::Ordering::Equal)
        })
        .map(|(_, node_id)| node_id)
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

    let snapshot = build_bible_graph_visual_snapshot(projection);
    let node_count = snapshot.nodes.len();
    let edge_count = snapshot.edges.len();

    let mut existing_edges = existing_native_edges(world);

    for edge in snapshot.edges {
        let edge_length = ((edge.to_position.x - edge.from_position.x).powi(2)
            + (edge.to_position.y - edge.from_position.y).powi(2))
        .sqrt()
        .max(1.0);
        let edge_angle = (edge.to_position.y - edge.from_position.y)
            .atan2(edge.to_position.x - edge.from_position.x);
        let edge_midpoint_x = (edge.from_position.x + edge.to_position.x) / 2.0;
        let edge_midpoint_y = (edge.from_position.y + edge.to_position.y) / 2.0;
        let edge_midpoint_z = (edge.from_position.z + edge.to_position.z) / 2.0;
        let edge_thickness = edge.width.max(1.0);
        let edge_mesh = {
            let mut meshes = world.resource_mut::<Assets<Mesh>>();
            meshes.add(Cuboid::new(edge_length, edge_thickness, edge_thickness))
        };
        let edge_material = {
            let mut materials = world.resource_mut::<Assets<StandardMaterial>>();
            materials.add(graph_color(edge.stroke_color))
        };
        let bundle = (
            BibleGraphNativeVisualEntity,
            BibleGraphNativeEdgeVisual {
                edge_id: edge.edge_id.clone(),
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
            Mesh3d(edge_mesh),
            MeshMaterial3d(edge_material),
            Transform {
                translation: Vec3::new(edge_midpoint_x, edge_midpoint_y, edge_midpoint_z),
                rotation: bevy::prelude::Quat::from_rotation_z(edge_angle),
                ..Default::default()
            },
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
        let node_mesh = {
            let mut meshes = world.resource_mut::<Assets<Mesh>>();
            meshes.add(Sphere::new(node.radius.max(1.0)))
        };
        let node_material = {
            let mut materials = world.resource_mut::<Assets<StandardMaterial>>();
            materials.add(graph_color(node.fill_color))
        };
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
                highlighted: node.highlighted,
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
            TextFont::from_font_size(if node.highlighted { 15.0 } else { 13.0 }),
            TextColor(Color::srgb(0.86, 0.9, 0.94)),
            TextLayout::new_with_justify(Justify::Center),
            bevy::sprite::Anchor::TOP_CENTER,
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

fn graph_color(color: &str) -> Color {
    match color {
        "#f2c94c" => Color::srgb(0.949, 0.788, 0.298),
        "#253041" => Color::srgb(0.145, 0.188, 0.255),
        "#11151d" => Color::srgb(0.067, 0.082, 0.114),
        "#40576f" => Color::srgb(0.251, 0.341, 0.435),
        "#52687f" => Color::srgb(0.322, 0.408, 0.498),
        "#1f6f78" => Color::srgb(0.122, 0.435, 0.471),
        "#2f7a6e" => Color::srgb(0.184, 0.478, 0.431),
        "#3f668f" => Color::srgb(0.247, 0.4, 0.561),
        "#7a5c8f" => Color::srgb(0.478, 0.361, 0.561),
        "#8a6f3d" => Color::srgb(0.541, 0.435, 0.239),
        "#34495e" => Color::srgb(0.204, 0.286, 0.369),
        _ => Color::srgb(0.8, 0.84, 0.9),
    }
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
