use super::NativeRendererPlatformStrategy;

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
    pub verified_support: bool,
    pub visible_window_supported: bool,
}

impl BibleGraphRendererWindowStrategyStatus {
    pub fn current() -> Self {
        NativeRendererPlatformStrategy::current().status()
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
