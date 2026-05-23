use tauri::Manager;

use crate::bevy_graph_host::DesktopBibleGraphRendererOwner;
use crate::embedded_viewport_host::{
    EmbeddedViewportHost, EmbeddedViewportHostError, EmbeddedViewportHostStatus,
    EmbeddedViewportKind, EmbeddedViewportState, MountEmbeddedViewportRequest,
    SetEmbeddedViewportFocusRequest, UpdateEmbeddedViewportBoundsRequest,
};
use crate::error::CommandError;

#[tauri::command]
pub fn viewport_mount(
    app: tauri::AppHandle,
    request: MountEmbeddedViewportRequest,
) -> Result<EmbeddedViewportState, CommandError> {
    let state = viewport_host(&app)?.mount(request).map_err(command_error)?;
    if state.kind == EmbeddedViewportKind::Graph
        && let Err(error) = graph_renderer_owner(&app)?.start_renderer()
    {
        let _ = viewport_host(&app)?.unmount(state.viewport_id.clone());
        return Err(CommandError::internal(format!(
            "graph renderer start failed: {error:?}"
        )));
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

fn command_error(error: EmbeddedViewportHostError) -> CommandError {
    match error {
        EmbeddedViewportHostError::InvalidViewportId(_)
        | EmbeddedViewportHostError::InvalidBounds(_)
        | EmbeddedViewportHostError::ViewportNotMounted(_) => {
            CommandError::bad_request(error.to_string())
        }
        EmbeddedViewportHostError::LockPoisoned => CommandError::internal(error.to_string()),
    }
}
