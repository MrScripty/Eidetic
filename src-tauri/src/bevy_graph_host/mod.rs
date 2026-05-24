mod host;
mod native_runner;
mod owner;
mod platform_strategy;
mod supervisor;
mod window_strategy;

use crate::renderer_window::DesktopRendererWindowKind;

pub use host::DesktopBibleGraphHost;
pub use native_runner::{
    NATIVE_RENDERER_RUNNER_COMMAND_QUEUE_CAPACITY, NATIVE_RENDERER_RUNNER_REPLY_TIMEOUT_MS,
    NativeRendererRunner, NativeRendererRunnerHandle, NativeRendererRunnerLifecycle,
    NativeRendererRunnerStatus,
};
pub use owner::{
    DesktopBibleGraphRendererOwner, GRAPH_RENDERER_COMMAND_QUEUE_CAPACITY,
    GRAPH_RENDERER_REPLY_TIMEOUT_MS,
};
pub use platform_strategy::{
    NativeRendererPlatformStrategy, NativeRendererRunnerStartupPlan, NativeRendererThreadingModel,
    current_renderer_window_platform,
};
pub use supervisor::{NativeRendererSupervisor, NativeRendererSupervisorLifecycle};
pub use window_strategy::{
    BibleGraphRendererWindowCapability, BibleGraphRendererWindowCapabilityReason,
    BibleGraphRendererWindowLifecycle, BibleGraphRendererWindowPlatform,
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
    pub renderer_window_platform: BibleGraphRendererWindowPlatform,
    pub renderer_runner_lifecycle: NativeRendererRunnerLifecycle,
    pub renderer_supervisor_lifecycle: NativeRendererSupervisorLifecycle,
    pub renderer_runner_threading_model: NativeRendererThreadingModel,
    pub renderer_window_capability: BibleGraphRendererWindowCapability,
    pub renderer_window_capability_reason: BibleGraphRendererWindowCapabilityReason,
    pub renderer_window_lifecycle: BibleGraphRendererWindowLifecycle,
    pub renderer_window_ready: bool,
    pub renderer_window_verified_support: bool,
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
    OwnerReplyTimeout,
    OwnerStopped,
}

pub(super) type BibleGraphHostResult<T> = Result<T, BibleGraphHostError>;
