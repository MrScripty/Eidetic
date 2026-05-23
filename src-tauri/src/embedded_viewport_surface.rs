use raw_window_handle::{HasWindowHandle, RawWindowHandle};
use serde::Serialize;
use tauri::Manager;

use crate::embedded_viewport_host::{
    EmbeddedViewportSurfaceState, EmbeddedViewportSurfaceStatus, EmbeddedViewportSurfaceStrategy,
};

const MAIN_WINDOW_LABEL: &str = "main";

pub fn detect_main_window_surface(app: &tauri::AppHandle) -> EmbeddedViewportSurfaceState {
    let Some(window) = app.get_webview_window(MAIN_WINDOW_LABEL) else {
        return EmbeddedViewportSurfaceState::from_capability(
            EmbeddedViewportSurfaceStatus::AttachmentUnsupported,
            EmbeddedViewportSurfaceStrategy::Unsupported,
            "main Tauri webview window was not found",
        );
    };

    match window.window_handle() {
        Ok(handle) => surface_state_for_raw_handle(handle.as_raw()),
        Err(error) => EmbeddedViewportSurfaceState::from_capability(
            EmbeddedViewportSurfaceStatus::AttachmentUnsupported,
            EmbeddedViewportSurfaceStrategy::Unsupported,
            format!("failed to read Tauri window handle: {error}"),
        ),
    }
}

pub fn surface_state_for_raw_handle(handle: RawWindowHandle) -> EmbeddedViewportSurfaceState {
    let capability = surface_capability_for_raw_handle(handle);
    EmbeddedViewportSurfaceState::from_capability(
        capability.status,
        capability.strategy,
        capability.message,
    )
}

fn surface_capability_for_raw_handle(handle: RawWindowHandle) -> EmbeddedViewportSurfaceCapability {
    match handle {
        RawWindowHandle::Xlib(_) | RawWindowHandle::Xcb(_) => EmbeddedViewportSurfaceCapability {
            status: EmbeddedViewportSurfaceStatus::PendingAttachment,
            strategy: EmbeddedViewportSurfaceStrategy::X11ChildWindow,
            message: "X11 parent window handle is available; Bevy child surface attachment is the next implementation step".to_string(),
        },
        RawWindowHandle::Wayland(_) => EmbeddedViewportSurfaceCapability {
            status: EmbeddedViewportSurfaceStatus::AttachmentUnsupported,
            strategy: EmbeddedViewportSurfaceStrategy::WaylandExternalSurface,
            message: "Wayland parent surface is available, but the current Milestone 8 attachment strategy requires a supported child-surface path".to_string(),
        },
        RawWindowHandle::Win32(_) => EmbeddedViewportSurfaceCapability {
            status: EmbeddedViewportSurfaceStatus::PendingAttachment,
            strategy: EmbeddedViewportSurfaceStrategy::Win32ChildWindow,
            message: "Win32 parent window handle is available; Bevy child surface attachment is the next implementation step".to_string(),
        },
        RawWindowHandle::AppKit(_) => EmbeddedViewportSurfaceCapability {
            status: EmbeddedViewportSurfaceStatus::PendingAttachment,
            strategy: EmbeddedViewportSurfaceStrategy::AppKitSubview,
            message: "AppKit parent view handle is available; Bevy child surface attachment is the next implementation step".to_string(),
        },
        _ => EmbeddedViewportSurfaceCapability {
            status: EmbeddedViewportSurfaceStatus::AttachmentUnsupported,
            strategy: EmbeddedViewportSurfaceStrategy::Unsupported,
            message: "current platform window handle is not supported for embedded Bevy viewport attachment".to_string(),
        },
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
struct EmbeddedViewportSurfaceCapability {
    status: EmbeddedViewportSurfaceStatus,
    strategy: EmbeddedViewportSurfaceStrategy,
    message: String,
}

#[cfg(test)]
mod tests {
    use raw_window_handle::{RawWindowHandle, WaylandWindowHandle, XlibWindowHandle};

    use super::*;

    #[test]
    fn x11_parent_window_is_pending_attachment() {
        let state = surface_state_for_raw_handle(RawWindowHandle::Xlib(XlibWindowHandle::new(42)));

        assert!(!state.attached);
        assert_eq!(
            state.status,
            EmbeddedViewportSurfaceStatus::PendingAttachment
        );
        assert_eq!(
            state.strategy,
            EmbeddedViewportSurfaceStrategy::X11ChildWindow
        );
        assert!(
            state
                .message
                .contains("X11 parent window handle is available")
        );
    }

    #[test]
    fn wayland_parent_surface_is_explicitly_unsupported_for_current_strategy() {
        let state = surface_state_for_raw_handle(RawWindowHandle::Wayland(
            WaylandWindowHandle::new(std::ptr::NonNull::dangling()),
        ));

        assert!(!state.attached);
        assert_eq!(
            state.status,
            EmbeddedViewportSurfaceStatus::AttachmentUnsupported
        );
        assert_eq!(
            state.strategy,
            EmbeddedViewportSurfaceStrategy::WaylandExternalSurface
        );
        assert!(
            state
                .message
                .contains("Wayland parent surface is available")
        );
    }
}
