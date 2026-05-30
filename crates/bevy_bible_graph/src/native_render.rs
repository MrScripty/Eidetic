use std::collections::HashMap;
use std::num::NonZeroU64;
use std::sync::atomic::{AtomicBool, AtomicU64, Ordering};
use std::sync::{Arc, Mutex};
use std::time::{Duration, Instant};

use bevy::app::AppExit;
use bevy::asset::{Asset, embedded_asset};
use bevy::color::LinearRgba;
use bevy::input::ButtonState;
use bevy::input::keyboard::KeyboardInput;
use bevy::input::mouse::{MouseMotion, MouseScrollUnit, MouseWheel};
use bevy::pbr::{Material, MaterialPlugin};
use bevy::prelude::{
    App, Assets, BorderRadius, ButtonInput, Camera, Camera2d, Camera3d, ClearColor,
    ClearColorConfig, Color, Commands, Component, Cuboid, DefaultPlugins, Entity, GlobalTransform,
    Handle, IntoScheduleConfigs, Justify, KeyCode, Mesh, Mesh3d, MeshMaterial3d, MessageReader,
    MessageWriter, MouseButton, Mut, Plugin, PluginGroup, PointLight, Quat, Query, Ray3d, Res,
    ResMut, Resource, Sphere, Startup, SystemSet, TextColor, TextFont, TextLayout, Time, Torus,
    Transform, Update, Vec2, Vec3, Visibility, Window, With, Without, World,
};
use bevy::reflect::TypePath;
use bevy::render::render_resource::AsBindGroup;
use bevy::shader::ShaderRef;
use bevy::ui::prelude::{
    BackgroundColor, BorderColor, IsDefaultUiCamera, Node, Overflow, PositionType, ScrollPosition,
    Text, UiRect, UiTargetCamera, Val,
};
use bevy::window::{
    ExitCondition, PrimaryWindow, WindowCloseRequested, WindowPlugin, WindowResolution,
};
use bevy::winit::WinitPlugin;
use eidetic_core::contracts::{
    BibleGraphEdgeId, BibleGraphNodeId, BibleRenderGraphProjection, ContextInfluenceId,
};
use serde::{Deserialize, Serialize};

use crate::{
    BIBLE_GRAPH_RENDERER_COMMAND_QUEUE_CAPACITY, BIBLE_GRAPH_WORKSPACE_TIMELINE_PANEL_DEPTH,
    BIBLE_GRAPH_WORKSPACE_TIMELINE_PANEL_HEIGHT, BIBLE_GRAPH_WORKSPACE_TIMELINE_PANEL_WIDTH,
    BibleGraphCameraCommand, BibleGraphRendererCommand, BibleGraphRendererError,
    BibleGraphVisual3dEdgeClass, BibleGraphWorkspaceTimelineClipVisual,
    BibleGraphWorkspaceTimelinePresentation, BibleGraphWorkspaceTimelinePresentationMode,
    BibleGraphWorkspaceTimelineTrackVisual, BibleGraphWorkspaceTimelineVisualSnapshot,
    build_bible_graph_visual_3d_snapshot,
    native_text_editor::{
        NATIVE_NODE_TEXT_EDITOR_CARET_HEIGHT_PX, NATIVE_NODE_TEXT_EDITOR_CARET_WIDTH_PX,
        NATIVE_NODE_TEXT_EDITOR_FONT_SIZE_PX, NATIVE_NODE_TEXT_EDITOR_HEIGHT_RATIO,
        NATIVE_NODE_TEXT_EDITOR_RIGHT_PX, NATIVE_NODE_TEXT_EDITOR_TOP_PX,
        NATIVE_NODE_TEXT_EDITOR_WIDTH_PX, bible_graph_native_text_editor_caret_position,
        bible_graph_native_text_editor_delete_backward,
        bible_graph_native_text_editor_index_from_position,
        bible_graph_native_text_editor_local_position, bible_graph_native_text_editor_move_left,
        bible_graph_native_text_editor_move_right, bible_graph_native_text_editor_move_vertical,
    },
};

const NATIVE_CAMERA_EDGE_PAN_MARGIN_PX: f32 = 36.0;
const NATIVE_CAMERA_EDGE_PAN_SPEED: f32 = 520.0;
const NATIVE_LABEL_SCREEN_OFFSET_PX: f32 = 18.0;
const NATIVE_NODE_TITLE_DOUBLE_CLICK_SECONDS: f64 = 0.45;
const NATIVE_NODE_TEXT_SAVE_DEBOUNCE_SECONDS: f64 = 0.8;

#[derive(Debug, Clone, Copy, Default, PartialEq, Eq, Resource)]
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
    workspace_timeline_visual_snapshot:
        Arc<Mutex<Option<BibleGraphWorkspaceTimelineVisualSnapshot>>>,
    native_visual_counts: Arc<Mutex<BibleGraphNativeVisualStatus>>,
    commands: Arc<Mutex<Vec<BibleGraphRendererCommand>>>,
    camera_commands: Arc<Mutex<Vec<BibleGraphCameraCommand>>>,
    text_editor_settings: Arc<Mutex<BibleGraphNativeTextEditorSettings>>,
    text_editor_settings_version: Arc<AtomicU64>,
}

#[derive(Debug, Clone, Resource)]
pub struct BibleGraphNativeWindowControl {
    shutdown_requested: Arc<AtomicBool>,
    show_requested: Arc<AtomicBool>,
    hide_requested: Arc<AtomicBool>,
    visible: Arc<AtomicBool>,
    ready: Arc<AtomicBool>,
    projection: Arc<Mutex<Option<BibleRenderGraphProjection>>>,
    workspace_timeline_visual_snapshot:
        Arc<Mutex<Option<BibleGraphWorkspaceTimelineVisualSnapshot>>>,
    native_visual_counts: Arc<Mutex<BibleGraphNativeVisualStatus>>,
    commands: Arc<Mutex<Vec<BibleGraphRendererCommand>>>,
    camera_commands: Arc<Mutex<Vec<BibleGraphCameraCommand>>>,
    text_editor_settings: Arc<Mutex<BibleGraphNativeTextEditorSettings>>,
    text_editor_settings_version: Arc<AtomicU64>,
}

#[derive(Debug, Clone, Copy, Default, PartialEq, Eq, Resource)]
pub struct BibleGraphNativeRendererWindowStatus {
    pub camera_count: usize,
    pub bounds: BibleGraphNativeRendererWindowBounds,
}

#[derive(Debug, Clone, Default, PartialEq, Resource)]
pub struct BibleGraphNativeWorkspaceTimelineVisualState {
    pub snapshot: Option<BibleGraphWorkspaceTimelineVisualSnapshot>,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct BibleGraphNativeTextEditorSettings {
    pub padding_px: f32,
    pub corner_radius_px: f32,
    pub editor_outline_width_px: f32,
    pub editor_outline_brightness: f32,
    pub editor_outline_transparency: f32,
    pub font_size_px: f32,
    pub font_brightness: f32,
    pub editor_background_color: String,
    pub editor_background_brightness: f32,
    pub editor_background_transparency: f32,
    #[serde(default = "default_native_label_size_scale")]
    pub label_size_scale: f32,
    pub selected_node_outline_width_px: f32,
    pub selected_node_outline_brightness: f32,
    pub selected_node_outline_color: String,
}

impl Default for BibleGraphNativeTextEditorSettings {
    fn default() -> Self {
        Self {
            padding_px: 17.0,
            corner_radius_px: 4.0,
            editor_outline_width_px: 1.0,
            editor_outline_brightness: 0.12,
            editor_outline_transparency: 0.05,
            font_size_px: NATIVE_NODE_TEXT_EDITOR_FONT_SIZE_PX,
            font_brightness: 0.88,
            editor_background_color: "#ffffff".to_string(),
            editor_background_brightness: 0.075,
            editor_background_transparency: 0.08,
            label_size_scale: 1.0,
            selected_node_outline_width_px: 4.0,
            selected_node_outline_brightness: 1.0,
            selected_node_outline_color: "#f2c94c".to_string(),
        }
    }
}

fn default_native_label_size_scale() -> f32 {
    1.0
}

#[derive(Debug, Clone, Default, PartialEq, Resource)]
struct BibleGraphNativeTextEditorSettingsState {
    settings: BibleGraphNativeTextEditorSettings,
    version: u64,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Resource)]
pub struct BibleGraphNativeLabelOverlayTarget {
    pub camera_entity: Entity,
}

type BibleGraphNativeLabelProjectionItem<'a> = (
    &'a BibleGraphNativeNodeLabelVisual,
    &'a mut Node,
    &'a mut TextFont,
    &'a mut Visibility,
);
type BibleGraphNativeLabelProjectionFilter = (
    With<BibleGraphNativeLabelBillboard>,
    Without<BibleGraphNativeCamera>,
);

#[derive(Debug, Clone, Copy, Default, PartialEq, Eq, Resource)]
pub struct BibleGraphNativeVisualStatus {
    pub node_count: usize,
    pub edge_count: usize,
}

#[derive(Debug, Default, Resource)]
struct BibleGraphNativeAssetCache {
    node_meshes: HashMap<u32, Handle<Mesh>>,
    selection_outline_meshes: HashMap<(u32, u32), Handle<Mesh>>,
    edge_meshes: HashMap<(u32, u32), Handle<Mesh>>,
    materials: HashMap<BibleGraphNativeMaterialKey, Handle<BibleGraphNativeMaterial>>,
}

#[derive(AsBindGroup, Asset, TypePath, Debug, Clone)]
pub(crate) struct BibleGraphNativeMaterial {
    #[uniform(0)]
    pub(crate) color: LinearRgba,
}

impl Material for BibleGraphNativeMaterial {
    fn fragment_shader() -> ShaderRef {
        "embedded://eidetic_bevy_bible_graph/native_graph_material.wgsl".into()
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Hash)]
struct BibleGraphNativeMaterialKey {
    color: String,
    selected: bool,
    highlighted: bool,
    dimmed: bool,
    brightness: u32,
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

#[derive(Component)]
pub struct BibleGraphNativeLabelOverlayCamera;

#[derive(Component)]
pub struct BibleGraphNativeWorkspaceTimelineRoot;

#[derive(Component)]
pub struct BibleGraphNativeWorkspaceTimelineVisualEntity;

#[derive(Component, Debug, Clone, PartialEq)]
pub struct BibleGraphNativeWorkspaceTimelineClipVisual {
    pub clip: BibleGraphWorkspaceTimelineClipVisual,
}

#[derive(Component, Debug, Clone, PartialEq)]
pub struct BibleGraphNativeWorkspaceTimelineTrackVisual {
    pub track: BibleGraphWorkspaceTimelineTrackVisual,
}

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
    pub x: f32,
    pub y: f32,
    pub z: f32,
    pub radius: f32,
    pub label_font_size: f32,
    pub label_visible: bool,
}

#[derive(Component, Debug, Clone, PartialEq)]
pub struct BibleGraphNativeNodeTextEditorVisual {
    pub node_id: BibleGraphNodeId,
    pub cursor_byte_index: usize,
    pub last_seen_text: String,
    pub last_sent_text: String,
    pub dirty_since_seconds: Option<f64>,
}

#[derive(Component, Debug, Clone, PartialEq)]
pub struct BibleGraphNativeNodeTextEditorText {
    pub node_id: BibleGraphNodeId,
}

#[derive(Component, Debug, Clone, PartialEq)]
pub struct BibleGraphNativeNodeTextEditorCaret {
    pub node_id: BibleGraphNodeId,
}

#[derive(Component, Debug, Clone, PartialEq)]
pub struct BibleGraphNativeLabelBillboard;

#[derive(Component, Debug, Clone, PartialEq)]
pub struct BibleGraphNativeSelectionOutlineVisual {
    pub node_id: BibleGraphNodeId,
    pub radius: f32,
    pub outline_width_px: f32,
    pub outline_color: String,
}

#[derive(Component, Debug, Clone, PartialEq)]
pub struct BibleGraphNativeSelectionOutlineBillboard;

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

#[derive(Debug, Clone, Default, PartialEq, Resource)]
pub(crate) struct BibleGraphNativeNodeTitleEdit {
    active: Option<BibleGraphNativeNodeTitleEditState>,
    last_click: Option<BibleGraphNativeNodeClick>,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub(crate) struct BibleGraphNativeNodeTitleEditState {
    node_id: BibleGraphNodeId,
    value: String,
}

#[derive(Debug, Clone, PartialEq)]
struct BibleGraphNativeNodeClick {
    node_id: BibleGraphNodeId,
    at_seconds: f64,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, SystemSet)]
enum BibleGraphNativeRenderSystem {
    Projection,
    Input,
    Camera,
    Labels,
    Outlines,
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
            workspace_timeline_visual_snapshot: Arc::new(Mutex::new(None)),
            native_visual_counts: Arc::new(Mutex::new(BibleGraphNativeVisualStatus::default())),
            commands: Arc::new(Mutex::new(Vec::new())),
            camera_commands: Arc::new(Mutex::new(Vec::new())),
            text_editor_settings: Arc::new(Mutex::new(
                BibleGraphNativeTextEditorSettings::default(),
            )),
            text_editor_settings_version: Arc::new(AtomicU64::new(0)),
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

    pub fn set_workspace_timeline_visual_snapshot(
        &self,
        snapshot: BibleGraphWorkspaceTimelineVisualSnapshot,
    ) {
        *self
            .workspace_timeline_visual_snapshot
            .lock()
            .unwrap_or_else(|error| error.into_inner()) = Some(snapshot);
    }

    pub fn take_workspace_timeline_visual_snapshot(
        &self,
    ) -> Option<BibleGraphWorkspaceTimelineVisualSnapshot> {
        self.workspace_timeline_visual_snapshot
            .lock()
            .unwrap_or_else(|error| error.into_inner())
            .take()
    }

    pub fn set_text_editor_settings(&self, settings: BibleGraphNativeTextEditorSettings) {
        *self
            .text_editor_settings
            .lock()
            .unwrap_or_else(|error| error.into_inner()) = settings;
        self.text_editor_settings_version
            .fetch_add(1, Ordering::AcqRel);
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
            workspace_timeline_visual_snapshot: Arc::clone(
                &handle.workspace_timeline_visual_snapshot,
            ),
            native_visual_counts: Arc::clone(&handle.native_visual_counts),
            commands: Arc::clone(&handle.commands),
            camera_commands: Arc::clone(&handle.camera_commands),
            text_editor_settings: Arc::clone(&handle.text_editor_settings),
            text_editor_settings_version: Arc::clone(&handle.text_editor_settings_version),
        }
    }
}

impl BibleGraphNativeWindowControl {
    pub fn request_close_from_os_window(&self) {
        self.shutdown_requested.store(true, Ordering::Release);
        self.visible.store(false, Ordering::Release);
    }

    fn take_workspace_timeline_visual_snapshot(
        &self,
    ) -> Option<BibleGraphWorkspaceTimelineVisualSnapshot> {
        self.workspace_timeline_visual_snapshot
            .lock()
            .unwrap_or_else(|error| error.into_inner())
            .take()
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
        if app
            .world()
            .contains_resource::<bevy::asset::io::embedded::EmbeddedAssetRegistry>()
        {
            embedded_asset!(app, "native_graph_material.wgsl");
            app.add_plugins(MaterialPlugin::<BibleGraphNativeMaterial>::default());
        } else {
            app.init_resource::<Assets<BibleGraphNativeMaterial>>();
        }
        app.insert_resource(BibleGraphNativeRenderConfig::default());
        app.insert_resource(BibleGraphNativeRendererWindowScene::default());
        app.insert_resource(BibleGraphNativeRendererWindowStatus::default());
        app.insert_resource(BibleGraphNativeVisualStatus::default());
        app.insert_resource(BibleGraphNativeWorkspaceTimelineVisualState::default());
        app.insert_resource(BibleGraphNativeAssetCache::default());
        app.insert_resource(BibleGraphNativeNodeTitleEdit::default());
        app.insert_resource(BibleGraphNativeTextEditorSettingsState::default());
        app.init_resource::<BibleGraphWorkspaceTimelinePresentation>();
        app.insert_resource(ClearColor(Color::srgb(0.067, 0.082, 0.114)));
        app.configure_sets(
            Update,
            (
                BibleGraphNativeRenderSystem::Projection,
                BibleGraphNativeRenderSystem::Input,
                BibleGraphNativeRenderSystem::Camera,
                BibleGraphNativeRenderSystem::Labels,
                BibleGraphNativeRenderSystem::Outlines,
            )
                .chain(),
        );
        app.add_systems(Startup, spawn_bible_graph_renderer_window_scene);
        app.add_systems(Startup, mark_bible_graph_native_window_ready);
        app.add_systems(
            Update,
            (
                apply_bible_graph_native_projection,
                apply_bible_graph_native_workspace_timeline_visual_snapshot,
            )
                .chain()
                .in_set(BibleGraphNativeRenderSystem::Projection),
        );
        app.add_systems(
            Update,
            (
                emit_bible_graph_native_click_selection,
                handle_bible_graph_native_title_edit_input,
                handle_bible_graph_native_node_text_editor_click,
                handle_bible_graph_native_node_text_editor_input,
                scroll_bible_graph_native_node_text_editor,
                emit_bible_graph_native_keyboard_commands,
            )
                .chain()
                .in_set(BibleGraphNativeRenderSystem::Input),
        );
        app.add_systems(
            Update,
            (
                navigate_bible_graph_native_camera,
                edge_pan_bible_graph_native_camera,
                drag_bible_graph_native_camera,
                scroll_bible_graph_native_camera,
                orbit_bible_graph_native_camera,
                recover_bible_graph_native_camera,
                frame_bible_graph_native_camera_on_selected,
                apply_bible_graph_native_camera_commands,
                position_bible_graph_native_workspace_timeline_panel,
            )
                .in_set(BibleGraphNativeRenderSystem::Camera),
        );
        app.add_systems(
            Update,
            (
                project_bible_graph_native_labels,
                apply_bible_graph_native_title_edit_overlay,
                apply_bible_graph_native_text_editor_settings,
                apply_bible_graph_native_node_text_editor_caret,
                emit_bible_graph_native_node_text_editor_updates,
            )
                .chain()
                .in_set(BibleGraphNativeRenderSystem::Labels),
        );
        app.add_systems(
            Update,
            billboard_bible_graph_native_selection_outlines
                .in_set(BibleGraphNativeRenderSystem::Outlines),
        );
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
    mut meshes: ResMut<Assets<Mesh>>,
    mut materials: ResMut<Assets<BibleGraphNativeMaterial>>,
) {
    commands.spawn((
        Camera3d::default(),
        Transform::from_xyz(0.0, 0.0, 900.0).looking_at(Vec3::ZERO, Vec3::Y),
        BibleGraphNativeCamera,
    ));
    let label_camera_entity = commands
        .spawn((
            Camera2d,
            Camera {
                order: 1,
                clear_color: ClearColorConfig::None,
                ..Default::default()
            },
            IsDefaultUiCamera,
            BibleGraphNativeLabelOverlayCamera,
        ))
        .id();
    commands.insert_resource(BibleGraphNativeLabelOverlayTarget {
        camera_entity: label_camera_entity,
    });
    commands.spawn((PointLight::default(), Transform::from_xyz(0.0, 0.0, 700.0)));
    commands.spawn((
        BibleGraphNativeWorkspaceTimelineRoot,
        Mesh3d(meshes.add(Cuboid::new(
            BIBLE_GRAPH_WORKSPACE_TIMELINE_PANEL_WIDTH,
            BIBLE_GRAPH_WORKSPACE_TIMELINE_PANEL_HEIGHT,
            BIBLE_GRAPH_WORKSPACE_TIMELINE_PANEL_DEPTH,
        ))),
        MeshMaterial3d(materials.add(BibleGraphNativeMaterial {
            color: LinearRgba::new(0.08, 0.1, 0.14, 0.92),
        })),
        Transform::from_xyz(0.0, -280.0, 120.0),
    ));
    status.camera_count = 2;
}

fn position_bible_graph_native_workspace_timeline_panel(
    presentation: Res<BibleGraphWorkspaceTimelinePresentation>,
    cameras: Query<&Transform, With<BibleGraphNativeCamera>>,
    mut panels: Query<
        &mut Transform,
        (
            With<BibleGraphNativeWorkspaceTimelineRoot>,
            Without<BibleGraphNativeCamera>,
        ),
    >,
) {
    let Ok(camera_transform) = cameras.single() else {
        return;
    };
    let Some(panel_transform) =
        native_workspace_timeline_panel_transform(camera_transform, &presentation)
    else {
        return;
    };

    for mut transform in &mut panels {
        *transform = panel_transform;
    }
}

pub fn native_workspace_timeline_panel_transform(
    camera_transform: &Transform,
    presentation: &BibleGraphWorkspaceTimelinePresentation,
) -> Option<Transform> {
    if matches!(
        presentation.mode,
        BibleGraphWorkspaceTimelinePresentationMode::WorldAnchoredTimeline
    ) {
        return None;
    }

    let forward = camera_transform.rotation * Vec3::NEG_Z;
    let right = camera_transform.rotation * Vec3::X;
    let up = camera_transform.rotation * Vec3::Y;
    let translation = camera_transform.translation
        + forward * presentation.camera_distance
        + right * (presentation.viewport_offset_x * presentation.camera_distance)
        + up * (presentation.viewport_offset_y * presentation.camera_distance);

    Some(Transform::from_translation(translation).looking_at(camera_transform.translation, up))
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

#[allow(clippy::too_many_arguments)]
fn emit_bible_graph_native_click_selection(
    buttons: Option<Res<ButtonInput<MouseButton>>>,
    time: Option<Res<Time>>,
    windows: Query<&Window, With<PrimaryWindow>>,
    cameras: Query<(&Camera, &GlobalTransform), With<BibleGraphNativeCamera>>,
    nodes: Query<&BibleGraphNativeNodeVisual>,
    edges: Query<&BibleGraphNativeEdgeVisual>,
    labels: Query<&BibleGraphNativeNodeLabelVisual>,
    editors: Query<&ScrollPosition, With<BibleGraphNativeNodeTextEditorVisual>>,
    text_editor_settings: Option<Res<BibleGraphNativeTextEditorSettingsState>>,
    mut title_edit: ResMut<BibleGraphNativeNodeTitleEdit>,
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
    if editors.iter().any(|scroll_position| {
        bible_graph_native_text_editor_local_position(
            cursor_position,
            Vec2::new(window.width(), window.height()),
            scroll_position.0.y,
            text_editor_settings
                .as_deref()
                .map(|state| state.settings.padding_px)
                .unwrap_or(BibleGraphNativeTextEditorSettings::default().padding_px),
        )
        .is_some()
    }) {
        return;
    }
    let Ok((camera, camera_transform)) = cameras.single() else {
        return;
    };
    let Ok(ray) = camera.viewport_to_world(camera_transform, cursor_position) else {
        return;
    };

    let command = bible_graph_native_click_command(nodes.iter(), edges.iter(), ray);
    if let BibleGraphRendererCommand::SelectNode { node_id } = &command {
        let Some(time) = time else {
            return;
        };
        let label = labels
            .iter()
            .find(|label| &label.node_id == node_id)
            .map(|label| label.label.clone())
            .unwrap_or_else(|| node_id.as_str().to_string());
        bible_graph_native_title_edit_register_node_click(
            &mut title_edit,
            node_id.clone(),
            label,
            time.elapsed_secs_f64(),
        );
    } else {
        bible_graph_native_title_edit_cancel(&mut title_edit);
    }
    let _ = push_native_command_to_control(&control, command);
}

pub(crate) fn bible_graph_native_click_command<'a>(
    nodes: impl Iterator<Item = &'a BibleGraphNativeNodeVisual>,
    edges: impl Iterator<Item = &'a BibleGraphNativeEdgeVisual>,
    ray: Ray3d,
) -> BibleGraphRendererCommand {
    if let Some(node_id) = nearest_native_node_on_ray(nodes, ray) {
        return BibleGraphRendererCommand::SelectNode { node_id };
    }

    if let Some(edge_id) = nearest_selectable_native_edge_on_ray(edges, ray) {
        return BibleGraphRendererCommand::SelectEdge { edge_id };
    }

    BibleGraphRendererCommand::ClearSelection
}

pub(crate) fn bible_graph_native_title_edit_register_node_click(
    title_edit: &mut BibleGraphNativeNodeTitleEdit,
    node_id: BibleGraphNodeId,
    label: String,
    at_seconds: f64,
) {
    let double_click = title_edit.last_click.as_ref().is_some_and(|last_click| {
        let elapsed_seconds = at_seconds - last_click.at_seconds;
        last_click.node_id == node_id
            && (0.0..=NATIVE_NODE_TITLE_DOUBLE_CLICK_SECONDS).contains(&elapsed_seconds)
    });

    if double_click {
        title_edit.active = Some(BibleGraphNativeNodeTitleEditState {
            node_id,
            value: label,
        });
        title_edit.last_click = None;
        return;
    }

    title_edit.active = None;
    title_edit.last_click = Some(BibleGraphNativeNodeClick {
        node_id,
        at_seconds,
    });
}

pub(crate) fn bible_graph_native_title_edit_append_text(
    title_edit: &mut BibleGraphNativeNodeTitleEdit,
    text: &str,
) {
    let Some(active) = title_edit.active.as_mut() else {
        return;
    };

    active
        .value
        .extend(text.chars().filter(|character| !character.is_control()));
}

pub(crate) fn bible_graph_native_title_edit_backspace(
    title_edit: &mut BibleGraphNativeNodeTitleEdit,
) {
    let Some(active) = title_edit.active.as_mut() else {
        return;
    };

    active.value.pop();
}

pub(crate) fn bible_graph_native_title_edit_commit(
    title_edit: &mut BibleGraphNativeNodeTitleEdit,
) -> Option<BibleGraphRendererCommand> {
    let active = title_edit.active.take()?;
    title_edit.last_click = None;
    let name = active.value.trim().to_string();
    if name.is_empty() {
        return None;
    }

    Some(BibleGraphRendererCommand::SetNodeName {
        node_id: active.node_id,
        name,
    })
}

pub(crate) fn bible_graph_native_title_edit_cancel(title_edit: &mut BibleGraphNativeNodeTitleEdit) {
    title_edit.active = None;
    title_edit.last_click = None;
}

fn bible_graph_native_title_edit_is_active(
    title_edit: Option<&BibleGraphNativeNodeTitleEdit>,
) -> bool {
    title_edit.is_some_and(|title_edit| title_edit.active.is_some())
}

fn bible_graph_native_text_editor_is_active(
    editor: &Query<Entity, With<BibleGraphNativeNodeTextEditorVisual>>,
) -> bool {
    !editor.is_empty()
}

fn emit_bible_graph_native_keyboard_commands(
    keys: Option<Res<ButtonInput<KeyCode>>>,
    nodes: Query<&BibleGraphNativeNodeVisual>,
    editor: Query<Entity, With<BibleGraphNativeNodeTextEditorVisual>>,
    title_edit: Option<Res<BibleGraphNativeNodeTitleEdit>>,
    control: Option<Res<BibleGraphNativeWindowControl>>,
) {
    let Some(keys) = keys else {
        return;
    };
    if bible_graph_native_title_edit_is_active(title_edit.as_deref()) {
        return;
    }
    let Some(control) = control else {
        return;
    };
    let text_editor_active = bible_graph_native_text_editor_is_active(&editor);

    if keys.just_pressed(KeyCode::Escape) {
        let _ = push_native_command_to_control(&control, BibleGraphRendererCommand::ClearSelection);
    }
    if keys.just_pressed(KeyCode::Delete)
        || (!text_editor_active && keys.just_pressed(KeyCode::Backspace))
    {
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

fn handle_bible_graph_native_title_edit_input(
    mut key_events: Option<MessageReader<KeyboardInput>>,
    mut title_edit: ResMut<BibleGraphNativeNodeTitleEdit>,
    control: Option<Res<BibleGraphNativeWindowControl>>,
) {
    if !bible_graph_native_title_edit_is_active(Some(&*title_edit)) {
        return;
    }
    let Some(mut key_events) = key_events.take() else {
        return;
    };

    for event in key_events.read() {
        if event.state != ButtonState::Pressed {
            continue;
        }

        match event.key_code {
            KeyCode::Escape => {
                bible_graph_native_title_edit_cancel(&mut title_edit);
                continue;
            }
            KeyCode::Enter | KeyCode::NumpadEnter => {
                if let Some(command) = bible_graph_native_title_edit_commit(&mut title_edit)
                    && let Some(control) = control.as_deref()
                {
                    let _ = push_native_command_to_control(control, command);
                }
                continue;
            }
            KeyCode::Backspace => {
                bible_graph_native_title_edit_backspace(&mut title_edit);
                continue;
            }
            _ => {}
        }

        if let Some(text) = event.text.as_deref() {
            bible_graph_native_title_edit_append_text(&mut title_edit, text);
        }
    }
}

fn handle_bible_graph_native_node_text_editor_input(
    mut key_events: Option<MessageReader<KeyboardInput>>,
    title_edit: Option<Res<BibleGraphNativeNodeTitleEdit>>,
    mut editors: Query<&mut BibleGraphNativeNodeTextEditorVisual>,
    mut texts: Query<&mut Text, With<BibleGraphNativeNodeTextEditorText>>,
) {
    if bible_graph_native_title_edit_is_active(title_edit.as_deref()) {
        return;
    }
    let Some(mut key_events) = key_events.take() else {
        return;
    };
    let Ok(mut editor) = editors.single_mut() else {
        return;
    };
    let Ok(mut text) = texts.single_mut() else {
        return;
    };

    for event in key_events.read() {
        if event.state != ButtonState::Pressed {
            continue;
        }

        match event.key_code {
            KeyCode::Enter | KeyCode::NumpadEnter => {
                let cursor_index = editor.cursor_byte_index;
                text.0.insert(cursor_index, '\n');
                editor.cursor_byte_index = cursor_index + '\n'.len_utf8();
            }
            KeyCode::Backspace => {
                editor.cursor_byte_index = bible_graph_native_text_editor_delete_backward(
                    &mut text.0,
                    editor.cursor_byte_index,
                );
            }
            KeyCode::ArrowLeft => {
                editor.cursor_byte_index =
                    bible_graph_native_text_editor_move_left(&text.0, editor.cursor_byte_index);
            }
            KeyCode::ArrowRight => {
                editor.cursor_byte_index =
                    bible_graph_native_text_editor_move_right(&text.0, editor.cursor_byte_index);
            }
            KeyCode::ArrowUp => {
                editor.cursor_byte_index = bible_graph_native_text_editor_move_vertical(
                    &text.0,
                    editor.cursor_byte_index,
                    -1,
                );
            }
            KeyCode::ArrowDown => {
                editor.cursor_byte_index = bible_graph_native_text_editor_move_vertical(
                    &text.0,
                    editor.cursor_byte_index,
                    1,
                );
            }
            _ => {
                if let Some(input_text) = event.text.as_deref() {
                    for character in input_text.chars().filter(|character| {
                        *character == '\n' || *character == '\t' || !character.is_control()
                    }) {
                        let cursor_index = editor.cursor_byte_index;
                        text.0.insert(cursor_index, character);
                        editor.cursor_byte_index = cursor_index + character.len_utf8();
                    }
                }
            }
        }
    }
}

fn handle_bible_graph_native_node_text_editor_click(
    buttons: Option<Res<ButtonInput<MouseButton>>>,
    windows: Query<&Window, With<PrimaryWindow>>,
    text_editor_settings: Option<Res<BibleGraphNativeTextEditorSettingsState>>,
    mut editors: Query<(&mut BibleGraphNativeNodeTextEditorVisual, &ScrollPosition)>,
    texts: Query<&Text, With<BibleGraphNativeNodeTextEditorText>>,
) {
    let Some(buttons) = buttons else {
        return;
    };
    if !buttons.just_pressed(MouseButton::Left) {
        return;
    }
    let Ok(window) = windows.single() else {
        return;
    };
    let Some(cursor_position) = window.cursor_position() else {
        return;
    };
    let Ok((mut editor, scroll_position)) = editors.single_mut() else {
        return;
    };
    let Ok(text) = texts.single() else {
        return;
    };
    let Some(local_position) = bible_graph_native_text_editor_local_position(
        cursor_position,
        Vec2::new(window.width(), window.height()),
        scroll_position.0.y,
        text_editor_settings
            .as_deref()
            .map(|state| state.settings.padding_px)
            .unwrap_or(BibleGraphNativeTextEditorSettings::default().padding_px),
    ) else {
        return;
    };
    editor.cursor_byte_index =
        bible_graph_native_text_editor_index_from_position(&text.0, local_position);
}

fn scroll_bible_graph_native_node_text_editor(
    mut scroll_events: Option<MessageReader<MouseWheel>>,
    mut editors: Query<&mut ScrollPosition, With<BibleGraphNativeNodeTextEditorVisual>>,
) {
    let Some(mut scroll_events) = scroll_events.take() else {
        return;
    };
    let Ok(mut scroll_position) = editors.single_mut() else {
        return;
    };

    let scroll_y = scroll_events.read().fold(0.0, |total, event| {
        let multiplier = match event.unit {
            MouseScrollUnit::Line => 20.0,
            MouseScrollUnit::Pixel => 1.0,
        };
        total + (event.y * multiplier)
    });
    if scroll_y == 0.0 {
        return;
    }

    scroll_position.0.y = (scroll_position.0.y - scroll_y).max(0.0);
}

fn navigate_bible_graph_native_camera(
    keys: Option<Res<ButtonInput<KeyCode>>>,
    time: Option<Res<Time>>,
    editor: Query<Entity, With<BibleGraphNativeNodeTextEditorVisual>>,
    title_edit: Option<Res<BibleGraphNativeNodeTitleEdit>>,
    mut cameras: Query<&mut Transform, With<BibleGraphNativeCamera>>,
) {
    let Some(keys) = keys else {
        return;
    };
    if bible_graph_native_title_edit_is_active(title_edit.as_deref()) {
        return;
    }
    if bible_graph_native_text_editor_is_active(&editor) {
        return;
    }
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

fn edge_pan_bible_graph_native_camera(
    buttons: Option<Res<ButtonInput<MouseButton>>>,
    time: Option<Res<Time>>,
    title_edit: Option<Res<BibleGraphNativeNodeTitleEdit>>,
    windows: Query<&Window, With<PrimaryWindow>>,
    mut cameras: Query<&mut Transform, With<BibleGraphNativeCamera>>,
) {
    if bible_graph_native_title_edit_is_active(title_edit.as_deref()) {
        return;
    }
    if buttons.as_ref().is_some_and(|buttons| {
        buttons.pressed(MouseButton::Left)
            || buttons.pressed(MouseButton::Middle)
            || buttons.pressed(MouseButton::Right)
    }) {
        return;
    }
    let Some(time) = time else {
        return;
    };
    let Ok(window) = windows.single() else {
        return;
    };
    let Some(cursor_position) = window.cursor_position() else {
        return;
    };
    let direction = native_camera_edge_pan_direction(
        cursor_position,
        Vec2::new(window.width().max(1.0), window.height().max(1.0)),
        NATIVE_CAMERA_EDGE_PAN_MARGIN_PX,
    );
    if direction == Vec2::ZERO {
        return;
    }
    let Ok(mut camera_transform) = cameras.single_mut() else {
        return;
    };

    let pan_delta = native_camera_edge_pan_delta(direction, &camera_transform, time.delta_secs());
    camera_transform.translation += pan_delta;
}

fn frame_bible_graph_native_camera_on_selected(
    keys: Option<Res<ButtonInput<KeyCode>>>,
    nodes: Query<&BibleGraphNativeNodeVisual>,
    editor: Query<Entity, With<BibleGraphNativeNodeTextEditorVisual>>,
    title_edit: Option<Res<BibleGraphNativeNodeTitleEdit>>,
    mut cameras: Query<&mut Transform, With<BibleGraphNativeCamera>>,
) {
    let Some(keys) = keys else {
        return;
    };
    if bible_graph_native_title_edit_is_active(title_edit.as_deref()) {
        return;
    }
    if bible_graph_native_text_editor_is_active(&editor) {
        return;
    }
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
    editor: Query<Entity, With<BibleGraphNativeNodeTextEditorVisual>>,
    mut cameras: Query<&mut Transform, With<BibleGraphNativeCamera>>,
) {
    if bible_graph_native_text_editor_is_active(&editor) {
        return;
    }
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
    editor: Query<Entity, With<BibleGraphNativeNodeTextEditorVisual>>,
    title_edit: Option<Res<BibleGraphNativeNodeTitleEdit>>,
    mut cameras: Query<&mut Transform, With<BibleGraphNativeCamera>>,
) {
    let Some(keys) = keys else {
        return;
    };
    if bible_graph_native_title_edit_is_active(title_edit.as_deref()) {
        return;
    }
    if bible_graph_native_text_editor_is_active(&editor) {
        return;
    }
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
    editor: Query<Entity, With<BibleGraphNativeNodeTextEditorVisual>>,
    title_edit: Option<Res<BibleGraphNativeNodeTitleEdit>>,
    mut cameras: Query<&mut Transform, With<BibleGraphNativeCamera>>,
) {
    let Some(keys) = keys else {
        return;
    };
    if bible_graph_native_title_edit_is_active(title_edit.as_deref()) {
        return;
    }
    if bible_graph_native_text_editor_is_active(&editor) {
        return;
    }
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

pub(crate) fn native_camera_edge_pan_direction(
    cursor_position: Vec2,
    viewport_size: Vec2,
    margin_px: f32,
) -> Vec2 {
    if viewport_size.x <= 0.0 || viewport_size.y <= 0.0 || margin_px <= 0.0 {
        return Vec2::ZERO;
    }

    let margin_x = margin_px.min(viewport_size.x * 0.5);
    let margin_y = margin_px.min(viewport_size.y * 0.5);
    let mut direction = Vec2::ZERO;
    if cursor_position.x <= margin_x {
        direction.x -= edge_pan_pressure(cursor_position.x, margin_x);
    } else if cursor_position.x >= viewport_size.x - margin_x {
        direction.x += edge_pan_pressure(viewport_size.x - cursor_position.x, margin_x);
    }
    if cursor_position.y <= margin_y {
        direction.y += edge_pan_pressure(cursor_position.y, margin_y);
    } else if cursor_position.y >= viewport_size.y - margin_y {
        direction.y -= edge_pan_pressure(viewport_size.y - cursor_position.y, margin_y);
    }

    direction.clamp_length_max(1.0)
}

fn edge_pan_pressure(distance_from_edge: f32, margin_px: f32) -> f32 {
    ((margin_px - distance_from_edge.max(0.0)) / margin_px).clamp(0.0, 1.0)
}

pub(crate) fn native_camera_edge_pan_delta(
    direction: Vec2,
    camera_transform: &Transform,
    delta_seconds: f32,
) -> Vec3 {
    if direction == Vec2::ZERO || delta_seconds <= 0.0 {
        return Vec3::ZERO;
    }

    let scale = native_camera_drag_pan_scale(camera_transform);
    let speed = NATIVE_CAMERA_EDGE_PAN_SPEED * scale * delta_seconds;
    let right = *camera_transform.right();
    let up = *camera_transform.up();
    right * direction.x * speed + up * direction.y * speed
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

fn project_bible_graph_native_labels(
    cameras: Query<(&Camera, &Transform), With<BibleGraphNativeCamera>>,
    windows: Query<&Window, With<PrimaryWindow>>,
    text_editor_settings: Option<Res<BibleGraphNativeTextEditorSettingsState>>,
    mut labels: Query<BibleGraphNativeLabelProjectionItem, BibleGraphNativeLabelProjectionFilter>,
) {
    let Ok((camera, camera_transform)) = cameras.single() else {
        return;
    };
    let Ok(window) = windows.single() else {
        return;
    };

    let viewport_size = Vec2::new(window.width(), window.height());
    let camera_global_transform = GlobalTransform::from(*camera_transform);
    let label_size_scale = label_size_scale_from_settings(text_editor_settings.as_deref());
    for (label, mut label_node, mut label_font, mut visibility) in &mut labels {
        if !label.label_visible {
            *visibility = Visibility::Hidden;
            continue;
        }

        let world_position = Vec3::new(label.x, label.y, label.z);
        let Ok(viewport_position) =
            camera.world_to_viewport(&camera_global_transform, world_position)
        else {
            *visibility = Visibility::Hidden;
            continue;
        };

        let projected_radius = native_node_label_projected_radius(
            camera,
            &camera_global_transform,
            camera_transform,
            world_position,
            label.radius,
        )
        .unwrap_or(label.radius)
        .max(1.0);
        let label_position =
            native_node_label_overlay_position(viewport_position, viewport_size, projected_radius);
        label_node.left = Val::Px(label_position.x);
        label_node.top = Val::Px(label_position.y);
        label_font.font_size = native_node_label_projected_font_size(
            projected_radius,
            label.label_font_size,
            label_size_scale,
        );
        *visibility = Visibility::Visible;
    }
}

fn label_size_scale_from_settings(
    text_editor_settings: Option<&BibleGraphNativeTextEditorSettingsState>,
) -> f32 {
    text_editor_settings
        .map(|state| state.settings.label_size_scale)
        .unwrap_or(BibleGraphNativeTextEditorSettings::default().label_size_scale)
        .clamp(0.25, 3.0)
}

fn apply_bible_graph_native_title_edit_overlay(
    title_edit: Option<Res<BibleGraphNativeNodeTitleEdit>>,
    mut labels: Query<(&BibleGraphNativeNodeLabelVisual, &mut Text)>,
) {
    let active = title_edit.as_deref().and_then(|title_edit| {
        title_edit
            .active
            .as_ref()
            .map(|active| (&active.node_id, active.value.as_str()))
    });

    for (label, mut text) in &mut labels {
        let next_text = active
            .filter(|(node_id, _)| *node_id == &label.node_id)
            .map(|(_, value)| format!("{value}|"))
            .unwrap_or_else(|| label.label.clone());
        if text.0 != next_text {
            text.0 = next_text;
        }
    }
}

fn emit_bible_graph_native_node_text_editor_updates(
    time: Option<Res<Time>>,
    control: Option<Res<BibleGraphNativeWindowControl>>,
    mut editors: Query<&mut BibleGraphNativeNodeTextEditorVisual>,
    texts: Query<&Text, With<BibleGraphNativeNodeTextEditorText>>,
) {
    let Some(time) = time else {
        return;
    };
    let Some(control) = control else {
        return;
    };
    let now_seconds = time.elapsed_secs_f64();

    let Ok(editable_text) = texts.single() else {
        return;
    };

    for mut editor in &mut editors {
        let current_text = editable_text.0.clone();
        if current_text != editor.last_seen_text {
            editor.last_seen_text = current_text;
            editor.dirty_since_seconds = Some(now_seconds);
        }

        let Some(dirty_since_seconds) = editor.dirty_since_seconds else {
            continue;
        };
        if now_seconds - dirty_since_seconds < NATIVE_NODE_TEXT_SAVE_DEBOUNCE_SECONDS {
            continue;
        }
        if editor.last_seen_text == editor.last_sent_text {
            editor.dirty_since_seconds = None;
            continue;
        }

        let command = BibleGraphRendererCommand::SetNodeText {
            node_id: editor.node_id.clone(),
            text: editor.last_seen_text.clone(),
        };
        if push_native_command_to_control(&control, command).is_ok() {
            editor.last_sent_text = editor.last_seen_text.clone();
            editor.dirty_since_seconds = None;
        }
    }
}

#[allow(clippy::too_many_arguments)]
fn apply_bible_graph_native_text_editor_settings(
    control: Option<Res<BibleGraphNativeWindowControl>>,
    mut state: ResMut<BibleGraphNativeTextEditorSettingsState>,
    mut editors: Query<
        (&mut Node, &mut BorderColor, &mut BackgroundColor),
        With<BibleGraphNativeNodeTextEditorVisual>,
    >,
    mut texts: Query<(&mut TextFont, &mut TextColor), With<BibleGraphNativeNodeTextEditorText>>,
    mut selection_outlines: Query<(
        &mut BibleGraphNativeSelectionOutlineVisual,
        &mut Mesh3d,
        &mut MeshMaterial3d<BibleGraphNativeMaterial>,
    )>,
    mut meshes: ResMut<Assets<Mesh>>,
    mut materials: ResMut<Assets<BibleGraphNativeMaterial>>,
    mut asset_cache: ResMut<BibleGraphNativeAssetCache>,
) {
    let Some(control) = control else {
        return;
    };
    let version = control.text_editor_settings_version.load(Ordering::Acquire);
    if version == state.version {
        return;
    }
    let settings = control
        .text_editor_settings
        .lock()
        .unwrap_or_else(|error| error.into_inner())
        .clone();
    state.settings = settings.clone();
    state.version = version;

    for (mut node, mut border_color, mut background_color) in &mut editors {
        apply_bible_graph_native_text_editor_node_settings(&mut node, &settings);
        *border_color = native_text_editor_border_color(&settings);
        *background_color = native_text_editor_background_color(&settings);
    }

    for (mut font, mut color) in &mut texts {
        font.font_size = settings.font_size_px.max(1.0);
        *color = native_text_editor_font_color(&settings);
    }

    for (mut outline, mut mesh, mut material) in &mut selection_outlines {
        outline.outline_width_px = settings.selected_node_outline_width_px.max(1.0);
        outline.outline_color = settings.selected_node_outline_color.clone();
        *mesh = Mesh3d(cached_native_selection_outline_mesh_from_assets(
            &mut meshes,
            &mut asset_cache,
            outline.radius,
            settings.selected_node_outline_width_px,
        ));
        *material = MeshMaterial3d(cached_native_material_from_assets(
            &mut materials,
            &mut asset_cache,
            &settings.selected_node_outline_color,
            false,
            false,
            false,
            settings.selected_node_outline_brightness,
        ));
    }
}

fn apply_bible_graph_native_node_text_editor_caret(
    editors: Query<(&BibleGraphNativeNodeTextEditorVisual, &ScrollPosition)>,
    texts: Query<&Text, With<BibleGraphNativeNodeTextEditorText>>,
    text_editor_settings: Option<Res<BibleGraphNativeTextEditorSettingsState>>,
    mut carets: Query<(
        &BibleGraphNativeNodeTextEditorCaret,
        &mut Node,
        &mut Visibility,
    )>,
) {
    let Ok((editor, scroll_position)) = editors.single() else {
        return;
    };
    let Ok(text) = texts.single() else {
        return;
    };
    let padding_px = text_editor_settings
        .as_deref()
        .map(|state| state.settings.padding_px)
        .unwrap_or(BibleGraphNativeTextEditorSettings::default().padding_px);
    let caret_position = bible_graph_native_text_editor_caret_position(
        &text.0,
        editor.cursor_byte_index,
        scroll_position.0.y,
    );

    for (caret, mut node, mut visibility) in &mut carets {
        if caret.node_id != editor.node_id {
            *visibility = Visibility::Hidden;
            continue;
        }
        node.left = Val::Px(padding_px + caret_position.x);
        node.top = Val::Px(padding_px + caret_position.y);
        *visibility = Visibility::Visible;
    }
}

fn apply_bible_graph_native_text_editor_node_settings(
    node: &mut Node,
    settings: &BibleGraphNativeTextEditorSettings,
) {
    node.padding = UiRect::all(Val::Px(settings.padding_px.max(0.0)));
    node.border = UiRect::all(Val::Px(settings.editor_outline_width_px.max(0.0)));
    node.border_radius = BorderRadius::all(Val::Px(settings.corner_radius_px.max(0.0)));
}

fn native_text_editor_border_color(settings: &BibleGraphNativeTextEditorSettings) -> BorderColor {
    let brightness = settings.editor_outline_brightness.clamp(0.0, 1.0);
    let alpha = 1.0 - settings.editor_outline_transparency.clamp(0.0, 1.0);
    BorderColor::all(Color::srgba(brightness, brightness, brightness, alpha))
}

fn native_text_editor_background_color(
    settings: &BibleGraphNativeTextEditorSettings,
) -> BackgroundColor {
    let (red, green, blue) = graph_color_components(&settings.editor_background_color);
    let brightness = settings.editor_background_brightness.clamp(0.0, 1.0);
    let alpha = 1.0 - settings.editor_background_transparency.clamp(0.0, 1.0);
    BackgroundColor(Color::srgba(
        red * brightness,
        green * brightness,
        blue * brightness,
        alpha,
    ))
}

fn native_text_editor_font_color(settings: &BibleGraphNativeTextEditorSettings) -> TextColor {
    let brightness = settings.font_brightness.clamp(0.0, 1.0);
    TextColor(Color::srgb(brightness, brightness, brightness))
}

pub(crate) fn native_node_label_overlay_position(
    viewport_position: Vec2,
    _viewport_size: Vec2,
    projected_node_radius: f32,
) -> Vec2 {
    let radius = projected_node_radius.max(1.0);
    Vec2::new(
        viewport_position.x,
        viewport_position.y - radius - NATIVE_LABEL_SCREEN_OFFSET_PX,
    )
}

fn native_node_label_projected_radius(
    camera: &Camera,
    camera_global_transform: &GlobalTransform,
    camera_transform: &Transform,
    world_position: Vec3,
    node_radius: f32,
) -> Option<f32> {
    let radius_sample = world_position + *camera_transform.right() * node_radius.max(1.0);
    let center = camera
        .world_to_viewport(camera_global_transform, world_position)
        .ok()?;
    let edge = camera
        .world_to_viewport(camera_global_transform, radius_sample)
        .ok()?;
    Some(center.distance(edge))
}

pub(crate) fn native_node_label_projected_font_size(
    projected_node_radius: f32,
    base_font_size: f32,
    label_size_scale: f32,
) -> f32 {
    let scaled_size = projected_node_radius.max(1.0) * 0.44 * label_size_scale.clamp(0.25, 3.0);
    scaled_size.clamp(5.0, base_font_size.max(5.0) * 1.8)
}

fn billboard_bible_graph_native_selection_outlines(
    cameras: Query<&Transform, With<BibleGraphNativeCamera>>,
    mut outlines: Query<
        &mut Transform,
        (
            With<BibleGraphNativeSelectionOutlineBillboard>,
            Without<BibleGraphNativeCamera>,
        ),
    >,
) {
    let Ok(camera_transform) = cameras.single() else {
        return;
    };

    let rotation = native_selection_outline_billboard_rotation(camera_transform);
    for mut outline_transform in &mut outlines {
        outline_transform.rotation = rotation;
    }
}

pub(crate) fn native_selection_outline_billboard_rotation(camera_transform: &Transform) -> Quat {
    Quat::from_rotation_arc(Vec3::Y, -*camera_transform.forward())
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

fn apply_bible_graph_native_workspace_timeline_visual_snapshot(world: &mut World) {
    let Some(control) = world
        .get_resource::<BibleGraphNativeWindowControl>()
        .cloned()
    else {
        return;
    };
    if let Some(snapshot) = control.take_workspace_timeline_visual_snapshot() {
        rebuild_bible_graph_native_workspace_timeline_visuals(world, &snapshot);
        let mut state = world.resource_mut::<BibleGraphNativeWorkspaceTimelineVisualState>();
        state.snapshot = Some(snapshot);
    }
}

pub fn rebuild_bible_graph_native_workspace_timeline_visuals(
    world: &mut World,
    snapshot: &BibleGraphWorkspaceTimelineVisualSnapshot,
) {
    if !world.contains_resource::<Assets<Mesh>>()
        || !world.contains_resource::<Assets<BibleGraphNativeMaterial>>()
    {
        return;
    }

    let existing = world
        .query_filtered::<Entity, With<BibleGraphNativeWorkspaceTimelineVisualEntity>>()
        .iter(world)
        .collect::<Vec<_>>();
    for entity in existing {
        world.entity_mut(entity).despawn();
    }

    let Some(root_entity) = world
        .query_filtered::<Entity, With<BibleGraphNativeWorkspaceTimelineRoot>>()
        .iter(world)
        .next()
    else {
        return;
    };

    let mut child_entities = Vec::new();
    let track_width = (snapshot.panel_width - 48.0).max(1.0);
    for track in &snapshot.tracks {
        let mesh = world
            .resource_mut::<Assets<Mesh>>()
            .add(Cuboid::new(track_width, 1.5, 1.0));
        let material = world
            .resource_mut::<Assets<BibleGraphNativeMaterial>>()
            .add(BibleGraphNativeMaterial {
                color: LinearRgba::new(0.22, 0.28, 0.38, 0.72),
            });
        let entity = world
            .spawn((
                BibleGraphNativeWorkspaceTimelineVisualEntity,
                BibleGraphNativeWorkspaceTimelineTrackVisual {
                    track: track.clone(),
                },
                Mesh3d(mesh),
                MeshMaterial3d(material),
                Transform::from_xyz(
                    0.0,
                    track.y,
                    (BIBLE_GRAPH_WORKSPACE_TIMELINE_PANEL_DEPTH / 2.0) + 0.35,
                ),
            ))
            .id();
        child_entities.push(entity);
    }

    for clip in &snapshot.clips {
        let mesh =
            world
                .resource_mut::<Assets<Mesh>>()
                .add(Cuboid::new(clip.width, clip.height, 1.5));
        let material = world
            .resource_mut::<Assets<BibleGraphNativeMaterial>>()
            .add(BibleGraphNativeMaterial {
                color: LinearRgba::new(
                    clip.color_rgb[0],
                    clip.color_rgb[1],
                    clip.color_rgb[2],
                    0.96,
                ),
            });
        let entity = world
            .spawn((
                BibleGraphNativeWorkspaceTimelineVisualEntity,
                BibleGraphNativeWorkspaceTimelineClipVisual { clip: clip.clone() },
                Mesh3d(mesh),
                MeshMaterial3d(material),
                Transform::from_xyz(
                    clip.x,
                    clip.y,
                    (BIBLE_GRAPH_WORKSPACE_TIMELINE_PANEL_DEPTH / 2.0) + 1.0,
                ),
            ))
            .id();
        child_entities.push(entity);
    }
    for child_entity in child_entities {
        world.entity_mut(root_entity).add_child(child_entity);
    }
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
            1.0,
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
    let mut existing_selection_outlines = existing_native_selection_outlines(world);
    let label_target_camera = world
        .resource::<BibleGraphNativeLabelOverlayTarget>()
        .camera_entity;

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
            1.0,
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
                x: node.position.x,
                y: node.position.y,
                z: node.position.z,
                radius: node.radius,
                label_font_size: node.label_font_size,
                label_visible: node.label_visible,
            },
            Text::new(node_label),
            TextFont::from_font_size(node.label_font_size),
            TextColor(native_color_from_hex(node.label_color)),
            TextLayout::new_with_justify(Justify::Center),
            Node {
                position_type: PositionType::Absolute,
                left: Val::Px(node.position.x),
                top: Val::Px(node.position.y),
                ..Default::default()
            },
            UiTargetCamera(label_target_camera),
            BibleGraphNativeLabelBillboard,
            Visibility::Hidden,
        );
        if let Some(entity) = existing_node_labels.remove(&node_id) {
            world.entity_mut(entity).insert(label_bundle);
        } else {
            world.spawn(label_bundle);
        }

        if node.selected {
            let outline_settings = world
                .get_resource::<BibleGraphNativeTextEditorSettingsState>()
                .map(|state| state.settings.clone())
                .unwrap_or_default();
            let outline_mesh = cached_native_selection_outline_mesh(
                world,
                node.radius,
                outline_settings.selected_node_outline_width_px,
            );
            let outline_material = cached_native_material(
                world,
                &outline_settings.selected_node_outline_color,
                false,
                false,
                false,
                outline_settings.selected_node_outline_brightness,
            );
            let outline_bundle = (
                BibleGraphNativeVisualEntity,
                BibleGraphNativeSelectionOutlineVisual {
                    node_id: node_id.clone(),
                    radius: node.radius,
                    outline_width_px: outline_settings.selected_node_outline_width_px,
                    outline_color: outline_settings.selected_node_outline_color,
                },
                BibleGraphNativeSelectionOutlineBillboard,
                Mesh3d(outline_mesh),
                MeshMaterial3d(outline_material),
                Transform::from_translation(Vec3::new(
                    node.position.x,
                    node.position.y,
                    node.position.z + 4.0,
                ))
                .with_rotation(Quat::from_rotation_arc(Vec3::Y, Vec3::Z)),
            );
            if let Some(entity) = existing_selection_outlines.remove(&node_id) {
                world.entity_mut(entity).insert(outline_bundle);
            } else {
                world.spawn(outline_bundle);
            }
        }
    }
    despawn_remaining_entities(world, existing_nodes);
    despawn_remaining_entities(world, existing_node_labels);
    despawn_remaining_entities(world, existing_selection_outlines);

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
    rebuild_bible_graph_native_node_text_editor(world, projection);

    let mut status = world.resource_mut::<BibleGraphNativeVisualStatus>();
    status.node_count = node_count;
    status.edge_count = edge_count;
}

fn rebuild_bible_graph_native_node_text_editor(
    world: &mut World,
    projection: &BibleRenderGraphProjection,
) {
    let existing_editors = existing_native_node_text_editor_entities(world);
    despawn_entities(world, existing_editors);

    let Some(selected_node_id) = projection.selected_node_id.as_ref() else {
        return;
    };
    let Some(selected_node) = projection
        .nodes
        .iter()
        .find(|node| &node.node_id == selected_node_id)
    else {
        return;
    };
    let Some(label_target) = world.get_resource::<BibleGraphNativeLabelOverlayTarget>() else {
        return;
    };
    let label_camera_entity = label_target.camera_entity;
    let text_editor_settings = world
        .get_resource::<BibleGraphNativeTextEditorSettingsState>()
        .map(|state| state.settings.clone())
        .unwrap_or_default();
    let text = selected_node.text_content.clone().unwrap_or_default();
    let mut editor_node = Node {
        position_type: PositionType::Absolute,
        top: Val::Px(NATIVE_NODE_TEXT_EDITOR_TOP_PX),
        right: Val::Px(NATIVE_NODE_TEXT_EDITOR_RIGHT_PX),
        width: Val::Px(NATIVE_NODE_TEXT_EDITOR_WIDTH_PX),
        height: Val::Percent(NATIVE_NODE_TEXT_EDITOR_HEIGHT_RATIO * 100.0),
        overflow: Overflow::scroll_y(),
        ..Default::default()
    };
    apply_bible_graph_native_text_editor_node_settings(&mut editor_node, &text_editor_settings);

    let editor_entity = world
        .spawn((
            BibleGraphNativeVisualEntity,
            BibleGraphNativeNodeTextEditorVisual {
                node_id: selected_node.node_id.clone(),
                cursor_byte_index: text.len(),
                last_seen_text: text.clone(),
                last_sent_text: text.clone(),
                dirty_since_seconds: None,
            },
            editor_node,
            native_text_editor_background_color(&text_editor_settings),
            native_text_editor_border_color(&text_editor_settings),
            ScrollPosition::default(),
            UiTargetCamera(label_camera_entity),
        ))
        .id();

    let text_entity = world
        .spawn((
            BibleGraphNativeVisualEntity,
            BibleGraphNativeNodeTextEditorText {
                node_id: selected_node.node_id.clone(),
            },
            Node {
                width: Val::Percent(100.0),
                ..Default::default()
            },
            Text::new(text),
            TextFont::from_font_size(text_editor_settings.font_size_px.max(1.0)),
            native_text_editor_font_color(&text_editor_settings),
            TextLayout::new_with_justify(Justify::Left),
            UiTargetCamera(label_camera_entity),
        ))
        .id();

    let caret_entity = world
        .spawn((
            BibleGraphNativeVisualEntity,
            BibleGraphNativeNodeTextEditorCaret {
                node_id: selected_node.node_id.clone(),
            },
            Node {
                position_type: PositionType::Absolute,
                left: Val::Px(0.0),
                top: Val::Px(0.0),
                width: Val::Px(NATIVE_NODE_TEXT_EDITOR_CARET_WIDTH_PX),
                height: Val::Px(NATIVE_NODE_TEXT_EDITOR_CARET_HEIGHT_PX),
                ..Default::default()
            },
            BackgroundColor(Color::srgb(0.94, 0.78, 0.35)),
            Visibility::Visible,
            UiTargetCamera(label_camera_entity),
        ))
        .id();
    world.entity_mut(editor_entity).add_child(text_entity);
    world.entity_mut(editor_entity).add_child(caret_entity);
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

fn cached_native_selection_outline_mesh(
    world: &mut World,
    radius: f32,
    width_px: f32,
) -> Handle<Mesh> {
    world.resource_scope(|world, mut asset_cache: Mut<BibleGraphNativeAssetCache>| {
        world.resource_scope(|_, mut meshes: Mut<Assets<Mesh>>| {
            cached_native_selection_outline_mesh_from_assets(
                &mut meshes,
                &mut asset_cache,
                radius,
                width_px,
            )
        })
    })
}

fn cached_native_selection_outline_mesh_from_assets(
    meshes: &mut Assets<Mesh>,
    asset_cache: &mut BibleGraphNativeAssetCache,
    radius: f32,
    width_px: f32,
) -> Handle<Mesh> {
    let width_px = width_px.max(1.0);
    let inner_radius = (radius + 3.0).max(1.0);
    let outer_radius = inner_radius + width_px;
    let key = (
        quantized_visual_scalar(radius),
        quantized_visual_scalar(width_px),
    );
    if let Some(handle) = asset_cache.selection_outline_meshes.get(&key).cloned() {
        return handle;
    }

    let handle = meshes.add(Torus::new(inner_radius, outer_radius));
    asset_cache
        .selection_outline_meshes
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
    color: &str,
    selected: bool,
    highlighted: bool,
    dimmed: bool,
    brightness: f32,
) -> Handle<BibleGraphNativeMaterial> {
    world.resource_scope(|world, mut asset_cache: Mut<BibleGraphNativeAssetCache>| {
        world.resource_scope(|_, mut materials: Mut<Assets<BibleGraphNativeMaterial>>| {
            cached_native_material_from_assets(
                &mut materials,
                &mut asset_cache,
                color,
                selected,
                highlighted,
                dimmed,
                brightness,
            )
        })
    })
}

fn cached_native_material_from_assets(
    materials: &mut Assets<BibleGraphNativeMaterial>,
    asset_cache: &mut BibleGraphNativeAssetCache,
    color: &str,
    selected: bool,
    highlighted: bool,
    dimmed: bool,
    brightness: f32,
) -> Handle<BibleGraphNativeMaterial> {
    let key = BibleGraphNativeMaterialKey {
        color: color.to_string(),
        selected,
        highlighted,
        dimmed,
        brightness: quantized_visual_scalar(brightness.clamp(0.0, 1.0)),
    };
    if let Some(handle) = asset_cache.materials.get(&key).cloned() {
        return handle;
    }

    let handle = materials.add(native_graph_material(
        color,
        selected,
        highlighted,
        dimmed,
        brightness,
    ));
    asset_cache.materials.insert(key, handle.clone());
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

pub fn emit_bible_graph_native_node_name_set(
    world: &mut World,
    node_id: BibleGraphNodeId,
    name: String,
) -> Result<(), BibleGraphRendererError> {
    validate_native_node(world, &node_id)?;
    push_native_command(
        world,
        BibleGraphRendererCommand::SetNodeName { node_id, name },
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
        return Color::srgb(red * 0.56, green * 0.56, blue * 0.56);
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

pub(crate) fn native_graph_material(
    color: &str,
    selected: bool,
    highlighted: bool,
    dimmed: bool,
    brightness: f32,
) -> BibleGraphNativeMaterial {
    let color = native_visual_state_color(color, selected, highlighted, dimmed);
    let brightness = brightness.clamp(0.0, 1.0);
    let color = Color::srgb(
        graph_color_component(color, 0) * brightness,
        graph_color_component(color, 1) * brightness,
        graph_color_component(color, 2) * brightness,
    );
    BibleGraphNativeMaterial {
        color: color.to_linear(),
    }
}

fn graph_color_component(color: Color, index: usize) -> f32 {
    let color = color.to_srgba();
    match index {
        0 => color.red,
        1 => color.green,
        _ => color.blue,
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

fn existing_native_node_text_editor_entities(world: &mut World) -> Vec<Entity> {
    let mut entities = world
        .query::<(Entity, &BibleGraphNativeNodeTextEditorVisual)>()
        .iter(world)
        .map(|(entity, _)| entity)
        .collect::<Vec<_>>();
    entities.extend(
        world
            .query::<(Entity, &BibleGraphNativeNodeTextEditorCaret)>()
            .iter(world)
            .map(|(entity, _)| entity),
    );
    entities.extend(
        world
            .query::<(Entity, &BibleGraphNativeNodeTextEditorText)>()
            .iter(world)
            .map(|(entity, _)| entity),
    );
    entities
}

fn existing_native_selection_outlines(world: &mut World) -> HashMap<BibleGraphNodeId, Entity> {
    world
        .query::<(Entity, &BibleGraphNativeSelectionOutlineVisual)>()
        .iter(world)
        .map(|(entity, outline)| (outline.node_id.clone(), entity))
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

fn despawn_entities(world: &mut World, existing_entities: Vec<Entity>) {
    for entity in existing_entities {
        let _ = world.despawn(entity);
    }
}
