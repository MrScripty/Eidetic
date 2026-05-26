#[derive(Debug, Clone, Copy, PartialEq, Eq, serde::Serialize)]
#[serde(rename_all = "snake_case")]
pub enum DesktopRendererWindowKind {
    BibleGraph,
    Timeline,
}
