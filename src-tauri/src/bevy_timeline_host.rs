use std::panic::{AssertUnwindSafe, catch_unwind};

use eidetic_bevy_timeline::{TimelineRendererApp, TimelineRendererCommand, TimelineRendererError};
use eidetic_core::contracts::TimelineRenderProjection;

use crate::renderer_window::{
    DesktopRendererRunnerLifecycle, DesktopRendererThreadingModel, DesktopRendererWindowCapability,
    DesktopRendererWindowCapabilityReason, DesktopRendererWindowKind,
    DesktopRendererWindowLifecycle, DesktopRendererWindowPlatform, DesktopRendererWindowStrategy,
    DesktopRendererWindowStrategyStatus,
};
use crate::timeline_renderer_supervisor::{
    TimelineRendererRunnerStatus, TimelineRendererSupervisor,
};

pub use crate::bevy_timeline_owner::{
    DesktopTimelineRendererOwner, TIMELINE_RENDERER_COMMAND_QUEUE_CAPACITY,
    TIMELINE_RENDERER_REPLY_TIMEOUT_MS,
};

#[derive(Debug, Clone, PartialEq, Eq, serde::Serialize)]
pub struct TimelineHostStatus {
    pub renderer_window_kind: DesktopRendererWindowKind,
    pub running: bool,
    pub renderer_window_open: bool,
    pub renderer_scene_ready: bool,
    pub renderer_window_lifecycle: DesktopRendererWindowLifecycle,
    pub renderer_runner_lifecycle: DesktopRendererRunnerLifecycle,
    pub renderer_runner_threading_model: DesktopRendererThreadingModel,
    pub renderer_window_strategy: DesktopRendererWindowStrategy,
    pub renderer_window_platform: DesktopRendererWindowPlatform,
    pub renderer_window_capability: DesktopRendererWindowCapability,
    pub renderer_window_capability_reason: DesktopRendererWindowCapabilityReason,
    pub renderer_window_visible: bool,
    pub renderer_window_ready: bool,
    pub renderer_window_verified_support: bool,
    pub renderer_window_visible_supported: bool,
    pub renderer_window_focus_supported: bool,
    pub renderer_window_message: String,
    pub track_count: usize,
    pub clip_count: usize,
    pub relationship_count: usize,
    pub affect_overlay_count: usize,
    pub queued_command_count: usize,
    pub last_error: Option<String>,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum TimelineHostError {
    Renderer(String),
    RendererPanic,
    QueueFull,
    OwnerReplyTimeout,
    OwnerStopped,
}

pub(crate) type TimelineHostResult<T> = Result<T, TimelineHostError>;

pub struct DesktopTimelineHost {
    renderer: Option<TimelineRendererApp>,
    native_window: TimelineRendererSupervisor,
    last_error: Option<String>,
}

impl Default for DesktopTimelineHost {
    fn default() -> Self {
        Self::new()
    }
}

impl DesktopTimelineHost {
    pub fn new() -> Self {
        Self {
            renderer: None,
            native_window: TimelineRendererSupervisor::current(),
            last_error: None,
        }
    }

    #[cfg(test)]
    pub(crate) fn with_native_window(native_window: TimelineRendererSupervisor) -> Self {
        Self {
            renderer: None,
            native_window,
            last_error: None,
        }
    }

    pub fn renderer_unavailable_status(message: String) -> TimelineHostStatus {
        let window_strategy = DesktopRendererWindowStrategyStatus::runner_error_current_platform();
        TimelineHostStatus {
            renderer_window_kind: DesktopRendererWindowKind::Timeline,
            running: false,
            renderer_window_open: false,
            renderer_scene_ready: false,
            renderer_window_lifecycle: DesktopRendererWindowLifecycle::Closed,
            renderer_runner_lifecycle: DesktopRendererRunnerLifecycle::Closed,
            renderer_runner_threading_model: DesktopRendererThreadingModel::Unsupported,
            renderer_window_strategy: window_strategy.strategy,
            renderer_window_platform: window_strategy.platform,
            renderer_window_capability: window_strategy.capability,
            renderer_window_capability_reason: window_strategy.capability_reason,
            renderer_window_visible: false,
            renderer_window_ready: false,
            renderer_window_verified_support: window_strategy.verified_support,
            renderer_window_visible_supported: window_strategy.visible_window_supported,
            renderer_window_focus_supported: false,
            renderer_window_message: "timeline renderer native window is unavailable".to_string(),
            track_count: 0,
            clip_count: 0,
            relationship_count: 0,
            affect_overlay_count: 0,
            queued_command_count: 0,
            last_error: Some(message),
        }
    }

    pub fn start(&mut self) -> TimelineHostStatus {
        if self.renderer.is_none() {
            self.renderer = Some(TimelineRendererApp::new());
        }
        self.last_error = None;
        self.status()
    }

    pub fn open_renderer(
        &mut self,
        projection: TimelineRenderProjection,
    ) -> Result<TimelineHostStatus, TimelineHostError> {
        self.set_projection(projection.clone())?;
        self.native_window.open_with_projection(projection);
        Ok(self.status())
    }

    pub fn focus(&mut self) -> TimelineHostStatus {
        self.status()
    }

    pub fn stop(&mut self) -> TimelineHostStatus {
        self.renderer = None;
        self.native_window.shutdown();
        self.last_error = None;
        self.status()
    }

    pub fn set_projection(
        &mut self,
        projection: TimelineRenderProjection,
    ) -> Result<TimelineHostStatus, TimelineHostError> {
        self.start();
        let Some(renderer) = self.renderer.as_mut() else {
            return Err(TimelineHostError::Renderer(
                TimelineRendererError::MissingProjection.to_string(),
            ));
        };
        let native_projection = projection.clone();

        Self::catch_renderer_panic(|| renderer.set_projection(projection)).map_err(|error| {
            self.last_error = Some(error_label(&error));
            error
        })?;
        self.native_window.update_projection(native_projection);
        self.last_error = None;
        Ok(self.status())
    }

    pub fn drain_commands(&mut self) -> Vec<TimelineRendererCommand> {
        let mut commands = self
            .renderer
            .as_mut()
            .map(TimelineRendererApp::drain_commands)
            .unwrap_or_default();
        commands.extend(self.native_window.drain_commands());
        commands
    }

    pub fn status(&mut self) -> TimelineHostStatus {
        let native_window_status = self.native_window.refresh_status();
        let (track_count, clip_count) = self
            .renderer
            .as_ref()
            .map(TimelineRendererApp::scene_counts)
            .unwrap_or_default();
        let relationship_count = self
            .renderer
            .as_ref()
            .map(TimelineRendererApp::scene_relationship_count)
            .unwrap_or_default();
        let affect_overlay_count = self
            .renderer
            .as_ref()
            .map(TimelineRendererApp::scene_affect_overlay_count)
            .unwrap_or_default();
        let queued_command_count = self
            .renderer
            .as_ref()
            .map(TimelineRendererApp::queued_command_count)
            .unwrap_or_default();
        let scene_running = self.renderer.is_some();

        TimelineHostStatus {
            renderer_window_kind: DesktopRendererWindowKind::Timeline,
            running: scene_running,
            renderer_window_open: scene_running,
            renderer_scene_ready: scene_running,
            renderer_window_lifecycle: DesktopRendererWindowLifecycle::from_state(
                scene_running,
                scene_running,
                native_window_status.window_visible,
            ),
            renderer_runner_lifecycle: native_window_status.lifecycle,
            renderer_runner_threading_model: native_window_status.threading_model,
            renderer_window_strategy: native_window_status.strategy,
            renderer_window_platform: native_window_status.platform,
            renderer_window_capability: native_window_status.capability,
            renderer_window_capability_reason: native_window_status.capability_reason,
            renderer_window_visible: native_window_status.window_visible,
            renderer_window_ready: native_window_status.window_ready,
            renderer_window_verified_support: native_window_status.verified_support,
            renderer_window_visible_supported: native_window_status.visible_window_supported,
            renderer_window_focus_supported: native_window_status.focus_supported,
            renderer_window_message: timeline_renderer_window_message(
                scene_running,
                &native_window_status,
            ),
            track_count,
            clip_count,
            relationship_count,
            affect_overlay_count,
            queued_command_count,
            last_error: self.last_error.clone(),
        }
    }

    fn catch_renderer_panic<T>(operation: impl FnOnce() -> T) -> Result<T, TimelineHostError> {
        catch_unwind(AssertUnwindSafe(operation)).map_err(|_| TimelineHostError::RendererPanic)
    }
}

fn timeline_renderer_window_message(
    scene_running: bool,
    native_window_status: &TimelineRendererRunnerStatus,
) -> String {
    match (
        scene_running,
        native_window_status.window_visible,
        native_window_status.last_error.as_ref(),
    ) {
        (true, true, _) => "floating timeline renderer window is visible".to_string(),
        (_, _, Some(error)) => error.clone(),
        (true, false, _) => {
            "timeline renderer scene is ready; native window is not connected".to_string()
        }
        (false, _, _) => "floating timeline renderer window is closed".to_string(),
    }
}

fn error_label(error: &TimelineHostError) -> String {
    match error {
        TimelineHostError::Renderer(message) => message.clone(),
        TimelineHostError::RendererPanic => "timeline renderer panicked".to_string(),
        TimelineHostError::QueueFull => "timeline renderer command queue is full".to_string(),
        TimelineHostError::OwnerReplyTimeout => {
            "timeline renderer owner reply timed out".to_string()
        }
        TimelineHostError::OwnerStopped => "timeline renderer owner stopped".to_string(),
    }
}

#[cfg(test)]
#[path = "bevy_timeline_host_tests.rs"]
mod tests;
