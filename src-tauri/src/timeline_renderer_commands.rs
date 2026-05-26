use eidetic_server::projection_service;
use eidetic_server::state::AppState;
use tauri::Manager;

use crate::bevy_timeline_host::{DesktopTimelineRendererOwner, TimelineHostStatus};
use crate::error::CommandError;

#[tauri::command]
pub async fn timeline_renderer_open(
    app: tauri::AppHandle,
) -> Result<TimelineHostStatus, CommandError> {
    let state = app.state::<AppState>().inner().clone();
    let projection = projection_service::timeline_render_projection(&state)
        .await
        .map_err(CommandError::from)?;
    timeline_host(&app)?
        .set_projection(projection.payload)
        .map_err(|error| {
            CommandError::internal(format!("timeline renderer open failed: {error:?}"))
        })
}

#[tauri::command]
pub fn timeline_renderer_status(app: tauri::AppHandle) -> Result<TimelineHostStatus, CommandError> {
    timeline_host(&app)?.status().map_err(|error| {
        CommandError::internal(format!("timeline renderer status failed: {error:?}"))
    })
}

#[tauri::command]
pub fn timeline_renderer_focus(app: tauri::AppHandle) -> Result<TimelineHostStatus, CommandError> {
    timeline_host(&app)?.focus_renderer().map_err(|error| {
        CommandError::internal(format!("timeline renderer focus failed: {error:?}"))
    })
}

#[tauri::command]
pub fn timeline_renderer_close(app: tauri::AppHandle) -> Result<TimelineHostStatus, CommandError> {
    timeline_host(&app)?.close_renderer().map_err(|error| {
        CommandError::internal(format!("timeline renderer close failed: {error:?}"))
    })
}

fn timeline_host(
    app: &tauri::AppHandle,
) -> Result<tauri::State<'_, DesktopTimelineRendererOwner>, CommandError> {
    app.try_state::<DesktopTimelineRendererOwner>()
        .ok_or_else(|| CommandError::internal("timeline renderer host is not managed"))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn timeline_renderer_host_status_starts_closed() {
        let host = DesktopTimelineRendererOwner::start().unwrap();
        let status = host.status().unwrap();
        let focused = host.focus_renderer().unwrap();
        let stopped = host.stop().unwrap();

        assert!(!status.running);
        assert_eq!(status.clip_count, 0);
        assert!(!focused.renderer_window_focus_supported);
        assert!(!stopped.running);
    }
}
