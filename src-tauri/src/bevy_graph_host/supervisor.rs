use super::{
    BibleGraphRendererWindowCapabilityReason, NativeRendererPlatformStrategy, NativeRendererRunner,
    NativeRendererRunnerLifecycle, NativeRendererRunnerStartupPlan, NativeRendererRunnerStatus,
};

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
    lifecycle: NativeRendererSupervisorLifecycle,
    last_error: Option<String>,
}

impl NativeRendererSupervisor {
    pub fn for_strategy(strategy: NativeRendererPlatformStrategy) -> Self {
        Self {
            strategy,
            startup_plan: strategy.runner_startup_plan(),
            lifecycle: NativeRendererSupervisorLifecycle::NotStarted,
            last_error: None,
        }
    }

    pub fn failed_current_platform_status(message: String) -> NativeRendererRunnerStatus {
        Self {
            strategy: NativeRendererPlatformStrategy::current(),
            startup_plan: NativeRendererPlatformStrategy::current().runner_startup_plan(),
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
}

impl NativeRendererRunner for NativeRendererSupervisor {
    fn open(&mut self) -> NativeRendererRunnerStatus {
        self.lifecycle = match self.startup_plan {
            NativeRendererRunnerStartupPlan::MinimalWindowProofCandidate { .. } => {
                NativeRendererSupervisorLifecycle::Starting
            }
            NativeRendererRunnerStartupPlan::PendingOnly { .. } => {
                NativeRendererSupervisorLifecycle::Closed
            }
        };
        self.status()
    }

    fn close(&mut self) -> NativeRendererRunnerStatus {
        self.lifecycle = NativeRendererSupervisorLifecycle::Closed;
        self.last_error = None;
        self.status()
    }

    fn focus(&mut self) -> NativeRendererRunnerStatus {
        self.status()
    }

    fn status(&self) -> NativeRendererRunnerStatus {
        let strategy = self.strategy.status();
        let capability_reason = if self.lifecycle == NativeRendererSupervisorLifecycle::Failed {
            BibleGraphRendererWindowCapabilityReason::RunnerError
        } else {
            strategy.capability_reason
        };

        NativeRendererRunnerStatus {
            strategy: strategy.strategy,
            platform: strategy.platform,
            lifecycle: self.runner_lifecycle(),
            supervisor_lifecycle: self.lifecycle,
            threading_model: self.strategy.threading_model(),
            capability: strategy.capability,
            capability_reason,
            visible_window_supported: strategy.visible_window_supported,
            window_visible: false,
            window_ready: false,
            focus_supported: false,
            last_error: self.last_error.clone(),
        }
    }
}
