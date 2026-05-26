use std::num::NonZeroU64;
use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::{Arc, Mutex, mpsc};
use std::time::{Duration, Instant};

use bevy::app::AppExit;
use bevy::prelude::{
    App, ClearColor, Color, MessageWriter, Plugin, PluginGroup, Query, Res, Resource, Update,
    Window,
};
use bevy::window::{
    ExitCondition, PrimaryWindow, WindowCloseRequested, WindowPlugin, WindowResolution,
};
use bevy::winit::WinitPlugin;
use eidetic_core::contracts::TimelineRenderProjection;

use crate::scene::{TimelineSceneStats, rebuild_timeline_scene};

const TIMELINE_NATIVE_PROJECTION_QUEUE_CAPACITY: usize = 8;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Resource)]
pub struct TimelineNativeRenderConfig {
    pub borderless_window: bool,
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
}

#[derive(Debug, Clone, Resource)]
pub struct TimelineNativeWindowControl {
    shutdown_requested: Arc<AtomicBool>,
    show_requested: Arc<AtomicBool>,
    hide_requested: Arc<AtomicBool>,
    visible: Arc<AtomicBool>,
    ready: Arc<AtomicBool>,
    projection_receiver: Arc<Mutex<mpsc::Receiver<TimelineRenderProjection>>>,
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
        Self {
            shutdown_requested: Arc::new(AtomicBool::new(false)),
            show_requested: Arc::new(AtomicBool::new(false)),
            hide_requested: Arc::new(AtomicBool::new(false)),
            visible: Arc::new(AtomicBool::new(false)),
            ready: Arc::new(AtomicBool::new(false)),
            projection_sender,
            projection_receiver: Arc::new(Mutex::new(projection_receiver)),
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
        app.insert_resource(ClearColor(Color::srgb(0.067, 0.082, 0.114)));
        app.insert_resource(TimelineSceneStats::default());
        app.add_systems(Update, mark_timeline_native_window_ready);
        app.add_systems(Update, apply_timeline_native_projection_updates);
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
    rebuild_timeline_scene(app.world_mut(), projection);
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
        rebuild_timeline_scene(world, &projection);
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
