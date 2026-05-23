#[derive(Debug, Clone, Copy, PartialEq, Eq, serde::Serialize)]
#[serde(rename_all = "snake_case")]
pub enum BibleGraphRendererWindowStrategy {
    BevyWinitFloatingWindow,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, serde::Serialize)]
#[serde(rename_all = "snake_case")]
pub enum BibleGraphRendererWindowCapability {
    PendingNativeRunner,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, serde::Serialize)]
pub struct BibleGraphRendererWindowStrategyStatus {
    pub strategy: BibleGraphRendererWindowStrategy,
    pub capability: BibleGraphRendererWindowCapability,
    pub visible_window_supported: bool,
}

impl BibleGraphRendererWindowStrategyStatus {
    pub fn current() -> Self {
        Self {
            strategy: BibleGraphRendererWindowStrategy::BevyWinitFloatingWindow,
            capability: BibleGraphRendererWindowCapability::PendingNativeRunner,
            visible_window_supported: false,
        }
    }
}
