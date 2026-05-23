use std::collections::BTreeMap;
use std::sync::Mutex;

use serde::{Deserialize, Serialize};

const MAX_VIEWPORT_DIMENSION: f64 = 32768.0;
const MAX_VIEWPORT_ID_LENGTH: usize = 128;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum EmbeddedViewportKind {
    Graph,
    Timeline,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct EmbeddedViewportBounds {
    pub x: f64,
    pub y: f64,
    pub width: f64,
    pub height: f64,
    pub scale_factor: f64,
}

#[derive(Debug, Clone, PartialEq, Serialize)]
pub struct EmbeddedViewportState {
    pub viewport_id: String,
    pub kind: EmbeddedViewportKind,
    pub bounds: EmbeddedViewportBounds,
    pub focused: bool,
    pub surface: EmbeddedViewportSurfaceState,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
pub struct EmbeddedViewportSurfaceState {
    pub attached: bool,
    pub status: EmbeddedViewportSurfaceStatus,
    pub strategy: EmbeddedViewportSurfaceStrategy,
    pub message: String,
    pub renderer_window: EmbeddedViewportRendererWindowState,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize)]
#[serde(rename_all = "snake_case")]
pub enum EmbeddedViewportSurfaceStatus {
    PendingAttachment,
    AttachmentUnsupported,
    #[allow(
        dead_code,
        reason = "native surface attachment lands in a later milestone 8 viewport slice"
    )]
    Attached,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize)]
#[serde(rename_all = "snake_case")]
pub enum EmbeddedViewportSurfaceStrategy {
    Unsupported,
    X11ChildWindow,
    WaylandExternalSurface,
    Win32ChildWindow,
    AppKitSubview,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
pub struct EmbeddedViewportRendererWindowState {
    pub status: EmbeddedViewportRendererWindowStatus,
    pub message: String,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize)]
#[serde(rename_all = "snake_case")]
pub enum EmbeddedViewportRendererWindowStatus {
    NotStarted,
    PendingCreation,
    CreationUnsupported,
    Attached,
}

#[derive(Debug, Clone, PartialEq, Serialize)]
pub struct EmbeddedViewportHostStatus {
    pub viewports: Vec<EmbeddedViewportState>,
}

#[derive(Debug, Clone, PartialEq, Deserialize)]
pub struct MountEmbeddedViewportRequest {
    pub viewport_id: String,
    pub kind: EmbeddedViewportKind,
    pub bounds: EmbeddedViewportBounds,
}

#[derive(Debug, Clone, PartialEq, Deserialize)]
pub struct UpdateEmbeddedViewportBoundsRequest {
    pub viewport_id: String,
    pub bounds: EmbeddedViewportBounds,
}

#[derive(Debug, Clone, PartialEq, Deserialize)]
pub struct SetEmbeddedViewportFocusRequest {
    pub viewport_id: String,
    pub focused: bool,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum EmbeddedViewportHostError {
    InvalidViewportId(String),
    InvalidBounds(String),
    ViewportAlreadyMounted(String),
    ViewportNotMounted(String),
    LockPoisoned,
}

impl std::fmt::Display for EmbeddedViewportHostError {
    fn fmt(&self, formatter: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            EmbeddedViewportHostError::InvalidViewportId(message) => {
                write!(formatter, "invalid viewport id: {message}")
            }
            EmbeddedViewportHostError::InvalidBounds(message) => {
                write!(formatter, "invalid viewport bounds: {message}")
            }
            EmbeddedViewportHostError::ViewportAlreadyMounted(viewport_id) => {
                write!(formatter, "viewport is already mounted: {viewport_id}")
            }
            EmbeddedViewportHostError::ViewportNotMounted(viewport_id) => {
                write!(formatter, "viewport is not mounted: {viewport_id}")
            }
            EmbeddedViewportHostError::LockPoisoned => {
                write!(formatter, "embedded viewport host lock is poisoned")
            }
        }
    }
}

impl std::error::Error for EmbeddedViewportHostError {}

#[derive(Default)]
pub struct EmbeddedViewportHost {
    viewports: Mutex<BTreeMap<String, EmbeddedViewportState>>,
}

impl EmbeddedViewportHost {
    pub fn mount(
        &self,
        request: MountEmbeddedViewportRequest,
    ) -> Result<EmbeddedViewportState, EmbeddedViewportHostError> {
        validate_viewport_id(&request.viewport_id)?;
        validate_bounds(&request.bounds)?;

        let state = EmbeddedViewportState {
            viewport_id: request.viewport_id.clone(),
            kind: request.kind,
            bounds: request.bounds,
            focused: false,
            surface: EmbeddedViewportSurfaceState::pending_attachment(),
        };

        let mut viewports = self
            .viewports
            .lock()
            .map_err(|_| EmbeddedViewportHostError::LockPoisoned)?;
        if viewports.contains_key(&request.viewport_id) {
            return Err(EmbeddedViewportHostError::ViewportAlreadyMounted(
                request.viewport_id,
            ));
        }
        viewports.insert(request.viewport_id, state.clone());
        Ok(state)
    }

    pub fn update_bounds(
        &self,
        request: UpdateEmbeddedViewportBoundsRequest,
    ) -> Result<EmbeddedViewportState, EmbeddedViewportHostError> {
        validate_bounds(&request.bounds)?;
        let mut viewports = self
            .viewports
            .lock()
            .map_err(|_| EmbeddedViewportHostError::LockPoisoned)?;
        let state = viewports
            .get_mut(&request.viewport_id)
            .ok_or_else(|| EmbeddedViewportHostError::ViewportNotMounted(request.viewport_id))?;
        state.bounds = request.bounds;
        Ok(state.clone())
    }

    pub fn set_focus(
        &self,
        request: SetEmbeddedViewportFocusRequest,
    ) -> Result<EmbeddedViewportState, EmbeddedViewportHostError> {
        let mut viewports = self
            .viewports
            .lock()
            .map_err(|_| EmbeddedViewportHostError::LockPoisoned)?;
        if request.focused {
            for state in viewports.values_mut() {
                state.focused = false;
            }
        }

        let state = viewports
            .get_mut(&request.viewport_id)
            .ok_or_else(|| EmbeddedViewportHostError::ViewportNotMounted(request.viewport_id))?;
        state.focused = request.focused;
        Ok(state.clone())
    }

    pub fn set_surface_state(
        &self,
        viewport_id: String,
        surface: EmbeddedViewportSurfaceState,
    ) -> Result<EmbeddedViewportState, EmbeddedViewportHostError> {
        let mut viewports = self
            .viewports
            .lock()
            .map_err(|_| EmbeddedViewportHostError::LockPoisoned)?;
        let state = viewports
            .get_mut(&viewport_id)
            .ok_or_else(|| EmbeddedViewportHostError::ViewportNotMounted(viewport_id))?;
        state.surface = surface;
        Ok(state.clone())
    }

    #[allow(
        dead_code,
        reason = "native surface attachment lands in a later milestone 8 viewport slice"
    )]
    pub fn attach_surface(
        &self,
        viewport_id: String,
    ) -> Result<EmbeddedViewportState, EmbeddedViewportHostError> {
        let mut viewports = self
            .viewports
            .lock()
            .map_err(|_| EmbeddedViewportHostError::LockPoisoned)?;
        let state = viewports
            .get_mut(&viewport_id)
            .ok_or_else(|| EmbeddedViewportHostError::ViewportNotMounted(viewport_id))?;
        state.surface = EmbeddedViewportSurfaceState::attached();
        Ok(state.clone())
    }

    pub fn unmount(
        &self,
        viewport_id: String,
    ) -> Result<EmbeddedViewportHostStatus, EmbeddedViewportHostError> {
        self.viewports
            .lock()
            .map_err(|_| EmbeddedViewportHostError::LockPoisoned)?
            .remove(&viewport_id)
            .ok_or_else(|| EmbeddedViewportHostError::ViewportNotMounted(viewport_id))?;
        self.status()
    }

    pub fn status(&self) -> Result<EmbeddedViewportHostStatus, EmbeddedViewportHostError> {
        let viewports = self
            .viewports
            .lock()
            .map_err(|_| EmbeddedViewportHostError::LockPoisoned)?
            .values()
            .cloned()
            .collect();
        Ok(EmbeddedViewportHostStatus { viewports })
    }
}

impl EmbeddedViewportSurfaceState {
    pub fn from_capability(
        status: EmbeddedViewportSurfaceStatus,
        strategy: EmbeddedViewportSurfaceStrategy,
        message: impl Into<String>,
    ) -> Self {
        let message = message.into();
        Self {
            attached: status == EmbeddedViewportSurfaceStatus::Attached,
            status,
            strategy,
            message,
            renderer_window: EmbeddedViewportRendererWindowState::from_surface_status(status),
        }
    }

    fn pending_attachment() -> Self {
        Self {
            attached: false,
            status: EmbeddedViewportSurfaceStatus::PendingAttachment,
            strategy: EmbeddedViewportSurfaceStrategy::Unsupported,
            message: "native Bevy surface attachment is not implemented yet".to_string(),
            renderer_window: EmbeddedViewportRendererWindowState::not_started(),
        }
    }

    #[allow(
        dead_code,
        reason = "native surface attachment lands in a later milestone 8 viewport slice"
    )]
    fn attached() -> Self {
        Self {
            attached: true,
            status: EmbeddedViewportSurfaceStatus::Attached,
            strategy: EmbeddedViewportSurfaceStrategy::Unsupported,
            message: "native Bevy surface is attached".to_string(),
            renderer_window: EmbeddedViewportRendererWindowState::attached(),
        }
    }
}

impl EmbeddedViewportRendererWindowState {
    fn from_surface_status(status: EmbeddedViewportSurfaceStatus) -> Self {
        match status {
            EmbeddedViewportSurfaceStatus::PendingAttachment => Self {
                status: EmbeddedViewportRendererWindowStatus::PendingCreation,
                message:
                    "renderer child window lifecycle is waiting for platform-specific creation"
                        .to_string(),
            },
            EmbeddedViewportSurfaceStatus::AttachmentUnsupported => Self {
                status: EmbeddedViewportRendererWindowStatus::CreationUnsupported,
                message: "renderer child window cannot be created for the detected parent surface"
                    .to_string(),
            },
            EmbeddedViewportSurfaceStatus::Attached => Self::attached(),
        }
    }

    fn not_started() -> Self {
        Self {
            status: EmbeddedViewportRendererWindowStatus::NotStarted,
            message: "renderer child window lifecycle has not started".to_string(),
        }
    }

    fn attached() -> Self {
        Self {
            status: EmbeddedViewportRendererWindowStatus::Attached,
            message: "renderer child window is attached to the viewport panel".to_string(),
        }
    }
}

fn validate_viewport_id(viewport_id: &str) -> Result<(), EmbeddedViewportHostError> {
    let trimmed = viewport_id.trim();
    if trimmed.is_empty() {
        return Err(EmbeddedViewportHostError::InvalidViewportId(
            "id must not be blank".to_string(),
        ));
    }
    if trimmed != viewport_id {
        return Err(EmbeddedViewportHostError::InvalidViewportId(
            "id must not contain leading or trailing whitespace".to_string(),
        ));
    }
    if viewport_id.len() > MAX_VIEWPORT_ID_LENGTH {
        return Err(EmbeddedViewportHostError::InvalidViewportId(format!(
            "id must be at most {MAX_VIEWPORT_ID_LENGTH} bytes"
        )));
    }
    if !viewport_id
        .chars()
        .all(|character| character.is_ascii_alphanumeric() || matches!(character, '-' | '_' | '.'))
    {
        return Err(EmbeddedViewportHostError::InvalidViewportId(
            "id may contain only ascii letters, numbers, hyphen, underscore, and dot".to_string(),
        ));
    }
    Ok(())
}

fn validate_bounds(bounds: &EmbeddedViewportBounds) -> Result<(), EmbeddedViewportHostError> {
    let values = [
        ("x", bounds.x),
        ("y", bounds.y),
        ("width", bounds.width),
        ("height", bounds.height),
        ("scale_factor", bounds.scale_factor),
    ];
    if let Some((field, _)) = values.iter().find(|(_, value)| !value.is_finite()) {
        return Err(EmbeddedViewportHostError::InvalidBounds(format!(
            "{field} must be finite"
        )));
    }
    if bounds.x < 0.0 || bounds.y < 0.0 {
        return Err(EmbeddedViewportHostError::InvalidBounds(
            "x and y must be non-negative".to_string(),
        ));
    }
    if bounds.width <= 0.0 || bounds.height <= 0.0 {
        return Err(EmbeddedViewportHostError::InvalidBounds(
            "width and height must be positive".to_string(),
        ));
    }
    if bounds.width > MAX_VIEWPORT_DIMENSION || bounds.height > MAX_VIEWPORT_DIMENSION {
        return Err(EmbeddedViewportHostError::InvalidBounds(format!(
            "width and height must be at most {MAX_VIEWPORT_DIMENSION}"
        )));
    }
    if bounds.scale_factor <= 0.0 {
        return Err(EmbeddedViewportHostError::InvalidBounds(
            "scale_factor must be positive".to_string(),
        ));
    }
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn mount_records_valid_borderless_viewport_panel() {
        let host = EmbeddedViewportHost::default();

        let state = host
            .mount(MountEmbeddedViewportRequest {
                viewport_id: "graph-main".to_string(),
                kind: EmbeddedViewportKind::Graph,
                bounds: sample_bounds(),
            })
            .unwrap();

        assert_eq!(state.viewport_id, "graph-main");
        assert_eq!(state.kind, EmbeddedViewportKind::Graph);
        assert!(!state.focused);
        assert_eq!(
            state.surface,
            EmbeddedViewportSurfaceState {
                attached: false,
                status: EmbeddedViewportSurfaceStatus::PendingAttachment,
                strategy: EmbeddedViewportSurfaceStrategy::Unsupported,
                message: "native Bevy surface attachment is not implemented yet".to_string(),
                renderer_window: EmbeddedViewportRendererWindowState {
                    status: EmbeddedViewportRendererWindowStatus::NotStarted,
                    message: "renderer child window lifecycle has not started".to_string(),
                },
            }
        );
        assert_eq!(host.status().unwrap().viewports, vec![state]);
    }

    #[test]
    fn mount_rejects_duplicate_viewport_ids_without_replacing_existing_state() {
        let host = EmbeddedViewportHost::default();
        let original = host
            .mount(MountEmbeddedViewportRequest {
                viewport_id: "graph-main".to_string(),
                kind: EmbeddedViewportKind::Graph,
                bounds: sample_bounds(),
            })
            .unwrap();

        assert_eq!(
            host.mount(MountEmbeddedViewportRequest {
                viewport_id: "graph-main".to_string(),
                kind: EmbeddedViewportKind::Timeline,
                bounds: EmbeddedViewportBounds {
                    x: 20.0,
                    y: 30.0,
                    width: 400.0,
                    height: 220.0,
                    scale_factor: 1.0,
                },
            })
            .unwrap_err(),
            EmbeddedViewportHostError::ViewportAlreadyMounted("graph-main".to_string())
        );

        assert_eq!(host.status().unwrap().viewports, vec![original]);
    }

    #[test]
    fn update_bounds_requires_existing_viewport_and_valid_dimensions() {
        let host = EmbeddedViewportHost::default();

        assert_eq!(
            host.update_bounds(UpdateEmbeddedViewportBoundsRequest {
                viewport_id: "graph-main".to_string(),
                bounds: sample_bounds(),
            })
            .unwrap_err(),
            EmbeddedViewportHostError::ViewportNotMounted("graph-main".to_string())
        );

        host.mount(MountEmbeddedViewportRequest {
            viewport_id: "graph-main".to_string(),
            kind: EmbeddedViewportKind::Graph,
            bounds: sample_bounds(),
        })
        .unwrap();
        let updated = host
            .update_bounds(UpdateEmbeddedViewportBoundsRequest {
                viewport_id: "graph-main".to_string(),
                bounds: EmbeddedViewportBounds {
                    x: 8.0,
                    y: 16.0,
                    width: 640.0,
                    height: 360.0,
                    scale_factor: 2.0,
                },
            })
            .unwrap();

        assert_eq!(updated.bounds.width, 640.0);
        assert_eq!(updated.bounds.scale_factor, 2.0);
    }

    #[test]
    fn focus_is_exclusive_across_viewports() {
        let host = EmbeddedViewportHost::default();
        host.mount(MountEmbeddedViewportRequest {
            viewport_id: "graph-main".to_string(),
            kind: EmbeddedViewportKind::Graph,
            bounds: sample_bounds(),
        })
        .unwrap();
        host.mount(MountEmbeddedViewportRequest {
            viewport_id: "timeline-bottom".to_string(),
            kind: EmbeddedViewportKind::Timeline,
            bounds: sample_bounds(),
        })
        .unwrap();

        host.set_focus(SetEmbeddedViewportFocusRequest {
            viewport_id: "graph-main".to_string(),
            focused: true,
        })
        .unwrap();
        host.set_focus(SetEmbeddedViewportFocusRequest {
            viewport_id: "timeline-bottom".to_string(),
            focused: true,
        })
        .unwrap();

        let status = host.status().unwrap();
        assert!(!status.viewports[0].focused);
        assert!(status.viewports[1].focused);
    }

    #[test]
    fn unmount_removes_viewport() {
        let host = EmbeddedViewportHost::default();
        host.mount(MountEmbeddedViewportRequest {
            viewport_id: "graph-main".to_string(),
            kind: EmbeddedViewportKind::Graph,
            bounds: sample_bounds(),
        })
        .unwrap();

        let status = host.unmount("graph-main".to_string()).unwrap();

        assert!(status.viewports.is_empty());
    }

    #[test]
    fn attach_surface_marks_viewport_as_visibly_attached() {
        let host = EmbeddedViewportHost::default();
        host.mount(MountEmbeddedViewportRequest {
            viewport_id: "graph-main".to_string(),
            kind: EmbeddedViewportKind::Graph,
            bounds: sample_bounds(),
        })
        .unwrap();

        let state = host.attach_surface("graph-main".to_string()).unwrap();

        assert_eq!(
            state.surface,
            EmbeddedViewportSurfaceState {
                attached: true,
                status: EmbeddedViewportSurfaceStatus::Attached,
                strategy: EmbeddedViewportSurfaceStrategy::Unsupported,
                message: "native Bevy surface is attached".to_string(),
                renderer_window: EmbeddedViewportRendererWindowState {
                    status: EmbeddedViewportRendererWindowStatus::Attached,
                    message: "renderer child window is attached to the viewport panel".to_string(),
                },
            }
        );
    }

    #[test]
    fn surface_capability_updates_existing_viewport() {
        let host = EmbeddedViewportHost::default();
        host.mount(MountEmbeddedViewportRequest {
            viewport_id: "graph-main".to_string(),
            kind: EmbeddedViewportKind::Graph,
            bounds: sample_bounds(),
        })
        .unwrap();

        let state = host
            .set_surface_state(
                "graph-main".to_string(),
                EmbeddedViewportSurfaceState::from_capability(
                    EmbeddedViewportSurfaceStatus::AttachmentUnsupported,
                    EmbeddedViewportSurfaceStrategy::WaylandExternalSurface,
                    "Wayland parent surface is available",
                ),
            )
            .unwrap();

        assert_eq!(
            state.surface,
            EmbeddedViewportSurfaceState {
                attached: false,
                status: EmbeddedViewportSurfaceStatus::AttachmentUnsupported,
                strategy: EmbeddedViewportSurfaceStrategy::WaylandExternalSurface,
                message: "Wayland parent surface is available".to_string(),
                renderer_window: EmbeddedViewportRendererWindowState {
                    status: EmbeddedViewportRendererWindowStatus::CreationUnsupported,
                    message:
                        "renderer child window cannot be created for the detected parent surface"
                            .to_string(),
                },
            }
        );
    }

    #[test]
    fn invalid_viewport_payloads_are_rejected() {
        let host = EmbeddedViewportHost::default();

        assert!(matches!(
            host.mount(MountEmbeddedViewportRequest {
                viewport_id: " graph ".to_string(),
                kind: EmbeddedViewportKind::Graph,
                bounds: sample_bounds(),
            }),
            Err(EmbeddedViewportHostError::InvalidViewportId(_))
        ));
        assert!(matches!(
            host.mount(MountEmbeddedViewportRequest {
                viewport_id: "graph-main".to_string(),
                kind: EmbeddedViewportKind::Graph,
                bounds: EmbeddedViewportBounds {
                    width: 0.0,
                    ..sample_bounds()
                },
            }),
            Err(EmbeddedViewportHostError::InvalidBounds(_))
        ));
    }

    fn sample_bounds() -> EmbeddedViewportBounds {
        EmbeddedViewportBounds {
            x: 0.0,
            y: 0.0,
            width: 1280.0,
            height: 720.0,
            scale_factor: 1.0,
        }
    }
}
