use super::{NativeRendererPlatformStrategy, current_renderer_window_platform};
pub use crate::renderer_window::DesktopRendererWindowCapability as BibleGraphRendererWindowCapability;
pub use crate::renderer_window::DesktopRendererWindowCapabilityReason as BibleGraphRendererWindowCapabilityReason;
pub use crate::renderer_window::DesktopRendererWindowLifecycle as BibleGraphRendererWindowLifecycle;
pub use crate::renderer_window::DesktopRendererWindowPlatform as BibleGraphRendererWindowPlatform;
pub use crate::renderer_window::DesktopRendererWindowStrategy as BibleGraphRendererWindowStrategy;

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
        current_renderer_window_platform()
    }
}
