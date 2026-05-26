use super::super::BibleGraphRendererWindowPlatform;
use crate::renderer_window::current_desktop_renderer_window_platform;

pub fn current_renderer_window_platform() -> BibleGraphRendererWindowPlatform {
    current_desktop_renderer_window_platform()
}
