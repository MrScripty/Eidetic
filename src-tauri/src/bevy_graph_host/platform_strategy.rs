use super::{
    BibleGraphRendererWindowCapability, BibleGraphRendererWindowCapabilityReason,
    BibleGraphRendererWindowPlatform, BibleGraphRendererWindowStrategy,
    BibleGraphRendererWindowStrategyStatus,
};
use eidetic_bevy_bible_graph::BibleGraphNativeWindowRunnerConfig;

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

impl NativeRendererPlatformStrategy {
    pub fn current() -> Self {
        match BibleGraphRendererWindowPlatform::current() {
            BibleGraphRendererWindowPlatform::Linux => Self::LinuxWorkerThreadUnproven,
            BibleGraphRendererWindowPlatform::Macos => Self::MacosMainThreadUnproven,
            BibleGraphRendererWindowPlatform::Windows => Self::WindowsWorkerThreadUnproven,
            BibleGraphRendererWindowPlatform::Unsupported => Self::UnsupportedPlatform,
        }
    }

    pub fn status(self) -> BibleGraphRendererWindowStrategyStatus {
        BibleGraphRendererWindowStrategyStatus {
            strategy: BibleGraphRendererWindowStrategy::BevyWinitFloatingWindow,
            platform: self.platform(),
            capability: BibleGraphRendererWindowCapability::PendingNativeRunner,
            capability_reason: self.capability_reason(),
            visible_window_supported: false,
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

    fn platform(self) -> BibleGraphRendererWindowPlatform {
        match self {
            Self::LinuxWorkerThreadUnproven => BibleGraphRendererWindowPlatform::Linux,
            Self::MacosMainThreadUnproven => BibleGraphRendererWindowPlatform::Macos,
            Self::WindowsWorkerThreadUnproven => BibleGraphRendererWindowPlatform::Windows,
            Self::UnsupportedPlatform => BibleGraphRendererWindowPlatform::Unsupported,
        }
    }

    fn capability_reason(self) -> BibleGraphRendererWindowCapabilityReason {
        match self {
            Self::LinuxWorkerThreadUnproven
            | Self::MacosMainThreadUnproven
            | Self::WindowsWorkerThreadUnproven => {
                BibleGraphRendererWindowCapabilityReason::PendingNativeRunner
            }
            Self::UnsupportedPlatform => {
                BibleGraphRendererWindowCapabilityReason::PlatformUnsupported
            }
        }
    }
}
