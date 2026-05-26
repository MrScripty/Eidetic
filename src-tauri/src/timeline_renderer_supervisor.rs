use std::time::Duration;
#[cfg(test)]
use std::time::Instant;

use eidetic_bevy_timeline::TimelineNativeWindowRunnerConfig;
use eidetic_core::contracts::TimelineRenderProjection;

use crate::renderer_window::{
    DesktopRendererRunnerLifecycle, DesktopRendererSupervisorLifecycle,
    DesktopRendererThreadingModel, DesktopRendererWindowCapability,
    DesktopRendererWindowCapabilityReason, DesktopRendererWindowPlatform,
    DesktopRendererWindowStrategy,
};
use crate::timeline_renderer_platform_strategy::{
    TimelineRendererPlatformStrategy, TimelineRendererRunnerStartupPlan,
};
use crate::timeline_renderer_window_thread::{
    TimelineRendererWindowProjectionUpdateError, TimelineRendererWindowThreadHandle,
    TimelineRendererWindowThreadResult,
};

const TIMELINE_RENDERER_WINDOW_STOP_TIMEOUT: Duration = Duration::from_millis(2_000);

type TimelineRendererWindowThreadStart =
    fn(TimelineNativeWindowRunnerConfig) -> std::io::Result<TimelineRendererWindowThreadHandle>;

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct TimelineRendererRunnerStatus {
    pub strategy: DesktopRendererWindowStrategy,
    pub platform: DesktopRendererWindowPlatform,
    pub lifecycle: DesktopRendererRunnerLifecycle,
    pub supervisor_lifecycle: DesktopRendererSupervisorLifecycle,
    pub threading_model: DesktopRendererThreadingModel,
    pub capability: DesktopRendererWindowCapability,
    pub capability_reason: DesktopRendererWindowCapabilityReason,
    pub verified_support: bool,
    pub visible_window_supported: bool,
    pub window_visible: bool,
    pub window_ready: bool,
    pub focus_supported: bool,
    pub last_error: Option<String>,
}

#[derive(Debug)]
pub struct TimelineRendererSupervisor {
    strategy: TimelineRendererPlatformStrategy,
    startup_plan: TimelineRendererRunnerStartupPlan,
    window_thread_start: TimelineRendererWindowThreadStart,
    window_thread: Option<TimelineRendererWindowThreadHandle>,
    window_ready: bool,
    lifecycle: DesktopRendererSupervisorLifecycle,
    last_error: Option<String>,
}

impl TimelineRendererSupervisor {
    pub fn current() -> Self {
        Self::with_strategy_and_window_thread_start(
            TimelineRendererPlatformStrategy::current(),
            TimelineRendererWindowThreadHandle::start,
        )
    }

    #[cfg(test)]
    pub(crate) fn for_strategy(strategy: TimelineRendererPlatformStrategy) -> Self {
        Self::with_strategy_and_window_thread_start(
            strategy,
            TimelineRendererWindowThreadHandle::start,
        )
    }

    #[cfg(test)]
    pub(crate) fn for_strategy_with_window_thread_start(
        strategy: TimelineRendererPlatformStrategy,
        window_thread_start: TimelineRendererWindowThreadStart,
    ) -> Self {
        Self::with_strategy_and_window_thread_start(strategy, window_thread_start)
    }

    fn with_strategy_and_window_thread_start(
        strategy: TimelineRendererPlatformStrategy,
        window_thread_start: TimelineRendererWindowThreadStart,
    ) -> Self {
        Self {
            strategy,
            startup_plan: strategy.runner_startup_plan(),
            window_thread_start,
            window_thread: None,
            window_ready: false,
            lifecycle: DesktopRendererSupervisorLifecycle::NotStarted,
            last_error: None,
        }
    }

    pub fn failed_current_platform_status(message: String) -> TimelineRendererRunnerStatus {
        Self {
            strategy: TimelineRendererPlatformStrategy::current(),
            startup_plan: TimelineRendererPlatformStrategy::current().runner_startup_plan(),
            window_thread_start: TimelineRendererWindowThreadHandle::start,
            window_thread: None,
            window_ready: false,
            lifecycle: DesktopRendererSupervisorLifecycle::Failed,
            last_error: Some(message),
        }
        .status()
    }

    pub fn refresh_status(&mut self) -> TimelineRendererRunnerStatus {
        self.refresh_window_thread();
        self.status()
    }

    pub fn open(&mut self) -> TimelineRendererRunnerStatus {
        match self.startup_plan.clone() {
            TimelineRendererRunnerStartupPlan::MinimalWindowProofCandidate { config, .. } => {
                return self.open_minimal_window(config);
            }
            TimelineRendererRunnerStartupPlan::PendingOnly { .. } => {
                self.lifecycle = DesktopRendererSupervisorLifecycle::Closed;
                self.last_error = None;
            }
        }
        self.status()
    }

    pub fn open_with_projection(
        &mut self,
        projection: TimelineRenderProjection,
    ) -> TimelineRendererRunnerStatus {
        match self.startup_plan.clone() {
            TimelineRendererRunnerStartupPlan::MinimalWindowProofCandidate { config, .. } => {
                return self.open_minimal_window(config.with_initial_projection(projection));
            }
            TimelineRendererRunnerStartupPlan::PendingOnly { .. } => {
                self.lifecycle = DesktopRendererSupervisorLifecycle::Closed;
                self.last_error = None;
            }
        }
        self.status()
    }

    pub fn update_projection(
        &mut self,
        projection: TimelineRenderProjection,
    ) -> TimelineRendererRunnerStatus {
        self.refresh_window_thread();
        let Some(window_thread) = self.window_thread.as_ref() else {
            return self.status();
        };
        if let Err(error) = window_thread.update_projection(projection) {
            self.lifecycle = DesktopRendererSupervisorLifecycle::Failed;
            self.last_error = Some(timeline_projection_update_error_message(error));
        }
        self.status()
    }

    pub fn drain_commands(&mut self) -> Vec<eidetic_bevy_timeline::TimelineRendererCommand> {
        self.refresh_window_thread();
        self.window_thread
            .as_ref()
            .map(TimelineRendererWindowThreadHandle::drain_commands)
            .unwrap_or_default()
    }

    pub fn close(&mut self) -> TimelineRendererRunnerStatus {
        self.close_window_thread();
        self.status()
    }

    pub fn shutdown(&mut self) -> TimelineRendererRunnerStatus {
        self.shutdown_window_thread();
        self.status()
    }

    fn runner_lifecycle(&self) -> DesktopRendererRunnerLifecycle {
        match self.lifecycle {
            DesktopRendererSupervisorLifecycle::NotStarted
            | DesktopRendererSupervisorLifecycle::Closed
            | DesktopRendererSupervisorLifecycle::Failed => DesktopRendererRunnerLifecycle::Closed,
            DesktopRendererSupervisorLifecycle::Starting
            | DesktopRendererSupervisorLifecycle::Closing => {
                DesktopRendererRunnerLifecycle::OpenRequested
            }
            DesktopRendererSupervisorLifecycle::Running => DesktopRendererRunnerLifecycle::Visible,
        }
    }

    fn refresh_window_thread(&mut self) {
        let Some(window_thread) = self.window_thread.as_mut() else {
            return;
        };
        let status = window_thread.join_completed();
        self.window_ready = status.ready;
        if status.running {
            return;
        }

        self.window_thread = None;
        self.window_ready = false;
        match status.result {
            Some(TimelineRendererWindowThreadResult::Completed) if status.close_requested => {
                self.lifecycle = DesktopRendererSupervisorLifecycle::Closed;
                self.last_error = None;
            }
            Some(TimelineRendererWindowThreadResult::Completed) => {
                self.lifecycle = DesktopRendererSupervisorLifecycle::Failed;
                self.last_error = Some("timeline renderer window closed unexpectedly".to_string());
            }
            Some(TimelineRendererWindowThreadResult::Panicked) => {
                self.lifecycle = DesktopRendererSupervisorLifecycle::Failed;
                self.last_error = Some("timeline renderer window thread panicked".to_string());
            }
            None => {}
        }
    }

    fn open_minimal_window(
        &mut self,
        config: TimelineNativeWindowRunnerConfig,
    ) -> TimelineRendererRunnerStatus {
        self.refresh_window_thread();
        if let Some(window_thread) = self.window_thread.as_ref() {
            window_thread.request_show();
            self.lifecycle = DesktopRendererSupervisorLifecycle::Running;
            self.last_error = None;
            return self.status();
        }

        self.lifecycle = DesktopRendererSupervisorLifecycle::Starting;
        match (self.window_thread_start)(config) {
            Ok(window_thread) => {
                self.window_thread = Some(window_thread);
                self.window_ready = false;
                self.lifecycle = DesktopRendererSupervisorLifecycle::Running;
                self.last_error = None;
            }
            Err(error) => {
                self.lifecycle = DesktopRendererSupervisorLifecycle::Failed;
                self.last_error =
                    Some(format!("failed to start timeline renderer window: {error}"));
            }
        }
        self.status()
    }

    fn close_window_thread(&mut self) {
        let Some(window_thread) = self.window_thread.as_ref() else {
            self.lifecycle = DesktopRendererSupervisorLifecycle::Closed;
            self.window_ready = false;
            self.last_error = None;
            return;
        };

        window_thread.request_hide();
        self.lifecycle = DesktopRendererSupervisorLifecycle::Closed;
        self.last_error = None;
    }

    fn shutdown_window_thread(&mut self) {
        let Some(mut window_thread) = self.window_thread.take() else {
            self.lifecycle = DesktopRendererSupervisorLifecycle::Closed;
            self.window_ready = false;
            self.last_error = None;
            return;
        };

        self.lifecycle = DesktopRendererSupervisorLifecycle::Closing;
        let status = window_thread.stop(TIMELINE_RENDERER_WINDOW_STOP_TIMEOUT);
        if status.running {
            self.lifecycle = DesktopRendererSupervisorLifecycle::Failed;
            self.window_ready = status.ready;
            self.last_error =
                Some("timeline renderer window did not stop before timeout".to_string());
            self.window_thread = Some(window_thread);
            return;
        }

        match status.result {
            Some(TimelineRendererWindowThreadResult::Completed) => {
                self.lifecycle = DesktopRendererSupervisorLifecycle::Closed;
                self.window_ready = false;
                self.last_error = None;
            }
            Some(TimelineRendererWindowThreadResult::Panicked) => {
                self.lifecycle = DesktopRendererSupervisorLifecycle::Failed;
                self.window_ready = false;
                self.last_error = Some("timeline renderer window thread panicked".to_string());
            }
            None => {
                self.lifecycle = DesktopRendererSupervisorLifecycle::Failed;
                self.window_ready = false;
                self.last_error =
                    Some("timeline renderer window stop completed without a result".to_string());
            }
        }
    }

    pub fn status(&self) -> TimelineRendererRunnerStatus {
        let strategy = self.strategy.status();
        let (capability, capability_reason) =
            if self.lifecycle == DesktopRendererSupervisorLifecycle::Failed {
                (
                    DesktopRendererWindowCapability::RunnerError,
                    DesktopRendererWindowCapabilityReason::RunnerError,
                )
            } else {
                (strategy.capability, strategy.capability_reason)
            };
        let visible_window_supported = if capability.verified_support() {
            strategy.visible_window_supported
        } else {
            false
        };
        let window_running = self.window_thread.is_some()
            && matches!(self.lifecycle, DesktopRendererSupervisorLifecycle::Running);

        TimelineRendererRunnerStatus {
            strategy: strategy.strategy,
            platform: strategy.platform,
            lifecycle: self.runner_lifecycle(),
            supervisor_lifecycle: self.lifecycle,
            threading_model: self.strategy.threading_model(),
            capability,
            capability_reason,
            verified_support: capability.verified_support(),
            visible_window_supported,
            window_visible: window_running,
            window_ready: window_running && self.window_ready,
            focus_supported: false,
            last_error: self.last_error.clone(),
        }
    }
}

fn timeline_projection_update_error_message(
    error: TimelineRendererWindowProjectionUpdateError,
) -> String {
    match error {
        TimelineRendererWindowProjectionUpdateError::QueueFull => {
            "timeline renderer projection update queue is full".to_string()
        }
        TimelineRendererWindowProjectionUpdateError::WindowClosed => {
            "timeline renderer window is closed".to_string()
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn timeline_renderer_supervisor_starts_injected_window_thread() {
        let mut supervisor = TimelineRendererSupervisor::for_strategy_with_window_thread_start(
            TimelineRendererPlatformStrategy::LinuxWorkerThreadVerified,
            injected_ready_window_thread,
        );

        let opened = supervisor.open();
        let refreshed = wait_for_ready_status(&mut supervisor);
        let stopped = supervisor.shutdown();

        assert_eq!(opened.lifecycle, DesktopRendererRunnerLifecycle::Visible);
        assert_eq!(
            opened.supervisor_lifecycle,
            DesktopRendererSupervisorLifecycle::Running
        );
        assert!(refreshed.window_ready);
        assert!(refreshed.window_visible);
        assert_eq!(stopped.lifecycle, DesktopRendererRunnerLifecycle::Closed);
    }

    #[test]
    fn timeline_renderer_supervisor_keeps_unsupported_platform_closed() {
        let mut supervisor = TimelineRendererSupervisor::for_strategy(
            TimelineRendererPlatformStrategy::UnsupportedPlatform,
        );

        let status = supervisor.open();

        assert_eq!(status.lifecycle, DesktopRendererRunnerLifecycle::Closed);
        assert_eq!(
            status.threading_model,
            DesktopRendererThreadingModel::Unsupported
        );
        assert_eq!(
            status.capability,
            DesktopRendererWindowCapability::PlatformUnsupported
        );
        assert!(!status.window_visible);
    }

    #[test]
    fn timeline_renderer_supervisor_reports_window_thread_panic() {
        let mut supervisor = TimelineRendererSupervisor::for_strategy_with_window_thread_start(
            TimelineRendererPlatformStrategy::LinuxWorkerThreadVerified,
            |_config| {
                TimelineRendererWindowThreadHandle::start_with(
                    TimelineNativeWindowRunnerConfig::minimal_smoke(true),
                    |_config, _control| {
                        panic!("timeline window panic");
                    },
                )
            },
        );

        supervisor.open();
        let status = supervisor.shutdown();

        assert_eq!(status.lifecycle, DesktopRendererRunnerLifecycle::Closed);
        assert_eq!(
            status.supervisor_lifecycle,
            DesktopRendererSupervisorLifecycle::Failed
        );
        assert_eq!(
            status.capability,
            DesktopRendererWindowCapability::RunnerError
        );
        assert!(status.last_error.is_some());
    }

    fn injected_ready_window_thread(
        config: TimelineNativeWindowRunnerConfig,
    ) -> std::io::Result<TimelineRendererWindowThreadHandle> {
        TimelineRendererWindowThreadHandle::start_with(config, |_config, control| {
            control.mark_ready();
            control.mark_visible(true);
            while !control.close_requested() {
                std::thread::sleep(Duration::from_millis(1));
            }
        })
    }

    fn wait_for_ready_status(
        supervisor: &mut TimelineRendererSupervisor,
    ) -> TimelineRendererRunnerStatus {
        let deadline = Instant::now() + Duration::from_millis(200);
        loop {
            let status = supervisor.refresh_status();
            if status.window_ready || Instant::now() >= deadline {
                return status;
            }
            std::thread::sleep(Duration::from_millis(1));
        }
    }
}
