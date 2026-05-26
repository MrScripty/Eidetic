use super::{NativeRendererPlatformStrategy, current_renderer_window_platform};
pub use crate::renderer_window::DesktopRendererWindowLifecycle as BibleGraphRendererWindowLifecycle;

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
    PlatformUnproven,
    PlatformUnsupported,
    RunnerError,
    VerifiedSupport,
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

impl BibleGraphRendererWindowCapability {
    pub fn verified_support(self) -> bool {
        matches!(self, Self::VerifiedSupport)
    }

    pub fn visible_window_supported(self) -> bool {
        matches!(self, Self::VerifiedSupport)
    }
}

impl BibleGraphRendererWindowPlatform {
    pub fn current() -> Self {
        current_renderer_window_platform()
    }
}
