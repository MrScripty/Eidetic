mod host;
mod owner;

pub use host::DesktopBibleGraphHost;
pub use owner::{DesktopBibleGraphRendererOwner, GRAPH_RENDERER_COMMAND_QUEUE_CAPACITY};

#[cfg(test)]
mod tests;

#[derive(Debug, Clone, PartialEq, Eq, serde::Serialize)]
pub struct BibleGraphHostStatus {
    pub running: bool,
    pub renderer_window_open: bool,
    pub renderer_scene_ready: bool,
    pub renderer_window_visible: bool,
    pub renderer_window_ready: bool,
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
    QueueFull,
    OwnerStopped,
}

pub(super) type BibleGraphHostResult<T> = Result<T, BibleGraphHostError>;
