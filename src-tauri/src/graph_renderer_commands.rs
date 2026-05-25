use eidetic_bevy_bible_graph::{BibleGraphCameraCommand, BibleGraphVisualSnapshot};
use eidetic_core::contracts::BibleRenderGraphProjectionRequest;
use serde::Deserialize;
use tauri::Manager;

use crate::bevy_graph_host::{BibleGraphHostStatus, DesktopBibleGraphRendererOwner};
use crate::error::CommandError;
use crate::graph_renderer_projection::{
    GraphRendererProjectionOwner, update_active_graph_renderer_projection_request,
};

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
    match graph_renderer_projection_owner(&app)?
        .seed(&app, projection_request)
        .await
    {
        Ok(status) => Ok(status),
        Err(error) => {
            let _ = graph_renderer_owner(&app)?.close_renderer();
            Err(error)
        }
    }
}

#[tauri::command]
pub fn graph_renderer_focus(app: tauri::AppHandle) -> Result<BibleGraphHostStatus, CommandError> {
    graph_renderer_owner(&app)?
        .focus_renderer()
        .map_err(|error| CommandError::internal(format!("graph renderer focus failed: {error:?}")))
}

#[tauri::command]
pub fn graph_renderer_close(app: tauri::AppHandle) -> Result<BibleGraphHostStatus, CommandError> {
    let status = graph_renderer_owner(&app)?
        .close_renderer()
        .map_err(|error| {
            CommandError::internal(format!("graph renderer close failed: {error:?}"))
        })?;
    graph_renderer_projection_owner(&app)?.reset();
    Ok(status)
}

#[tauri::command]
pub fn graph_renderer_status(app: tauri::AppHandle) -> Result<BibleGraphHostStatus, CommandError> {
    graph_renderer_owner(&app)?
        .status()
        .map_err(|error| CommandError::internal(format!("graph renderer status failed: {error:?}")))
}

#[tauri::command]
pub async fn graph_renderer_update_projection_request(
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

    update_active_graph_renderer_projection_request(&app, request).await
}

#[tauri::command]
pub fn graph_renderer_camera_command(
    app: tauri::AppHandle,
    command: BibleGraphCameraCommand,
) -> Result<BibleGraphHostStatus, CommandError> {
    graph_renderer_owner(&app)?
        .apply_camera_command(command)
        .map_err(|error| {
            CommandError::internal(format!("graph renderer camera command failed: {error:?}"))
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
    use eidetic_bevy_bible_graph::BibleGraphCameraCommand;
    use eidetic_core::contracts::BibleGraphNodeId;
    use serde_json::json;

    use super::{
        OpenGraphRendererRequest, RendererWindowSizeHint, validate_renderer_window_size_hint,
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
    fn camera_command_deserializes_backend_owned_viewport_intent() {
        let command: BibleGraphCameraCommand = serde_json::from_value(json!({
            "type": "frame_node",
            "node_id": "node.character.ada"
        }))
        .unwrap();

        assert_eq!(
            command,
            BibleGraphCameraCommand::FrameNode {
                node_id: BibleGraphNodeId::new("node.character.ada").unwrap()
            }
        );
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

fn graph_renderer_projection_owner(
    app: &tauri::AppHandle,
) -> Result<tauri::State<'_, GraphRendererProjectionOwner>, CommandError> {
    app.try_state::<GraphRendererProjectionOwner>()
        .ok_or_else(|| CommandError::internal("graph renderer projection owner is not managed"))
}
