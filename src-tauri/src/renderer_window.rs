#[derive(Debug, Clone, Copy, PartialEq, Eq, serde::Serialize)]
#[serde(rename_all = "snake_case")]
pub enum DesktopRendererWindowKind {
    BibleGraph,
    Timeline,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, serde::Serialize)]
#[serde(rename_all = "snake_case")]
pub enum DesktopRendererWindowStrategy {
    BevyWinitFloatingWindow,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, serde::Serialize)]
pub struct DesktopRendererWindowStrategyStatus {
    pub strategy: DesktopRendererWindowStrategy,
    pub platform: DesktopRendererWindowPlatform,
    pub capability: DesktopRendererWindowCapability,
    pub capability_reason: DesktopRendererWindowCapabilityReason,
    pub verified_support: bool,
    pub visible_window_supported: bool,
}

impl DesktopRendererWindowStrategyStatus {
    pub fn pending_native_runner_current_platform() -> Self {
        Self::current_platform_with_capability(
            DesktopRendererWindowCapability::PendingNativeRunner,
            DesktopRendererWindowCapabilityReason::PendingNativeRunner,
        )
    }

    pub fn runner_error_current_platform() -> Self {
        Self::current_platform_with_capability(
            DesktopRendererWindowCapability::RunnerError,
            DesktopRendererWindowCapabilityReason::RunnerError,
        )
    }

    fn current_platform_with_capability(
        capability: DesktopRendererWindowCapability,
        capability_reason: DesktopRendererWindowCapabilityReason,
    ) -> Self {
        Self {
            strategy: DesktopRendererWindowStrategy::BevyWinitFloatingWindow,
            platform: current_desktop_renderer_window_platform(),
            capability,
            capability_reason,
            verified_support: capability.verified_support(),
            visible_window_supported: capability.visible_window_supported(),
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, serde::Serialize)]
#[serde(rename_all = "snake_case")]
pub enum DesktopRendererWindowPlatform {
    Linux,
    Macos,
    Windows,
    Unsupported,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, serde::Serialize)]
#[serde(rename_all = "snake_case")]
pub enum DesktopRendererWindowCapability {
    PendingNativeRunner,
    PlatformUnproven,
    PlatformUnsupported,
    RunnerError,
    VerifiedSupport,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, serde::Serialize)]
#[serde(rename_all = "snake_case")]
pub enum DesktopRendererWindowCapabilityReason {
    PendingNativeRunner,
    PlatformUnproven,
    PlatformUnsupported,
    RunnerError,
    VerifiedSupport,
}

impl DesktopRendererWindowCapability {
    pub fn verified_support(self) -> bool {
        matches!(self, Self::VerifiedSupport)
    }

    pub fn visible_window_supported(self) -> bool {
        matches!(self, Self::VerifiedSupport)
    }
}

pub fn current_desktop_renderer_window_platform() -> DesktopRendererWindowPlatform {
    if cfg!(target_os = "linux") {
        DesktopRendererWindowPlatform::Linux
    } else if cfg!(target_os = "macos") {
        DesktopRendererWindowPlatform::Macos
    } else if cfg!(target_os = "windows") {
        DesktopRendererWindowPlatform::Windows
    } else {
        DesktopRendererWindowPlatform::Unsupported
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, serde::Serialize)]
#[serde(rename_all = "snake_case")]
pub enum DesktopRendererWindowLifecycle {
    Closed,
    SceneStarting,
    SceneReadyPendingNativeRunner,
    Visible,
}

impl DesktopRendererWindowLifecycle {
    pub fn from_state(running: bool, scene_ready: bool, window_visible: bool) -> Self {
        match (running, scene_ready, window_visible) {
            (true, true, true) => Self::Visible,
            (true, true, false) => Self::SceneReadyPendingNativeRunner,
            (true, false, _) => Self::SceneStarting,
            (false, _, _) => Self::Closed,
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, serde::Serialize)]
#[serde(rename_all = "snake_case")]
pub enum DesktopRendererRunnerLifecycle {
    Closed,
    OpenRequested,
    Visible,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, serde::Serialize)]
#[serde(rename_all = "snake_case")]
pub enum DesktopRendererSupervisorLifecycle {
    NotStarted,
    Starting,
    Running,
    Closing,
    Closed,
    Failed,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, serde::Serialize)]
#[serde(rename_all = "snake_case")]
pub enum DesktopRendererThreadingModel {
    WorkerThread,
    MainThread,
    Unsupported,
}
