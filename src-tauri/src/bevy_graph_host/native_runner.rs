use super::{
    BibleGraphRendererWindowCapability, BibleGraphRendererWindowStrategy,
    BibleGraphRendererWindowStrategyStatus,
};

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct NativeRendererRunnerStatus {
    pub strategy: BibleGraphRendererWindowStrategy,
    pub capability: BibleGraphRendererWindowCapability,
    pub visible_window_supported: bool,
    pub window_visible: bool,
    pub window_ready: bool,
    pub focus_supported: bool,
}

pub trait NativeRendererRunner {
    fn open(&mut self) -> NativeRendererRunnerStatus;
    fn close(&mut self) -> NativeRendererRunnerStatus;
    fn focus(&mut self) -> NativeRendererRunnerStatus;
    fn status(&self) -> NativeRendererRunnerStatus;
}

#[derive(Debug, Default)]
pub struct PendingNativeRendererRunner {
    open_requested: bool,
}

impl NativeRendererRunner for PendingNativeRendererRunner {
    fn open(&mut self) -> NativeRendererRunnerStatus {
        self.open_requested = true;
        self.status()
    }

    fn close(&mut self) -> NativeRendererRunnerStatus {
        self.open_requested = false;
        self.status()
    }

    fn focus(&mut self) -> NativeRendererRunnerStatus {
        self.status()
    }

    fn status(&self) -> NativeRendererRunnerStatus {
        let strategy = BibleGraphRendererWindowStrategyStatus::current();
        NativeRendererRunnerStatus {
            strategy: strategy.strategy,
            capability: strategy.capability,
            visible_window_supported: strategy.visible_window_supported,
            window_visible: false,
            window_ready: false,
            focus_supported: false,
        }
    }
}

#[cfg(test)]
impl PendingNativeRendererRunner {
    pub fn open_requested(&self) -> bool {
        self.open_requested
    }
}
