use super::{
    BibleGraphRendererWindowCapability, BibleGraphRendererWindowCapabilityReason,
    NativeRendererPlatformStrategy, NativeRendererRunner, NativeRendererRunnerLifecycle,
    NativeRendererRunnerStartupPlan, NativeRendererRunnerStatus, NativeRendererWindowThreadHandle,
    NativeRendererWindowThreadResult,
};
use eidetic_bevy_bible_graph::BibleGraphNativeWindowRunnerConfig;
use std::time::Duration;

const NATIVE_RENDERER_WINDOW_STOP_TIMEOUT: Duration = Duration::from_millis(2_000);

type NativeRendererWindowThreadStart =
    fn(BibleGraphNativeWindowRunnerConfig) -> std::io::Result<NativeRendererWindowThreadHandle>;

#[derive(Debug, Clone, Copy, PartialEq, Eq, serde::Serialize)]
#[serde(rename_all = "snake_case")]
pub enum NativeRendererSupervisorLifecycle {
    NotStarted,
    Starting,
    Running,
    Closing,
    Closed,
    Failed,
}

#[derive(Debug)]
pub struct NativeRendererSupervisor {
    strategy: NativeRendererPlatformStrategy,
    startup_plan: NativeRendererRunnerStartupPlan,
    window_thread_start: NativeRendererWindowThreadStart,
    window_thread: Option<NativeRendererWindowThreadHandle>,
    window_ready: bool,
    lifecycle: NativeRendererSupervisorLifecycle,
    last_error: Option<String>,
}

impl NativeRendererSupervisor {
    pub fn for_strategy(strategy: NativeRendererPlatformStrategy) -> Self {
        Self::for_strategy_with_window_thread_start(
            strategy,
            NativeRendererWindowThreadHandle::start,
        )
    }

    pub(crate) fn for_strategy_with_window_thread_start(
        strategy: NativeRendererPlatformStrategy,
        window_thread_start: NativeRendererWindowThreadStart,
    ) -> Self {
        Self {
            strategy,
            startup_plan: strategy.runner_startup_plan(),
            window_thread_start,
            window_thread: None,
            window_ready: false,
            lifecycle: NativeRendererSupervisorLifecycle::NotStarted,
            last_error: None,
        }
    }

    pub fn failed_current_platform_status(message: String) -> NativeRendererRunnerStatus {
        Self {
            strategy: NativeRendererPlatformStrategy::current(),
            startup_plan: NativeRendererPlatformStrategy::current().runner_startup_plan(),
            window_thread_start: NativeRendererWindowThreadHandle::start,
            window_thread: None,
            window_ready: false,
            lifecycle: NativeRendererSupervisorLifecycle::Failed,
            last_error: Some(message),
        }
        .status()
    }

    pub fn startup_plan(&self) -> &NativeRendererRunnerStartupPlan {
        &self.startup_plan
    }

    pub fn lifecycle(&self) -> NativeRendererSupervisorLifecycle {
        self.lifecycle
    }

    fn runner_lifecycle(&self) -> NativeRendererRunnerLifecycle {
        match self.lifecycle {
            NativeRendererSupervisorLifecycle::NotStarted
            | NativeRendererSupervisorLifecycle::Closed
            | NativeRendererSupervisorLifecycle::Failed => NativeRendererRunnerLifecycle::Closed,
            NativeRendererSupervisorLifecycle::Starting
            | NativeRendererSupervisorLifecycle::Closing => {
                NativeRendererRunnerLifecycle::OpenRequested
            }
            NativeRendererSupervisorLifecycle::Running => NativeRendererRunnerLifecycle::Visible,
        }
    }

    pub fn refresh_status(&mut self) -> NativeRendererRunnerStatus {
        self.refresh_window_thread();
        self.status()
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
            Some(NativeRendererWindowThreadResult::Completed) if status.close_requested => {
                self.lifecycle = NativeRendererSupervisorLifecycle::Closed;
                self.last_error = None;
            }
            Some(NativeRendererWindowThreadResult::Completed) => {
                self.lifecycle = NativeRendererSupervisorLifecycle::Failed;
                self.last_error = Some("native renderer window closed unexpectedly".to_string());
            }
            Some(NativeRendererWindowThreadResult::Panicked) => {
                self.lifecycle = NativeRendererSupervisorLifecycle::Failed;
                self.last_error = Some("native renderer window thread panicked".to_string());
            }
            None => {}
        }
    }

    fn open_minimal_window(
        &mut self,
        config: BibleGraphNativeWindowRunnerConfig,
    ) -> NativeRendererRunnerStatus {
        self.refresh_window_thread();
        if self.window_thread.is_some() {
            self.lifecycle = NativeRendererSupervisorLifecycle::Running;
            self.last_error = None;
            return self.status();
        }

        self.lifecycle = NativeRendererSupervisorLifecycle::Starting;
        match (self.window_thread_start)(config) {
            Ok(window_thread) => {
                self.window_thread = Some(window_thread);
                self.window_ready = false;
                self.lifecycle = NativeRendererSupervisorLifecycle::Running;
                self.last_error = None;
            }
            Err(error) => {
                self.lifecycle = NativeRendererSupervisorLifecycle::Failed;
                self.last_error = Some(format!("failed to start native renderer window: {error}"));
            }
        }
        self.status()
    }

    fn close_window_thread(&mut self) {
        let Some(mut window_thread) = self.window_thread.take() else {
            self.lifecycle = NativeRendererSupervisorLifecycle::Closed;
            self.window_ready = false;
            self.last_error = None;
            return;
        };

        self.lifecycle = NativeRendererSupervisorLifecycle::Closing;
        let status = window_thread.stop(NATIVE_RENDERER_WINDOW_STOP_TIMEOUT);
        if status.running {
            self.lifecycle = NativeRendererSupervisorLifecycle::Failed;
            self.window_ready = status.ready;
            self.last_error =
                Some("native renderer window did not stop before timeout".to_string());
            self.window_thread = Some(window_thread);
            return;
        }

        match status.result {
            Some(NativeRendererWindowThreadResult::Completed) => {
                self.lifecycle = NativeRendererSupervisorLifecycle::Closed;
                self.window_ready = false;
                self.last_error = None;
            }
            Some(NativeRendererWindowThreadResult::Panicked) => {
                self.lifecycle = NativeRendererSupervisorLifecycle::Failed;
                self.window_ready = false;
                self.last_error = Some("native renderer window thread panicked".to_string());
            }
            None => {
                self.lifecycle = NativeRendererSupervisorLifecycle::Failed;
                self.window_ready = false;
                self.last_error =
                    Some("native renderer window stop completed without a result".to_string());
            }
        }
    }
}

impl NativeRendererRunner for NativeRendererSupervisor {
    fn open(&mut self) -> NativeRendererRunnerStatus {
        match self.startup_plan.clone() {
            NativeRendererRunnerStartupPlan::MinimalWindowProofCandidate { config, .. } => {
                return self.open_minimal_window(config);
            }
            NativeRendererRunnerStartupPlan::PendingOnly { .. } => {
                self.lifecycle = NativeRendererSupervisorLifecycle::Closed;
                self.last_error = None;
            }
        }
        self.status()
    }

    fn close(&mut self) -> NativeRendererRunnerStatus {
        self.close_window_thread();
        self.status()
    }

    fn focus(&mut self) -> NativeRendererRunnerStatus {
        self.refresh_status()
    }

    fn status(&self) -> NativeRendererRunnerStatus {
        let strategy = self.strategy.status();
        let (capability, capability_reason) =
            if self.lifecycle == NativeRendererSupervisorLifecycle::Failed {
                (
                    BibleGraphRendererWindowCapability::RunnerError,
                    BibleGraphRendererWindowCapabilityReason::RunnerError,
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
            && matches!(self.lifecycle, NativeRendererSupervisorLifecycle::Running);

        NativeRendererRunnerStatus {
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
