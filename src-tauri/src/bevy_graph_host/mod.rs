mod host;
mod native_runner;
mod owner;
mod window_strategy;

use crate::renderer_window::DesktopRendererWindowKind;

pub use host::DesktopBibleGraphHost;
pub use native_runner::{
    NATIVE_RENDERER_RUNNER_COMMAND_QUEUE_CAPACITY, NATIVE_RENDERER_RUNNER_REPLY_TIMEOUT_MS,
    NativeRendererRunner, NativeRendererRunnerHandle, NativeRendererRunnerStatus,
    PendingNativeRendererRunner,
};
pub use owner::{DesktopBibleGraphRendererOwner, GRAPH_RENDERER_COMMAND_QUEUE_CAPACITY};
pub use window_strategy::{
    BibleGraphRendererWindowCapability, BibleGraphRendererWindowLifecycle,
    BibleGraphRendererWindowStrategy, BibleGraphRendererWindowStrategyStatus,
};

#[cfg(test)]
mod tests;

#[derive(Debug, Clone, PartialEq, Eq, serde::Serialize)]
pub struct BibleGraphHostStatus {
    pub renderer_window_kind: DesktopRendererWindowKind,
    pub running: bool,
    pub renderer_window_open: bool,
    pub renderer_scene_ready: bool,
    pub renderer_window_visible: bool,
    pub renderer_window_strategy: BibleGraphRendererWindowStrategy,
    pub renderer_window_capability: BibleGraphRendererWindowCapability,
    pub renderer_window_lifecycle: BibleGraphRendererWindowLifecycle,
    pub renderer_window_ready: bool,
    pub renderer_window_visible_supported: bool,
    pub renderer_window_focus_supported: bool,
    pub renderer_window_message: String,
    pub node_count: usize,
    pub edge_count: usize,
    pub native_visual_node_count: usize,
    pub native_visual_edge_count: usize,
    pub renderer_window_width_px: u32,
    pub renderer_window_height_px: u32,
    pub influence_count: usize,
    pub last_error: Option<String>,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum BibleGraphHostError {
    Renderer(String),
    RendererPanic,
    InvalidRendererWindowBounds { width_px: u32, height_px: u32 },
    QueueFull,
    OwnerStopped,
}

pub(super) type BibleGraphHostResult<T> = Result<T, BibleGraphHostError>;
