use std::sync::Mutex;

use eidetic_core::contracts::{BibleRenderGraphProjection, BibleRenderGraphProjectionRequest};
use eidetic_server::bible_render_graph_projection;
use eidetic_server::state::AppState;
use tauri::Manager;

use crate::bevy_graph_host::{BibleGraphHostStatus, DesktopBibleGraphRendererOwner};
use crate::error::CommandError;

#[derive(Default)]
pub struct GraphRendererProjectionRequestState {
    state: Mutex<GraphRendererProjectionState>,
}

impl GraphRendererProjectionRequestState {
    pub fn current(&self) -> BibleRenderGraphProjectionRequest {
        self.state
            .lock()
            .unwrap_or_else(|error| error.into_inner())
            .request
            .clone()
    }

    pub fn replace(&self, request: BibleRenderGraphProjectionRequest) {
        self.state
            .lock()
            .unwrap_or_else(|error| error.into_inner())
            .request = request;
    }

    pub fn reset(&self) {
        *self.state.lock().unwrap_or_else(|error| error.into_inner()) =
            GraphRendererProjectionState::default();
    }

    fn begin_refresh(&self) -> GraphRendererProjectionRefreshDecision {
        self.state
            .lock()
            .unwrap_or_else(|error| error.into_inner())
            .refresh
            .begin_refresh()
    }

    fn complete_refresh(&self) -> GraphRendererProjectionRefreshCompletion {
        self.state
            .lock()
            .unwrap_or_else(|error| error.into_inner())
            .refresh
            .complete_refresh()
    }
}

#[derive(Debug, Default)]
struct GraphRendererProjectionState {
    request: BibleRenderGraphProjectionRequest,
    refresh: GraphRendererProjectionRefreshState,
}

#[derive(Debug, Default)]
struct GraphRendererProjectionRefreshState {
    in_flight: bool,
    follow_up_requested: bool,
}

impl GraphRendererProjectionRefreshState {
    fn begin_refresh(&mut self) -> GraphRendererProjectionRefreshDecision {
        if self.in_flight {
            self.follow_up_requested = true;
            GraphRendererProjectionRefreshDecision::AlreadyRefreshing
        } else {
            self.in_flight = true;
            self.follow_up_requested = false;
            GraphRendererProjectionRefreshDecision::Started
        }
    }

    fn complete_refresh(&mut self) -> GraphRendererProjectionRefreshCompletion {
        if self.follow_up_requested {
            self.follow_up_requested = false;
            GraphRendererProjectionRefreshCompletion::RunFollowUp
        } else {
            self.in_flight = false;
            GraphRendererProjectionRefreshCompletion::Idle
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum GraphRendererProjectionRefreshDecision {
    Started,
    AlreadyRefreshing,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum GraphRendererProjectionRefreshCompletion {
    Idle,
    RunFollowUp,
}

pub async fn seed_graph_renderer_projection(
    app: &tauri::AppHandle,
    state: &AppState,
    request: BibleRenderGraphProjectionRequest,
) -> Result<BibleGraphHostStatus, CommandError> {
    let projection = load_graph_renderer_projection(state, request).await?;
    write_graph_renderer_projection(app, projection, GraphRendererProjectionWriteMode::Seed)
}

pub async fn refresh_open_graph_renderer_projection(
    app: &tauri::AppHandle,
    state: &AppState,
    request: BibleRenderGraphProjectionRequest,
) -> Result<BibleGraphHostStatus, CommandError> {
    let status = graph_renderer_owner(app)?.status().map_err(|error| {
        CommandError::internal(format!(
            "graph renderer projection status failed: {error:?}"
        ))
    })?;
    if !status.renderer_window_open {
        return Ok(status);
    }

    let projection = load_graph_renderer_projection(state, request).await?;
    write_graph_renderer_projection(
        app,
        projection,
        GraphRendererProjectionWriteMode::UpdateOpen,
    )
}

pub async fn refresh_active_graph_renderer_projection(
    app: &tauri::AppHandle,
    state: &AppState,
) -> Result<BibleGraphHostStatus, CommandError> {
    let request_state = graph_renderer_projection_request_state(app)?;
    refresh_active_graph_renderer_projection_with_state(app, state, &request_state).await
}

pub async fn update_active_graph_renderer_projection_request(
    app: &tauri::AppHandle,
    state: &AppState,
    request: BibleRenderGraphProjectionRequest,
) -> Result<BibleGraphHostStatus, CommandError> {
    let request_state = graph_renderer_projection_request_state(app)?;
    request_state.replace(request);
    refresh_active_graph_renderer_projection_with_state(app, state, &request_state).await
}

async fn refresh_active_graph_renderer_projection_with_state(
    app: &tauri::AppHandle,
    state: &AppState,
    request_state: &GraphRendererProjectionRequestState,
) -> Result<BibleGraphHostStatus, CommandError> {
    if request_state.begin_refresh() == GraphRendererProjectionRefreshDecision::AlreadyRefreshing {
        return graph_renderer_status(app);
    }

    loop {
        let request = request_state.current();
        let result = refresh_open_graph_renderer_projection(app, state, request).await;
        if request_state.complete_refresh() == GraphRendererProjectionRefreshCompletion::Idle {
            return result;
        }
    }
}

async fn load_graph_renderer_projection(
    state: &AppState,
    request: BibleRenderGraphProjectionRequest,
) -> Result<BibleRenderGraphProjection, CommandError> {
    let envelope = bible_render_graph_projection::bible_render_graph_projection(state, request)
        .await
        .map_err(CommandError::from)?;
    Ok(envelope.payload)
}

fn write_graph_renderer_projection(
    app: &tauri::AppHandle,
    projection: BibleRenderGraphProjection,
    mode: GraphRendererProjectionWriteMode,
) -> Result<BibleGraphHostStatus, CommandError> {
    let result = match mode {
        GraphRendererProjectionWriteMode::Seed => {
            graph_renderer_owner(app)?.set_projection(projection)
        }
        GraphRendererProjectionWriteMode::UpdateOpen => {
            graph_renderer_owner(app)?.update_projection_if_open(projection)
        }
    };

    result.map_err(|error| {
        CommandError::internal(format!("graph renderer projection write failed: {error:?}"))
    })
}

fn graph_renderer_owner(
    app: &tauri::AppHandle,
) -> Result<tauri::State<'_, DesktopBibleGraphRendererOwner>, CommandError> {
    app.try_state::<DesktopBibleGraphRendererOwner>()
        .ok_or_else(|| CommandError::internal("graph renderer owner is not managed"))
}

fn graph_renderer_status(app: &tauri::AppHandle) -> Result<BibleGraphHostStatus, CommandError> {
    graph_renderer_owner(app)?
        .status()
        .map_err(|error| CommandError::internal(format!("graph renderer status failed: {error:?}")))
}

fn graph_renderer_projection_request_state(
    app: &tauri::AppHandle,
) -> Result<tauri::State<'_, GraphRendererProjectionRequestState>, CommandError> {
    app.try_state::<GraphRendererProjectionRequestState>()
        .ok_or_else(|| CommandError::internal("graph renderer projection request is not managed"))
}

enum GraphRendererProjectionWriteMode {
    Seed,
    UpdateOpen,
}

#[cfg(test)]
mod tests {
    use eidetic_core::contracts::{BibleGraphNodeId, BibleRenderGraphProjectionRequest};

    use super::{
        GraphRendererProjectionRefreshCompletion, GraphRendererProjectionRefreshDecision,
        GraphRendererProjectionRefreshState, GraphRendererProjectionRequestState,
    };

    #[test]
    fn graph_renderer_projection_request_state_tracks_active_request() {
        let state = GraphRendererProjectionRequestState::default();
        let request = eidetic_core::contracts::BibleRenderGraphProjectionRequest {
            selected_node_id: Some(BibleGraphNodeId::new("node.character.ada").unwrap()),
            ..Default::default()
        };

        state.replace(request.clone());

        assert_eq!(state.current(), request);
    }

    #[test]
    fn graph_renderer_projection_request_state_resets_request_and_refresh_state() {
        let state = GraphRendererProjectionRequestState::default();
        let request = eidetic_core::contracts::BibleRenderGraphProjectionRequest {
            selected_node_id: Some(BibleGraphNodeId::new("node.character.ada").unwrap()),
            ..Default::default()
        };

        state.replace(request);
        assert_eq!(
            state.begin_refresh(),
            GraphRendererProjectionRefreshDecision::Started
        );
        assert_eq!(
            state.begin_refresh(),
            GraphRendererProjectionRefreshDecision::AlreadyRefreshing
        );

        state.reset();

        assert_eq!(
            state.current(),
            BibleRenderGraphProjectionRequest::default()
        );
        assert_eq!(
            state.begin_refresh(),
            GraphRendererProjectionRefreshDecision::Started
        );
    }

    #[test]
    fn graph_renderer_projection_refresh_state_coalesces_follow_up_requests() {
        let mut state = GraphRendererProjectionRefreshState::default();

        assert_eq!(
            state.begin_refresh(),
            GraphRendererProjectionRefreshDecision::Started
        );
        assert_eq!(
            state.begin_refresh(),
            GraphRendererProjectionRefreshDecision::AlreadyRefreshing
        );
        assert_eq!(
            state.begin_refresh(),
            GraphRendererProjectionRefreshDecision::AlreadyRefreshing
        );
        assert_eq!(
            state.complete_refresh(),
            GraphRendererProjectionRefreshCompletion::RunFollowUp
        );
        assert_eq!(
            state.complete_refresh(),
            GraphRendererProjectionRefreshCompletion::Idle
        );
        assert_eq!(
            state.begin_refresh(),
            GraphRendererProjectionRefreshDecision::Started
        );
    }
}
