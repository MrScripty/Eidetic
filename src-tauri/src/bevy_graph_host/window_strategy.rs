#[derive(Debug, Clone, Copy, PartialEq, Eq, serde::Serialize)]
#[serde(rename_all = "snake_case")]
pub enum BibleGraphRendererWindowStrategy {
    BevyWinitFloatingWindow,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, serde::Serialize)]
#[serde(rename_all = "snake_case")]
pub enum BibleGraphRendererWindowPlatform {
    Linux,
    Macos,
    Windows,
    Unsupported,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, serde::Serialize)]
#[serde(rename_all = "snake_case")]
pub enum BibleGraphRendererWindowCapability {
    PendingNativeRunner,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, serde::Serialize)]
#[serde(rename_all = "snake_case")]
pub enum BibleGraphRendererWindowCapabilityReason {
    PendingNativeRunner,
    PlatformUnproven,
    PlatformUnsupported,
    RunnerError,
    VerifiedSupport,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, serde::Serialize)]
#[serde(rename_all = "snake_case")]
pub enum BibleGraphRendererWindowLifecycle {
    Closed,
    SceneStarting,
    SceneReadyPendingNativeRunner,
    Visible,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, serde::Serialize)]
pub struct BibleGraphRendererWindowStrategyStatus {
    pub strategy: BibleGraphRendererWindowStrategy,
    pub platform: BibleGraphRendererWindowPlatform,
    pub capability: BibleGraphRendererWindowCapability,
    pub capability_reason: BibleGraphRendererWindowCapabilityReason,
    pub visible_window_supported: bool,
}

impl BibleGraphRendererWindowStrategyStatus {
    pub fn current() -> Self {
        let platform = BibleGraphRendererWindowPlatform::current();
        Self {
            strategy: BibleGraphRendererWindowStrategy::BevyWinitFloatingWindow,
            platform,
            capability: BibleGraphRendererWindowCapability::PendingNativeRunner,
            capability_reason: platform.pending_capability_reason(),
            visible_window_supported: false,
        }
    }
}

impl BibleGraphRendererWindowPlatform {
    pub fn current() -> Self {
        if cfg!(target_os = "linux") {
            Self::Linux
        } else if cfg!(target_os = "macos") {
            Self::Macos
        } else if cfg!(target_os = "windows") {
            Self::Windows
        } else {
            Self::Unsupported
        }
    }

    fn pending_capability_reason(self) -> BibleGraphRendererWindowCapabilityReason {
        match self {
            Self::Linux | Self::Macos | Self::Windows => {
                BibleGraphRendererWindowCapabilityReason::PendingNativeRunner
            }
            Self::Unsupported => BibleGraphRendererWindowCapabilityReason::PlatformUnsupported,
        }
    }
}

impl BibleGraphRendererWindowLifecycle {
    pub fn from_state(running: bool, scene_ready: bool, window_visible: bool) -> Self {
        match (running, scene_ready, window_visible) {
            (true, true, true) => Self::Visible,
            (true, true, false) => Self::SceneReadyPendingNativeRunner,
            (true, false, _) => Self::SceneStarting,
            (false, _, _) => Self::Closed,
        }
    }
}
