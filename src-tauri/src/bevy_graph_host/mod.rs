mod host;
mod owner;

pub use host::DesktopBibleGraphHost;
pub use owner::DesktopBibleGraphRendererOwner;

#[cfg(test)]
mod tests;

#[derive(Debug, Clone, PartialEq, Eq, serde::Serialize)]
pub struct BibleGraphHostStatus {
    pub running: bool,
    pub node_count: usize,
    pub edge_count: usize,
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
