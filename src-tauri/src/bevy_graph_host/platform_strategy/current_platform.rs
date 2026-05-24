use super::super::BibleGraphRendererWindowPlatform;

pub fn current_renderer_window_platform() -> BibleGraphRendererWindowPlatform {
    if cfg!(target_os = "linux") {
        BibleGraphRendererWindowPlatform::Linux
    } else if cfg!(target_os = "macos") {
        BibleGraphRendererWindowPlatform::Macos
    } else if cfg!(target_os = "windows") {
        BibleGraphRendererWindowPlatform::Windows
    } else {
        BibleGraphRendererWindowPlatform::Unsupported
    }
}
