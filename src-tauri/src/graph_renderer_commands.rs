use std::sync::Mutex;

use eidetic_bevy_bible_graph::{BibleGraphRendererCommand, BibleGraphVisualSnapshot};
use eidetic_core::contracts::BibleRenderGraphProjectionRequest;
use eidetic_server::bible_render_graph_projection;
use eidetic_server::state::AppState;
use serde::Deserialize;
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

    fn replace(&self, request: BibleRenderGraphProjectionRequest) {
        *self
            .request
            .lock()
            .unwrap_or_else(|error| error.into_inner()) = request;
    }
}

#[derive(Debug, Clone, Deserialize)]
pub struct OpenGraphRendererRequest {
    #[serde(default)]
    pub graph_projection_request: Option<BibleRenderGraphProjectionRequest>,
    #[serde(default)]
    pub renderer_window_size_hint: Option<RendererWindowSizeHint>,
}

#[derive(Debug, Clone, Copy, Deserialize)]
pub struct RendererWindowSizeHint {
    pub width_px: u32,
    pub height_px: u32,
}

#[tauri::command]
pub async fn graph_renderer_open(
    app: tauri::AppHandle,
    request: OpenGraphRendererRequest,
) -> Result<BibleGraphHostStatus, CommandError> {
    validate_renderer_window_size_hint(request.renderer_window_size_hint)?;
    graph_renderer_owner(&app)?
        .start_renderer()
        .map_err(|error| {
            CommandError::internal(format!("graph renderer open failed: {error:?}"))
        })?;
    if let Err(error) = apply_renderer_window_size_hint(&app, request.renderer_window_size_hint) {
        let _ = graph_renderer_owner(&app)?.close_renderer();
        return Err(error);
    }
    let projection_request = request.graph_projection_request.unwrap_or_default();
    match seed_graph_renderer_projection(&app, projection_request.clone()).await {
        Ok(status) => {
            graph_renderer_projection_request_state(&app)?.replace(projection_request);
            Ok(status)
        }
        Err(error) => {
            let _ = graph_renderer_owner(&app)?.close_renderer();
            Err(error)
        }
    }
}

#[tauri::command]
pub fn graph_renderer_focus(app: tauri::AppHandle) -> Result<BibleGraphHostStatus, CommandError> {
    graph_renderer_owner(&app)?
        .status()
        .map_err(|error| CommandError::internal(format!("graph renderer focus failed: {error:?}")))
}

#[tauri::command]
pub fn graph_renderer_close(app: tauri::AppHandle) -> Result<BibleGraphHostStatus, CommandError> {
    graph_renderer_owner(&app)?
        .close_renderer()
        .map_err(|error| CommandError::internal(format!("graph renderer close failed: {error:?}")))
}

#[tauri::command]
pub fn graph_renderer_status(app: tauri::AppHandle) -> Result<BibleGraphHostStatus, CommandError> {
    graph_renderer_owner(&app)?
        .status()
        .map_err(|error| CommandError::internal(format!("graph renderer status failed: {error:?}")))
}

#[tauri::command]
pub async fn graph_renderer_set_projection(
    app: tauri::AppHandle,
    request: BibleRenderGraphProjectionRequest,
) -> Result<BibleGraphHostStatus, CommandError> {
    let status = graph_renderer_owner(&app)?.status().map_err(|error| {
        CommandError::internal(format!(
            "graph renderer projection status failed: {error:?}"
        ))
    })?;
    if !status.renderer_window_open {
        return Ok(status);
    }

    let status = seed_graph_renderer_projection(&app, request.clone()).await?;
    graph_renderer_projection_request_state(&app)?.replace(request);
    Ok(status)
}

#[tauri::command]
pub fn graph_renderer_drain_commands(
    app: tauri::AppHandle,
) -> Result<Vec<BibleGraphRendererCommand>, CommandError> {
    graph_renderer_owner(&app)?
        .drain_commands()
        .map_err(|error| {
            CommandError::internal(format!("graph renderer command drain failed: {error:?}"))
        })
}

fn apply_renderer_window_size_hint(
    app: &tauri::AppHandle,
    size_hint: Option<RendererWindowSizeHint>,
) -> Result<(), CommandError> {
    let Some(size_hint) = size_hint else {
        return Ok(());
    };

    graph_renderer_owner(app)?
        .set_renderer_window_bounds(size_hint.width_px, size_hint.height_px)
        .map(|_| ())
        .map_err(|error| {
            CommandError::internal(format!("graph renderer size hint failed: {error:?}"))
        })
}

fn validate_renderer_window_size_hint(
    size_hint: Option<RendererWindowSizeHint>,
) -> Result<(), CommandError> {
    let Some(size_hint) = size_hint else {
        return Ok(());
    };

    if size_hint.width_px == 0 || size_hint.height_px == 0 {
        return Err(CommandError::bad_request(
            "graph renderer window size hint must be greater than zero",
        ));
    }

    Ok(())
}

#[cfg(test)]
mod tests {
    use eidetic_core::contracts::BibleGraphNodeId;
    use serde_json::json;

    use super::{
        GraphRendererProjectionRequestState, OpenGraphRendererRequest, RendererWindowSizeHint,
        validate_renderer_window_size_hint,
    };

    #[test]
    fn open_request_deserializes_renderer_window_size_hint() {
        let request: OpenGraphRendererRequest = serde_json::from_value(json!({
            "renderer_window_size_hint": {
                "width_px": 1280,
                "height_px": 720
            }
        }))
        .unwrap();

        let size_hint = request.renderer_window_size_hint.unwrap();
        assert_eq!(size_hint.width_px, 1280);
        assert_eq!(size_hint.height_px, 720);
    }

    #[test]
    fn renderer_window_size_hint_rejects_zero_dimensions() {
        let error = validate_renderer_window_size_hint(Some(RendererWindowSizeHint {
            width_px: 0,
            height_px: 720,
        }))
        .unwrap_err();

        assert_eq!(
            serde_json::to_value(error).unwrap(),
            json!({
                "kind": "bad_request",
                "message": "graph renderer window size hint must be greater than zero"
            })
        );
    }

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

#[tauri::command]
pub fn graph_renderer_visual_snapshot(
    app: tauri::AppHandle,
) -> Result<BibleGraphVisualSnapshot, CommandError> {
    graph_renderer_owner(&app)?
        .visual_snapshot()
        .map_err(|error| {
            CommandError::internal(format!("graph renderer visual snapshot failed: {error:?}"))
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

async fn seed_graph_renderer_projection(
    app: &tauri::AppHandle,
    request: BibleRenderGraphProjectionRequest,
) -> Result<BibleGraphHostStatus, CommandError> {
    let state = app.state::<AppState>().inner().clone();
    let envelope = bible_render_graph_projection::bible_render_graph_projection(&state, request)
        .await
        .map_err(CommandError::from)?;

    graph_renderer_owner(app)?
        .set_projection(envelope.payload)
        .map_err(|error| {
            CommandError::internal(format!("graph renderer projection seed failed: {error:?}"))
        })
}
