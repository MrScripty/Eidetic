use bevy::prelude::{
    App, Camera2d, ClearColor, Color, Commands, Component, Entity, MinimalPlugins, Plugin, ResMut,
    Resource, Startup, With, World,
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
}

#[derive(Debug, Clone, PartialEq, Eq, Resource)]
pub struct BibleGraphNativeRendererWindowScene {
    pub background_color: &'static str,
    pub grid_color: &'static str,
    pub accent_color: &'static str,
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
    }
}

pub fn configure_minimal_bible_graph_native_window_app(
    app: &mut App,
    config: BibleGraphNativeWindowRunnerConfig,
) {
    app.add_plugins(MinimalPlugins);
    app.add_plugins(WindowPlugin {
        primary_window: Some(Window {
            title: config.title,
            resolution: WindowResolution::new(config.width_px, config.height_px),
            decorations: !config.borderless_window,
            ..Default::default()
        }),
        exit_condition: ExitCondition::OnPrimaryClosed,
        close_when_requested: true,
        ..Default::default()
    });
    app.add_plugins(WinitPlugin {
        run_on_any_thread: config.run_on_any_thread,
    });
    app.add_plugins(BibleGraphNativeRenderPlugin);
}

pub fn run_minimal_bible_graph_native_window(config: BibleGraphNativeWindowRunnerConfig) {
    let mut app = App::new();
    configure_minimal_bible_graph_native_window_app(&mut app, config);
    app.run();
}

fn spawn_bible_graph_renderer_window_scene(
    mut commands: Commands,
    mut status: ResMut<BibleGraphNativeRendererWindowStatus>,
) {
    commands.spawn((Camera2d, BibleGraphNativeCamera));
    status.camera_count = 1;
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

fn despawn_existing_native_visuals(world: &mut World) {
    let entities: Vec<Entity> = world
        .query_filtered::<Entity, With<BibleGraphNativeVisualEntity>>()
        .iter(world)
        .collect();

    for entity in entities {
        let _ = world.despawn(entity);
    }
}
