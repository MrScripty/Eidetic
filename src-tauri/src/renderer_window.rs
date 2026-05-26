#[derive(Debug, Clone, Copy, PartialEq, Eq, serde::Serialize)]
#[serde(rename_all = "snake_case")]
pub enum DesktopRendererWindowKind {
    BibleGraph,
    Timeline,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, serde::Serialize)]
#[serde(rename_all = "snake_case")]
pub enum DesktopRendererWindowLifecycle {
    Closed,
    SceneStarting,
    SceneReadyPendingNativeRunner,
    Visible,
}

impl DesktopRendererWindowLifecycle {
    pub fn from_state(running: bool, scene_ready: bool, window_visible: bool) -> Self {
        match (running, scene_ready, window_visible) {
            (true, true, true) => Self::Visible,
            (true, true, false) => Self::SceneReadyPendingNativeRunner,
            (true, false, _) => Self::SceneStarting,
            (false, _, _) => Self::Closed,
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, serde::Serialize)]
#[serde(rename_all = "snake_case")]
pub enum DesktopRendererRunnerLifecycle {
    Closed,
    OpenRequested,
    Visible,
}
