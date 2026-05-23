use std::sync::Mutex;

use eidetic_core::contracts::{BibleRenderGraphProjection, BibleRenderGraphProjectionRequest};
use eidetic_server::bible_render_graph_projection;
use eidetic_server::state::AppState;
use tauri::Manager;

use crate::bevy_graph_host::{BibleGraphHostStatus, DesktopBibleGraphRendererOwner};
use crate::error::CommandError;

#[derive(Default)]
pub struct GraphRendererProjectionRequestState {
    request: Mutex<BibleRenderGraphProjectionRequest>,
}

impl GraphRendererProjectionRequestState {
    pub fn current(&self) -> BibleRenderGraphProjectionRequest {
        self.request
            .lock()
            .unwrap_or_else(|error| error.into_inner())
            .clone()
    }

    pub fn replace(&self, request: BibleRenderGraphProjectionRequest) {
        *self
            .request
            .lock()
            .unwrap_or_else(|error| error.into_inner()) = request;
    }
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
    let request = graph_renderer_projection_request_state(app)
        .map(|request_state| request_state.current())
        .unwrap_or_default();

    refresh_open_graph_renderer_projection(app, state, request).await
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
    use eidetic_core::contracts::BibleGraphNodeId;

    use super::GraphRendererProjectionRequestState;

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
}
