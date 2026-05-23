use eidetic_bevy_bible_graph::{BibleGraphRendererCommand, BibleGraphVisualSnapshot};
use tauri::Manager;

use crate::bevy_graph_host::{BibleGraphHostStatus, DesktopBibleGraphRendererOwner};
use crate::error::CommandError;

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
