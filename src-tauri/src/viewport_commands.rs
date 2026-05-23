use tauri::Manager;

use crate::bevy_graph_host::DesktopBibleGraphRendererOwner;
use crate::embedded_viewport_host::{
    EmbeddedViewportHost, EmbeddedViewportHostError, EmbeddedViewportHostStatus,
    EmbeddedViewportKind, EmbeddedViewportState, MountEmbeddedViewportRequest,
    SetEmbeddedViewportFocusRequest, UpdateEmbeddedViewportBoundsRequest,
};
use crate::embedded_viewport_surface::detect_main_window_surface;
use crate::error::CommandError;
use eidetic_core::contracts::BibleRenderGraphProjectionRequest;
use eidetic_server::bible_render_graph_projection;
use eidetic_server::state::AppState;
use serde::Deserialize;

#[derive(Debug, Clone, Deserialize)]
pub struct MountViewportCommandRequest {
    pub viewport_id: String,
    pub kind: EmbeddedViewportKind,
    pub bounds: crate::embedded_viewport_host::EmbeddedViewportBounds,
    #[serde(default)]
    pub graph_projection_request: Option<BibleRenderGraphProjectionRequest>,
}

#[tauri::command]
pub async fn viewport_mount(
    app: tauri::AppHandle,
    request: MountViewportCommandRequest,
) -> Result<EmbeddedViewportState, CommandError> {
    let graph_projection_request = request.graph_projection_request.clone().unwrap_or_default();
    let mut state = viewport_host(&app)?
        .mount(MountEmbeddedViewportRequest {
            viewport_id: request.viewport_id,
            kind: request.kind,
            bounds: request.bounds,
        })
        .map_err(command_error)?;
    let surface = detect_main_window_surface(&app);
    state = viewport_host(&app)?
        .set_surface_state(state.viewport_id.clone(), surface)
        .map_err(command_error)?;
    if state.kind == EmbeddedViewportKind::Graph
        && let Err(error) = graph_renderer_owner(&app)?.start_renderer()
    {
        let _ = viewport_host(&app)?.unmount(state.viewport_id.clone());
        return Err(CommandError::internal(format!(
            "graph renderer start failed: {error:?}"
        )));
    }
    if state.kind == EmbeddedViewportKind::Graph
        && let Err(error) = seed_graph_renderer_projection(&app, graph_projection_request).await
    {
        let status = viewport_host(&app)?
            .unmount(state.viewport_id.clone())
            .map_err(command_error)?;
        stop_graph_renderer_when_unmounted(&app, &status)?;
        return Err(error);
    }
    Ok(state)
}

#[tauri::command]
pub fn viewport_update_bounds(
    app: tauri::AppHandle,
    request: UpdateEmbeddedViewportBoundsRequest,
) -> Result<EmbeddedViewportState, CommandError> {
    viewport_host(&app)?
        .update_bounds(request)
        .map_err(command_error)
}

#[tauri::command]
pub fn viewport_set_focus(
    app: tauri::AppHandle,
    request: SetEmbeddedViewportFocusRequest,
) -> Result<EmbeddedViewportState, CommandError> {
    viewport_host(&app)?
        .set_focus(request)
        .map_err(command_error)
}

#[tauri::command]
pub fn viewport_unmount(
    app: tauri::AppHandle,
    viewport_id: String,
) -> Result<EmbeddedViewportHostStatus, CommandError> {
    let removed_kind = viewport_host(&app)?
        .status()
        .map_err(command_error)?
        .viewports
        .iter()
        .find(|viewport| viewport.viewport_id == viewport_id)
        .map(|viewport| viewport.kind);
    let status = viewport_host(&app)?
        .unmount(viewport_id)
        .map_err(command_error)?;
    if removed_kind == Some(EmbeddedViewportKind::Graph) {
        stop_graph_renderer_when_unmounted(&app, &status)?;
    }
    Ok(status)
}

#[tauri::command]
pub fn viewport_status(app: tauri::AppHandle) -> Result<EmbeddedViewportHostStatus, CommandError> {
    viewport_host(&app)?.status().map_err(command_error)
}

fn viewport_host(
    app: &tauri::AppHandle,
) -> Result<tauri::State<'_, EmbeddedViewportHost>, CommandError> {
    app.try_state::<EmbeddedViewportHost>()
        .ok_or_else(|| CommandError::internal("embedded viewport host is not managed"))
}

fn graph_renderer_owner(
    app: &tauri::AppHandle,
) -> Result<tauri::State<'_, DesktopBibleGraphRendererOwner>, CommandError> {
    app.try_state::<DesktopBibleGraphRendererOwner>()
        .ok_or_else(|| CommandError::internal("graph renderer owner is not managed"))
}

fn stop_graph_renderer_when_unmounted(
    app: &tauri::AppHandle,
    status: &EmbeddedViewportHostStatus,
) -> Result<(), CommandError> {
    if status
        .viewports
        .iter()
        .any(|viewport| viewport.kind == EmbeddedViewportKind::Graph)
    {
        return Ok(());
    }

    graph_renderer_owner(app)?
        .stop()
        .map(|_| ())
        .map_err(|error| CommandError::internal(format!("graph renderer stop failed: {error:?}")))
}

async fn seed_graph_renderer_projection(
    app: &tauri::AppHandle,
    request: BibleRenderGraphProjectionRequest,
) -> Result<(), CommandError> {
    let state = app.state::<AppState>().inner().clone();
    let envelope = bible_render_graph_projection::bible_render_graph_projection(&state, request)
        .await
        .map_err(CommandError::from)?;

    graph_renderer_owner(app)?
        .set_projection(envelope.payload)
        .map(|_| ())
        .map_err(|error| {
            CommandError::internal(format!("graph renderer projection seed failed: {error:?}"))
        })
}

fn command_error(error: EmbeddedViewportHostError) -> CommandError {
    match error {
        EmbeddedViewportHostError::InvalidViewportId(_)
        | EmbeddedViewportHostError::InvalidBounds(_)
        | EmbeddedViewportHostError::ViewportAlreadyMounted(_)
        | EmbeddedViewportHostError::ViewportNotMounted(_) => {
            CommandError::bad_request(error.to_string())
        }
        EmbeddedViewportHostError::LockPoisoned => CommandError::internal(error.to_string()),
    }
}
