use std::fs;
use std::io::ErrorKind;
use std::path::PathBuf;

use eidetic_bevy_bible_graph::{
    BibleGraphCameraCommand, BibleGraphNativeTextEditorSettings, BibleGraphVisualSnapshot,
};
use eidetic_core::contracts::BibleRenderGraphProjectionRequest;
use serde::Deserialize;
use tauri::Manager;

use crate::bevy_graph_host::{BibleGraphHostStatus, DesktopBibleGraphRendererOwner};
use crate::error::CommandError;
use crate::graph_renderer_projection::{
    GraphRendererProjectionOwner, update_active_graph_renderer_projection_request,
};

const GRAPH_RENDERER_SETTINGS_FILE_NAME: &str = "graph-renderer-settings.json";

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

#[tauri::command]
pub fn graph_renderer_text_editor_settings(
    app: tauri::AppHandle,
    settings: BibleGraphNativeTextEditorSettings,
) -> Result<BibleGraphHostStatus, CommandError> {
    validate_text_editor_settings(&settings)?;
    graph_renderer_owner(&app)?
        .apply_text_editor_settings(settings)
        .map_err(|error| {
            CommandError::internal(format!(
                "graph renderer text editor settings failed: {error:?}"
            ))
        })
}

#[tauri::command]
pub fn graph_renderer_text_editor_settings_load(
    app: tauri::AppHandle,
) -> Result<BibleGraphNativeTextEditorSettings, CommandError> {
    load_graph_renderer_text_editor_settings(&app)
}

#[tauri::command]
pub fn graph_renderer_text_editor_settings_save(
    app: tauri::AppHandle,
    settings: BibleGraphNativeTextEditorSettings,
) -> Result<BibleGraphHostStatus, CommandError> {
    validate_text_editor_settings(&settings)?;
    save_graph_renderer_text_editor_settings(&app, &settings)?;
    graph_renderer_owner(&app)?
        .apply_text_editor_settings(settings)
        .map_err(|error| {
            CommandError::internal(format!(
                "graph renderer text editor settings save failed: {error:?}"
            ))
        })
}

pub(crate) fn load_graph_renderer_text_editor_settings(
    app: &tauri::AppHandle,
) -> Result<BibleGraphNativeTextEditorSettings, CommandError> {
    let path = graph_renderer_settings_path(app)?;
    match fs::read_to_string(&path) {
        Ok(contents) => {
            let settings = serde_json::from_str::<BibleGraphNativeTextEditorSettings>(&contents)
                .map_err(|error| {
                    CommandError::internal(format!(
                        "graph renderer settings file is invalid: {error}"
                    ))
                })?;
            validate_text_editor_settings(&settings)?;
            Ok(settings)
        }
        Err(error) if error.kind() == ErrorKind::NotFound => {
            Ok(BibleGraphNativeTextEditorSettings::default())
        }
        Err(error) => Err(CommandError::internal(format!(
            "failed to read graph renderer settings: {error}"
        ))),
    }
}

fn save_graph_renderer_text_editor_settings(
    app: &tauri::AppHandle,
    settings: &BibleGraphNativeTextEditorSettings,
) -> Result<(), CommandError> {
    let path = graph_renderer_settings_path(app)?;
    let Some(parent) = path.parent() else {
        return Err(CommandError::internal(
            "graph renderer settings path has no parent directory",
        ));
    };
    fs::create_dir_all(parent).map_err(|error| {
        CommandError::internal(format!(
            "failed to create graph renderer settings directory: {error}"
        ))
    })?;
    let contents = serde_json::to_string_pretty(settings).map_err(|error| {
        CommandError::internal(format!(
            "failed to serialize graph renderer settings: {error}"
        ))
    })?;
    fs::write(&path, contents).map_err(|error| {
        CommandError::internal(format!("failed to write graph renderer settings: {error}"))
    })
}

fn graph_renderer_settings_path(app: &tauri::AppHandle) -> Result<PathBuf, CommandError> {
    app.path()
        .app_config_dir()
        .map(|directory| directory.join(GRAPH_RENDERER_SETTINGS_FILE_NAME))
        .map_err(|error| {
            CommandError::internal(format!(
                "failed to resolve graph renderer settings directory: {error}"
            ))
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

fn validate_text_editor_settings(
    settings: &BibleGraphNativeTextEditorSettings,
) -> Result<(), CommandError> {
    if !(0.0..=48.0).contains(&settings.padding_px) {
        return Err(CommandError::bad_request(
            "graph renderer text editor padding must be between 0 and 48 pixels",
        ));
    }
    if !(0.0..=16.0).contains(&settings.corner_radius_px) {
        return Err(CommandError::bad_request(
            "graph renderer text editor corner radius must be between 0 and 16 pixels",
        ));
    }
    if !(0.0..=8.0).contains(&settings.editor_outline_width_px) {
        return Err(CommandError::bad_request(
            "graph renderer text editor outline width must be between 0 and 8 pixels",
        ));
    }
    if !(0.0..=1.0).contains(&settings.editor_outline_brightness) {
        return Err(CommandError::bad_request(
            "graph renderer text editor outline brightness must be between 0 and 1",
        ));
    }
    if !(0.0..=1.0).contains(&settings.editor_outline_transparency) {
        return Err(CommandError::bad_request(
            "graph renderer text editor outline transparency must be between 0 and 1",
        ));
    }
    if !(8.0..=40.0).contains(&settings.font_size_px) {
        return Err(CommandError::bad_request(
            "graph renderer text editor font size must be between 8 and 40 pixels",
        ));
    }
    if !(0.0..=1.0).contains(&settings.font_brightness) {
        return Err(CommandError::bad_request(
            "graph renderer text editor font brightness must be between 0 and 1",
        ));
    }
    validate_graph_renderer_hex_color(
        &settings.editor_background_color,
        "graph renderer text editor background color must be a hex color",
    )?;
    if !(0.0..=1.0).contains(&settings.editor_background_brightness) {
        return Err(CommandError::bad_request(
            "graph renderer text editor background brightness must be between 0 and 1",
        ));
    }
    if !(0.0..=1.0).contains(&settings.editor_background_transparency) {
        return Err(CommandError::bad_request(
            "graph renderer text editor background transparency must be between 0 and 1",
        ));
    }
    if !(0.25..=3.0).contains(&settings.label_size_scale) {
        return Err(CommandError::bad_request(
            "graph renderer label size scale must be between 0.25 and 3",
        ));
    }
    if !(1.0..=24.0).contains(&settings.selected_node_outline_width_px) {
        return Err(CommandError::bad_request(
            "graph renderer selected node outline width must be between 1 and 24 pixels",
        ));
    }
    if !(0.0..=1.0).contains(&settings.selected_node_outline_brightness) {
        return Err(CommandError::bad_request(
            "graph renderer selected node outline brightness must be between 0 and 1",
        ));
    }
    validate_graph_renderer_hex_color(
        &settings.selected_node_outline_color,
        "graph renderer selected node outline color must be a hex color",
    )?;

    Ok(())
}

fn validate_graph_renderer_hex_color(
    color: &str,
    message: &'static str,
) -> Result<(), CommandError> {
    let Some(hex) = color.strip_prefix('#') else {
        return Err(CommandError::bad_request(message));
    };
    if hex.len() != 6 || !hex.chars().all(|character| character.is_ascii_hexdigit()) {
        return Err(CommandError::bad_request(message));
    }

    Ok(())
}

#[cfg(test)]
mod tests {
    use eidetic_bevy_bible_graph::{BibleGraphCameraCommand, BibleGraphNativeTextEditorSettings};
    use eidetic_core::contracts::BibleGraphNodeId;
    use serde_json::json;

    use super::{
        OpenGraphRendererRequest, RendererWindowSizeHint, validate_renderer_window_size_hint,
        validate_text_editor_settings,
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

    #[test]
    fn text_editor_settings_reject_out_of_range_values() {
        let error = validate_text_editor_settings(&BibleGraphNativeTextEditorSettings {
            padding_px: 80.0,
            ..BibleGraphNativeTextEditorSettings::default()
        })
        .unwrap_err();

        assert_eq!(
            serde_json::to_value(error).unwrap(),
            json!({
                "kind": "bad_request",
                "message": "graph renderer text editor padding must be between 0 and 48 pixels"
            })
        );
    }

    #[test]
    fn text_editor_settings_default_missing_label_size_scale() {
        let settings: BibleGraphNativeTextEditorSettings = serde_json::from_value(json!({
            "padding_px": 17.0,
            "corner_radius_px": 4.0,
            "editor_outline_width_px": 1.0,
            "editor_outline_brightness": 0.1,
            "editor_outline_transparency": 0.61,
            "font_size_px": 11.0,
            "font_brightness": 0.88,
            "editor_background_color": "#ffffff",
            "editor_background_brightness": 0.08,
            "editor_background_transparency": 0.0,
            "selected_node_outline_width_px": 3.0,
            "selected_node_outline_brightness": 1.0,
            "selected_node_outline_color": "#f6f5f4"
        }))
        .unwrap();

        assert_eq!(settings.label_size_scale, 1.0);
        validate_text_editor_settings(&settings).unwrap();
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
