mod current_platform;

use super::{
    BibleGraphRendererWindowCapability, BibleGraphRendererWindowCapabilityReason,
    BibleGraphRendererWindowPlatform, BibleGraphRendererWindowStrategy,
    BibleGraphRendererWindowStrategyStatus,
};
use eidetic_bevy_bible_graph::BibleGraphNativeWindowRunnerConfig;

pub use current_platform::current_renderer_window_platform;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum NativeRendererPlatformStrategy {
    LinuxWorkerThreadUnproven,
    MacosMainThreadUnproven,
    WindowsWorkerThreadUnproven,
    UnsupportedPlatform,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, serde::Serialize)]
#[serde(rename_all = "snake_case")]
pub enum NativeRendererThreadingModel {
    WorkerThread,
    MainThread,
    Unsupported,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum NativeRendererRunnerStartupPlan {
    MinimalWindowProofCandidate {
        threading_model: NativeRendererThreadingModel,
        config: BibleGraphNativeWindowRunnerConfig,
    },
    PendingOnly {
        threading_model: NativeRendererThreadingModel,
    },
}

impl NativeRendererPlatformStrategy {
    pub fn current() -> Self {
        match current_renderer_window_platform() {
            BibleGraphRendererWindowPlatform::Linux => Self::LinuxWorkerThreadUnproven,
            BibleGraphRendererWindowPlatform::Macos => Self::MacosMainThreadUnproven,
            BibleGraphRendererWindowPlatform::Windows => Self::WindowsWorkerThreadUnproven,
            BibleGraphRendererWindowPlatform::Unsupported => Self::UnsupportedPlatform,
        }
    }

    pub fn status(self) -> BibleGraphRendererWindowStrategyStatus {
        let capability = self.capability();
        BibleGraphRendererWindowStrategyStatus {
            strategy: BibleGraphRendererWindowStrategy::BevyWinitFloatingWindow,
            platform: self.platform(),
            capability,
            capability_reason: self.capability_reason(),
            verified_support: capability.verified_support(),
            visible_window_supported: capability.visible_window_supported(),
        }
    }

    pub fn threading_model(self) -> NativeRendererThreadingModel {
        match self {
            Self::LinuxWorkerThreadUnproven | Self::WindowsWorkerThreadUnproven => {
                NativeRendererThreadingModel::WorkerThread
            }
            Self::MacosMainThreadUnproven => NativeRendererThreadingModel::MainThread,
            Self::UnsupportedPlatform => NativeRendererThreadingModel::Unsupported,
        }
    }

    pub fn can_attempt_minimal_window_proof(self) -> bool {
        !matches!(
            self.threading_model(),
            NativeRendererThreadingModel::Unsupported
        )
    }

    pub fn minimal_window_runner_config(self) -> Option<BibleGraphNativeWindowRunnerConfig> {
        let run_on_any_thread = match self.threading_model() {
            NativeRendererThreadingModel::WorkerThread => true,
            NativeRendererThreadingModel::MainThread => false,
            NativeRendererThreadingModel::Unsupported => return None,
        };

        Some(BibleGraphNativeWindowRunnerConfig::minimal_smoke(
            run_on_any_thread,
        ))
    }

    pub fn runner_startup_plan(self) -> NativeRendererRunnerStartupPlan {
        match self.minimal_window_runner_config() {
            Some(config) => NativeRendererRunnerStartupPlan::MinimalWindowProofCandidate {
                threading_model: self.threading_model(),
                config,
            },
            None => NativeRendererRunnerStartupPlan::PendingOnly {
                threading_model: self.threading_model(),
            },
        }
    }

    fn platform(self) -> BibleGraphRendererWindowPlatform {
        match self {
            Self::LinuxWorkerThreadUnproven => BibleGraphRendererWindowPlatform::Linux,
            Self::MacosMainThreadUnproven => BibleGraphRendererWindowPlatform::Macos,
            Self::WindowsWorkerThreadUnproven => BibleGraphRendererWindowPlatform::Windows,
            Self::UnsupportedPlatform => BibleGraphRendererWindowPlatform::Unsupported,
        }
    }

    fn capability(self) -> BibleGraphRendererWindowCapability {
        match self {
            Self::LinuxWorkerThreadUnproven
            | Self::MacosMainThreadUnproven
            | Self::WindowsWorkerThreadUnproven => {
                BibleGraphRendererWindowCapability::PlatformUnproven
            }
            Self::UnsupportedPlatform => BibleGraphRendererWindowCapability::PlatformUnsupported,
        }
    }

    fn capability_reason(self) -> BibleGraphRendererWindowCapabilityReason {
        match self {
            Self::LinuxWorkerThreadUnproven
            | Self::MacosMainThreadUnproven
            | Self::WindowsWorkerThreadUnproven => {
                BibleGraphRendererWindowCapabilityReason::PlatformUnproven
            }
            Self::UnsupportedPlatform => {
                BibleGraphRendererWindowCapabilityReason::PlatformUnsupported
            }
        }
    }
}
