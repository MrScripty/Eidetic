mod host;
mod owner;

pub use host::DesktopBibleGraphHost;
pub use owner::DesktopBibleGraphRendererOwner;

#[cfg(test)]
mod tests;

#[derive(Debug, Clone, PartialEq, Eq, serde::Serialize)]
pub struct BibleGraphHostStatus {
    pub running: bool,
    pub renderer_window_open: bool,
    pub renderer_window_ready: bool,
    pub renderer_window_message: String,
    pub native_panel_ready: bool,
    pub node_count: usize,
    pub edge_count: usize,
    pub native_visual_node_count: usize,
    pub native_visual_edge_count: usize,
    pub native_panel_width_px: u32,
    pub native_panel_height_px: u32,
    pub influence_count: usize,
    pub last_error: Option<String>,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum BibleGraphHostError {
    Renderer(String),
    RendererPanic,
    OwnerStopped,
}

pub(super) type BibleGraphHostResult<T> = Result<T, BibleGraphHostError>;
