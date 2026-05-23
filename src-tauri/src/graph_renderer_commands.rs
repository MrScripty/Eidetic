use eidetic_bevy_bible_graph::{BibleGraphRendererCommand, BibleGraphVisualSnapshot};
use eidetic_core::contracts::BibleRenderGraphProjectionRequest;
use eidetic_server::bible_render_graph_projection;
use eidetic_server::state::AppState;
use serde::Deserialize;
use tauri::Manager;

use crate::bevy_graph_host::{BibleGraphHostStatus, DesktopBibleGraphRendererOwner};
use crate::error::CommandError;

#[derive(Debug, Clone, Deserialize)]
pub struct OpenGraphRendererRequest {
    #[serde(default)]
    pub graph_projection_request: Option<BibleRenderGraphProjectionRequest>,
}

#[tauri::command]
pub async fn graph_renderer_open(
    app: tauri::AppHandle,
    request: OpenGraphRendererRequest,
) -> Result<BibleGraphHostStatus, CommandError> {
    graph_renderer_owner(&app)?
        .start_renderer()
        .map_err(|error| {
            CommandError::internal(format!("graph renderer open failed: {error:?}"))
        })?;
    match seed_graph_renderer_projection(&app, request.graph_projection_request.unwrap_or_default())
        .await
    {
        Ok(status) => Ok(status),
        Err(error) => {
            let _ = graph_renderer_owner(&app)?.stop();
            Err(error)
        }
    }
}

#[tauri::command]
pub fn graph_renderer_focus(app: tauri::AppHandle) -> Result<BibleGraphHostStatus, CommandError> {
    graph_renderer_owner(&app)?
        .start_renderer()
        .map_err(|error| CommandError::internal(format!("graph renderer focus failed: {error:?}")))
}

#[tauri::command]
pub fn graph_renderer_close(app: tauri::AppHandle) -> Result<BibleGraphHostStatus, CommandError> {
    graph_renderer_owner(&app)?
        .stop()
        .map_err(|error| CommandError::internal(format!("graph renderer close failed: {error:?}")))
}

#[tauri::command]
pub fn graph_renderer_status(app: tauri::AppHandle) -> Result<BibleGraphHostStatus, CommandError> {
    graph_renderer_owner(&app)?
        .status()
        .map_err(|error| CommandError::internal(format!("graph renderer status failed: {error:?}")))
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
