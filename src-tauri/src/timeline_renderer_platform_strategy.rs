use eidetic_bevy_timeline::TimelineNativeWindowRunnerConfig;

use crate::renderer_window::{
    DesktopRendererThreadingModel, DesktopRendererWindowCapability,
    DesktopRendererWindowCapabilityReason, DesktopRendererWindowPlatform,
    DesktopRendererWindowStrategy, DesktopRendererWindowStrategyStatus,
    current_desktop_renderer_window_platform,
};

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(crate) enum TimelineRendererPlatformStrategy {
    LinuxWorkerThreadVerified,
    MacosMainThreadUnproven,
    WindowsWorkerThreadUnproven,
    UnsupportedPlatform,
}

#[derive(Debug, Clone, PartialEq)]
pub(crate) enum TimelineRendererRunnerStartupPlan {
    MinimalWindowProofCandidate {
        threading_model: DesktopRendererThreadingModel,
        config: Box<TimelineNativeWindowRunnerConfig>,
    },
    PendingOnly {
        threading_model: DesktopRendererThreadingModel,
    },
}

impl TimelineRendererPlatformStrategy {
    pub(crate) fn current() -> Self {
        match current_desktop_renderer_window_platform() {
            DesktopRendererWindowPlatform::Linux => Self::LinuxWorkerThreadVerified,
            DesktopRendererWindowPlatform::Macos => Self::MacosMainThreadUnproven,
            DesktopRendererWindowPlatform::Windows => Self::WindowsWorkerThreadUnproven,
            DesktopRendererWindowPlatform::Unsupported => Self::UnsupportedPlatform,
        }
    }

    pub(crate) fn status(self) -> DesktopRendererWindowStrategyStatus {
        let capability = self.capability();
        DesktopRendererWindowStrategyStatus {
            strategy: DesktopRendererWindowStrategy::BevyWinitFloatingWindow,
            platform: self.platform(),
            capability,
            capability_reason: self.capability_reason(),
            verified_support: capability.verified_support(),
            visible_window_supported: capability.visible_window_supported(),
        }
    }

    pub(crate) fn threading_model(self) -> DesktopRendererThreadingModel {
        match self {
            Self::LinuxWorkerThreadVerified | Self::WindowsWorkerThreadUnproven => {
                DesktopRendererThreadingModel::WorkerThread
            }
            Self::MacosMainThreadUnproven => DesktopRendererThreadingModel::MainThread,
            Self::UnsupportedPlatform => DesktopRendererThreadingModel::Unsupported,
        }
    }

    pub(crate) fn minimal_window_runner_config(self) -> Option<TimelineNativeWindowRunnerConfig> {
        let run_on_any_thread = match self.threading_model() {
            DesktopRendererThreadingModel::WorkerThread => true,
            DesktopRendererThreadingModel::MainThread => false,
            DesktopRendererThreadingModel::Unsupported => return None,
        };

        Some(TimelineNativeWindowRunnerConfig::minimal_smoke(
            run_on_any_thread,
        ))
    }

    pub(crate) fn runner_startup_plan(self) -> TimelineRendererRunnerStartupPlan {
        match self.minimal_window_runner_config() {
            Some(config) => TimelineRendererRunnerStartupPlan::MinimalWindowProofCandidate {
                threading_model: self.threading_model(),
                config: Box::new(config),
            },
            None => TimelineRendererRunnerStartupPlan::PendingOnly {
                threading_model: self.threading_model(),
            },
        }
    }

    fn platform(self) -> DesktopRendererWindowPlatform {
        match self {
            Self::LinuxWorkerThreadVerified => DesktopRendererWindowPlatform::Linux,
            Self::MacosMainThreadUnproven => DesktopRendererWindowPlatform::Macos,
            Self::WindowsWorkerThreadUnproven => DesktopRendererWindowPlatform::Windows,
            Self::UnsupportedPlatform => DesktopRendererWindowPlatform::Unsupported,
        }
    }

    fn capability(self) -> DesktopRendererWindowCapability {
        match self {
            Self::LinuxWorkerThreadVerified => DesktopRendererWindowCapability::VerifiedSupport,
            Self::MacosMainThreadUnproven | Self::WindowsWorkerThreadUnproven => {
                DesktopRendererWindowCapability::PlatformUnproven
            }
            Self::UnsupportedPlatform => DesktopRendererWindowCapability::PlatformUnsupported,
        }
    }

    fn capability_reason(self) -> DesktopRendererWindowCapabilityReason {
        match self {
            Self::LinuxWorkerThreadVerified => {
                DesktopRendererWindowCapabilityReason::VerifiedSupport
            }
            Self::MacosMainThreadUnproven | Self::WindowsWorkerThreadUnproven => {
                DesktopRendererWindowCapabilityReason::PlatformUnproven
            }
            Self::UnsupportedPlatform => DesktopRendererWindowCapabilityReason::PlatformUnsupported,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn timeline_renderer_platform_strategy_reports_current_platform_status() {
        let strategy = TimelineRendererPlatformStrategy::current();
        let status = strategy.status();

        assert_eq!(status.platform, current_desktop_renderer_window_platform());
        assert_eq!(
            status.strategy,
            DesktopRendererWindowStrategy::BevyWinitFloatingWindow
        );
        assert_eq!(status.capability, expected_capability());
        assert_eq!(status.capability_reason, expected_capability_reason());
        assert_eq!(status.verified_support, expected_verified_support());
        assert_eq!(
            status.visible_window_supported,
            expected_visible_window_supported()
        );
    }

    #[test]
    fn timeline_renderer_platform_strategy_builds_startup_plan() {
        let strategy = TimelineRendererPlatformStrategy::LinuxWorkerThreadVerified;
        let plan = strategy.runner_startup_plan();

        let TimelineRendererRunnerStartupPlan::MinimalWindowProofCandidate {
            threading_model,
            config,
        } = plan
        else {
            panic!("expected minimal window proof candidate");
        };

        assert_eq!(threading_model, DesktopRendererThreadingModel::WorkerThread);
        assert_eq!(config.title, "Eidetic Timeline");
        assert!(config.run_on_any_thread);
    }

    #[test]
    fn timeline_renderer_platform_strategy_keeps_unsupported_platform_pending_only() {
        let plan = TimelineRendererPlatformStrategy::UnsupportedPlatform.runner_startup_plan();

        assert_eq!(
            plan,
            TimelineRendererRunnerStartupPlan::PendingOnly {
                threading_model: DesktopRendererThreadingModel::Unsupported
            }
        );
    }

    fn expected_capability() -> DesktopRendererWindowCapability {
        match current_desktop_renderer_window_platform() {
            DesktopRendererWindowPlatform::Linux => {
                DesktopRendererWindowCapability::VerifiedSupport
            }
            DesktopRendererWindowPlatform::Macos | DesktopRendererWindowPlatform::Windows => {
                DesktopRendererWindowCapability::PlatformUnproven
            }
            DesktopRendererWindowPlatform::Unsupported => {
                DesktopRendererWindowCapability::PlatformUnsupported
            }
        }
    }

    fn expected_capability_reason() -> DesktopRendererWindowCapabilityReason {
        match current_desktop_renderer_window_platform() {
            DesktopRendererWindowPlatform::Linux => {
                DesktopRendererWindowCapabilityReason::VerifiedSupport
            }
            DesktopRendererWindowPlatform::Macos | DesktopRendererWindowPlatform::Windows => {
                DesktopRendererWindowCapabilityReason::PlatformUnproven
            }
            DesktopRendererWindowPlatform::Unsupported => {
                DesktopRendererWindowCapabilityReason::PlatformUnsupported
            }
        }
    }

    fn expected_verified_support() -> bool {
        matches!(
            current_desktop_renderer_window_platform(),
            DesktopRendererWindowPlatform::Linux
        )
    }

    fn expected_visible_window_supported() -> bool {
        expected_verified_support()
    }
}
