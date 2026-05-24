use std::sync::Mutex;

use eidetic_core::contracts::{BibleRenderGraphProjection, BibleRenderGraphProjectionRequest};
use eidetic_server::bible_render_graph_projection;
use eidetic_server::state::AppState;
use tauri::Manager;

use crate::bevy_graph_host::{BibleGraphHostStatus, DesktopBibleGraphRendererOwner};
use crate::error::CommandError;

pub struct GraphRendererProjectionOwner {
    app_state: AppState,
    state: Mutex<GraphRendererProjectionState>,
}

impl GraphRendererProjectionOwner {
    pub fn new(app_state: AppState) -> Self {
        Self {
            app_state,
            state: Mutex::new(GraphRendererProjectionState::default()),
        }
    }

    pub async fn seed(
        &self,
        app: &tauri::AppHandle,
        request: BibleRenderGraphProjectionRequest,
    ) -> Result<BibleGraphHostStatus, CommandError> {
        self.replace_request(request.clone());
        let projection = self.load_projection(request).await?;
        write_graph_renderer_projection(app, projection, GraphRendererProjectionWriteMode::Seed)
    }

    pub async fn replace_request_and_refresh(
        &self,
        app: &tauri::AppHandle,
        request: BibleRenderGraphProjectionRequest,
    ) -> Result<BibleGraphHostStatus, CommandError> {
        self.replace_request(request);
        self.refresh_active(app).await
    }

    pub async fn refresh_active(
        &self,
        app: &tauri::AppHandle,
    ) -> Result<BibleGraphHostStatus, CommandError> {
        if self.begin_refresh() == GraphRendererProjectionRefreshDecision::AlreadyRefreshing {
            return graph_renderer_status(app);
        }

        loop {
            let request = self.current_request();
            let result = self.refresh_open(app, request).await;
            if self.complete_refresh() == GraphRendererProjectionRefreshCompletion::Idle {
                return result;
            }
        }
    }

    pub fn reset(&self) {
        *self.state.lock().unwrap_or_else(|error| error.into_inner()) =
            GraphRendererProjectionState::default();
    }

    fn current_request(&self) -> BibleRenderGraphProjectionRequest {
        self.state
            .lock()
            .unwrap_or_else(|error| error.into_inner())
            .request
            .clone()
    }

    fn replace_request(&self, request: BibleRenderGraphProjectionRequest) {
        self.state
            .lock()
            .unwrap_or_else(|error| error.into_inner())
            .request = request;
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

    async fn refresh_open(
        &self,
        app: &tauri::AppHandle,
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

        let projection = self.load_projection(request).await?;
        write_graph_renderer_projection(
            app,
            projection,
            GraphRendererProjectionWriteMode::UpdateOpen,
        )
    }

    async fn load_projection(
        &self,
        request: BibleRenderGraphProjectionRequest,
    ) -> Result<BibleRenderGraphProjection, CommandError> {
        let envelope =
            bible_render_graph_projection::bible_render_graph_projection(&self.app_state, request)
                .await
                .map_err(CommandError::from)?;
        Ok(envelope.payload)
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

pub async fn refresh_active_graph_renderer_projection(
    app: &tauri::AppHandle,
) -> Result<BibleGraphHostStatus, CommandError> {
    graph_renderer_projection_owner(app)?
        .refresh_active(app)
        .await
}

pub async fn update_active_graph_renderer_projection_request(
    app: &tauri::AppHandle,
    request: BibleRenderGraphProjectionRequest,
) -> Result<BibleGraphHostStatus, CommandError> {
    graph_renderer_projection_owner(app)?
        .replace_request_and_refresh(app, request)
        .await
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

fn graph_renderer_projection_owner(
    app: &tauri::AppHandle,
) -> Result<tauri::State<'_, GraphRendererProjectionOwner>, CommandError> {
    app.try_state::<GraphRendererProjectionOwner>()
        .ok_or_else(|| CommandError::internal("graph renderer projection owner is not managed"))
}

enum GraphRendererProjectionWriteMode {
    Seed,
    UpdateOpen,
}

#[cfg(test)]
mod tests {
    use eidetic_core::contracts::{BibleGraphNodeId, BibleRenderGraphProjectionRequest};

    use super::{
        GraphRendererProjectionOwner, GraphRendererProjectionRefreshCompletion,
        GraphRendererProjectionRefreshDecision, GraphRendererProjectionRefreshState,
    };

    #[test]
    fn graph_renderer_projection_owner_tracks_active_request() {
        let app_state = tauri::async_runtime::block_on(eidetic_server::state::AppState::new());
        let state = GraphRendererProjectionOwner::new(app_state);
        let request = eidetic_core::contracts::BibleRenderGraphProjectionRequest {
            selected_node_id: Some(BibleGraphNodeId::new("node.character.ada").unwrap()),
            ..Default::default()
        };

        state.replace_request(request.clone());

        assert_eq!(state.current_request(), request);
        state.app_state.shutdown_tasks();
    }

    #[test]
    fn graph_renderer_projection_owner_resets_request_and_refresh_state() {
        let app_state = tauri::async_runtime::block_on(eidetic_server::state::AppState::new());
        let state = GraphRendererProjectionOwner::new(app_state);
        let request = eidetic_core::contracts::BibleRenderGraphProjectionRequest {
            selected_node_id: Some(BibleGraphNodeId::new("node.character.ada").unwrap()),
            ..Default::default()
        };

        state.replace_request(request);
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
            state.current_request(),
            BibleRenderGraphProjectionRequest::default()
        );
        assert_eq!(
            state.begin_refresh(),
            GraphRendererProjectionRefreshDecision::Started
        );
        state.app_state.shutdown_tasks();
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
