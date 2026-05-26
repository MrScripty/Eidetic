use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::{Arc, Mutex, mpsc};

use bevy::prelude::Resource;
use eidetic_core::contracts::TimelineRenderProjection;

use crate::TimelineRendererCommand;

const TIMELINE_NATIVE_PROJECTION_QUEUE_CAPACITY: usize = 8;
const TIMELINE_NATIVE_COMMAND_QUEUE_CAPACITY: usize = 128;

#[derive(Debug, Clone)]
pub struct TimelineNativeWindowControlHandle {
    pub(crate) shutdown_requested: Arc<AtomicBool>,
    pub(crate) show_requested: Arc<AtomicBool>,
    pub(crate) hide_requested: Arc<AtomicBool>,
    pub(crate) visible: Arc<AtomicBool>,
    pub(crate) ready: Arc<AtomicBool>,
    projection_sender: mpsc::SyncSender<TimelineRenderProjection>,
    pub(crate) projection_receiver: Arc<Mutex<mpsc::Receiver<TimelineRenderProjection>>>,
    pub(crate) command_sender: mpsc::SyncSender<TimelineRendererCommand>,
    command_receiver: Arc<Mutex<mpsc::Receiver<TimelineRendererCommand>>>,
}

#[derive(Debug, Clone, Resource)]
pub struct TimelineNativeWindowControl {
    pub(crate) shutdown_requested: Arc<AtomicBool>,
    pub(crate) show_requested: Arc<AtomicBool>,
    pub(crate) hide_requested: Arc<AtomicBool>,
    pub(crate) visible: Arc<AtomicBool>,
    pub(crate) ready: Arc<AtomicBool>,
    pub(crate) projection_receiver: Arc<Mutex<mpsc::Receiver<TimelineRenderProjection>>>,
    pub(crate) command_sender: mpsc::SyncSender<TimelineRendererCommand>,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum TimelineNativeWindowProjectionUpdateError {
    QueueFull,
    WindowClosed,
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

    pub(crate) fn enqueue_command(&self, command: TimelineRendererCommand) {
        let _ = self.command_sender.try_send(command);
    }
}
