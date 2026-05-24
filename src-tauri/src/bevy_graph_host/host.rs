use std::panic::{AssertUnwindSafe, catch_unwind};

use eidetic_bevy_bible_graph::{
    BibleGraphRendererApp, BibleGraphRendererError, BibleGraphVisualSnapshot,
};
use eidetic_core::contracts::{
    BibleGraphEdgeId, BibleGraphNodeId, BibleRenderGraphProjection, ContextInfluenceId,
};

use crate::renderer_window::DesktopRendererWindowKind;

use super::{
    BibleGraphHostError, BibleGraphHostStatus, BibleGraphRendererWindowLifecycle,
    NativeRendererRunner, NativeRendererRunnerHandle, NativeRendererRunnerStatus,
    NativeRendererSupervisor,
};

pub struct DesktopBibleGraphHost {
    renderer: Option<BibleGraphRendererApp>,
    native_runner: DesktopNativeRendererRunner,
    last_error: Option<String>,
}

enum DesktopNativeRendererRunner {
    Managed(NativeRendererRunnerHandle),
    Unavailable(NativeRendererRunnerStatus),
}

impl DesktopNativeRendererRunner {
    fn start_current_platform() -> Self {
        match NativeRendererRunnerHandle::start_for_current_platform() {
            Ok(runner) => Self::Managed(runner),
            Err(error) => {
                Self::Unavailable(NativeRendererSupervisor::failed_current_platform_status(
                    format!("failed to start native renderer runner boundary: {error}"),
                ))
            }
        }
    }

    fn open(&mut self) -> NativeRendererRunnerStatus {
        match self {
            Self::Managed(runner) => runner.open(),
            Self::Unavailable(status) => status.clone(),
        }
    }

    fn close(&mut self) -> NativeRendererRunnerStatus {
        match self {
            Self::Managed(runner) => runner.close(),
            Self::Unavailable(status) => status.clone(),
        }
    }

    fn focus(&mut self) -> NativeRendererRunnerStatus {
        match self {
            Self::Managed(runner) => runner.focus(),
            Self::Unavailable(status) => status.clone(),
        }
    }

    fn status(&self) -> NativeRendererRunnerStatus {
        match self {
            Self::Managed(runner) => runner.status(),
            Self::Unavailable(status) => status.clone(),
        }
    }
}

impl Default for DesktopBibleGraphHost {
    fn default() -> Self {
        Self::new()
    }
}

impl DesktopBibleGraphHost {
    pub fn new() -> Self {
        Self {
            renderer: None,
            native_runner: DesktopNativeRendererRunner::start_current_platform(),
            last_error: None,
        }
    }

    pub fn renderer_unavailable_status(message: String) -> BibleGraphHostStatus {
        Self::status_from_parts(
            None,
            NativeRendererSupervisor::failed_current_platform_status(message),
            None,
        )
    }

    pub fn start(&mut self) -> Result<BibleGraphHostStatus, BibleGraphHostError> {
        if self.renderer.is_none() {
            self.renderer = Some(Self::catch_renderer_panic(
                BibleGraphRendererApp::new_renderer_window,
            )?);
        }
        self.native_runner.open();
        self.last_error = None;
        Ok(self.status())
    }

    pub fn stop(&mut self) -> BibleGraphHostStatus {
        self.renderer = None;
        self.native_runner.close();
        self.last_error = None;
        self.status()
    }

    pub fn focus(&mut self) -> BibleGraphHostStatus {
        self.native_runner.focus();
        self.status()
    }

    pub fn set_projection(
        &mut self,
        projection: BibleRenderGraphProjection,
    ) -> Result<BibleGraphHostStatus, BibleGraphHostError> {
        self.start()?;
        self.with_renderer_mut(|renderer| renderer.set_projection(projection))?;
        Ok(self.status())
    }

    pub fn update_projection_if_open(
        &mut self,
        projection: BibleRenderGraphProjection,
    ) -> Result<BibleGraphHostStatus, BibleGraphHostError> {
        let Some(renderer) = self.renderer.as_mut() else {
            return Ok(self.status());
        };

        let result =
            Self::catch_renderer_panic(|| renderer.set_projection(projection))?.map_err(|error| {
                let message = error.to_string();
                self.last_error = Some(message.clone());
                BibleGraphHostError::Renderer(message)
            });
        if result.is_ok() {
            self.last_error = None;
        }
        result?;
        Ok(self.status())
    }

    pub fn set_renderer_window_bounds(
        &mut self,
        width_px: u32,
        height_px: u32,
    ) -> Result<BibleGraphHostStatus, BibleGraphHostError> {
        validate_renderer_window_bounds(width_px, height_px)?;
        self.start()?;
        self.with_renderer_mut(|renderer| {
            renderer.set_renderer_window_bounds(width_px, height_px);
            Ok(())
        })?;
        Ok(self.status())
    }

    pub fn select_node(&mut self, node_id: BibleGraphNodeId) -> Result<(), BibleGraphHostError> {
        self.with_renderer_mut(|renderer| renderer.select_node(node_id))
    }

    pub fn inspect_node(&mut self, node_id: BibleGraphNodeId) -> Result<(), BibleGraphHostError> {
        self.with_renderer_mut(|renderer| renderer.inspect_node(node_id))
    }

    pub fn select_edge(&mut self, edge_id: BibleGraphEdgeId) -> Result<(), BibleGraphHostError> {
        self.with_renderer_mut(|renderer| renderer.select_edge(edge_id))
    }

    pub fn select_influence(
        &mut self,
        influence_id: ContextInfluenceId,
    ) -> Result<(), BibleGraphHostError> {
        self.with_renderer_mut(|renderer| renderer.select_influence(influence_id))
    }

    pub fn drain_commands(&mut self) -> Vec<eidetic_bevy_bible_graph::BibleGraphRendererCommand> {
        self.renderer
            .as_mut()
            .map(BibleGraphRendererApp::drain_commands)
            .unwrap_or_default()
    }

    pub fn visual_snapshot(&mut self) -> Result<BibleGraphVisualSnapshot, BibleGraphHostError> {
        let Some(renderer) = self.renderer.as_ref() else {
            return Err(BibleGraphHostError::Renderer(
                BibleGraphRendererError::MissingProjection.to_string(),
            ));
        };
        let result = Self::catch_renderer_panic(|| renderer.visual_snapshot())?.map_err(|error| {
            let message = error.to_string();
            self.last_error = Some(message.clone());
            BibleGraphHostError::Renderer(message)
        });
        if result.is_ok() {
            self.last_error = None;
        }
        result
    }

    pub fn status(&self) -> BibleGraphHostStatus {
        let native_runner = self.native_runner.status();
        Self::status_from_parts(
            self.renderer.as_ref(),
            native_runner,
            self.last_error.clone(),
        )
    }

    fn status_from_parts(
        renderer: Option<&BibleGraphRendererApp>,
        native_runner: NativeRendererRunnerStatus,
        host_last_error: Option<String>,
    ) -> BibleGraphHostStatus {
        let (node_count, edge_count, influence_count) = renderer
            .map(|renderer| {
                let (node_count, edge_count) = renderer.scene_counts();
                (node_count, edge_count, renderer.influence_count())
            })
            .unwrap_or_default();
        let renderer_scene_ready = renderer
            .map(BibleGraphRendererApp::renderer_window_ready)
            .unwrap_or_default();
        let (native_visual_node_count, native_visual_edge_count) = renderer
            .map(BibleGraphRendererApp::native_visual_counts)
            .unwrap_or_default();
        let renderer_window_bounds = renderer
            .map(BibleGraphRendererApp::renderer_window_bounds)
            .unwrap_or_default();
        let last_error = host_last_error.or(native_runner.last_error.clone());
        let renderer_window_lifecycle = BibleGraphRendererWindowLifecycle::from_state(
            renderer.is_some(),
            renderer_scene_ready,
            native_runner.window_visible,
        );

        BibleGraphHostStatus {
            renderer_window_kind: DesktopRendererWindowKind::BibleGraph,
            running: renderer.is_some(),
            renderer_window_open: renderer.is_some(),
            renderer_scene_ready,
            renderer_window_visible: native_runner.window_visible,
            renderer_window_strategy: native_runner.strategy,
            renderer_window_platform: native_runner.platform,
            renderer_runner_lifecycle: native_runner.lifecycle,
            renderer_supervisor_lifecycle: native_runner.supervisor_lifecycle,
            renderer_runner_threading_model: native_runner.threading_model,
            renderer_window_capability: native_runner.capability,
            renderer_window_capability_reason: native_runner.capability_reason,
            renderer_window_lifecycle,
            renderer_window_ready: native_runner.window_ready,
            renderer_window_verified_support: native_runner.verified_support,
            renderer_window_visible_supported: native_runner.visible_window_supported,
            renderer_window_focus_supported: native_runner.focus_supported,
            renderer_window_message: renderer_window_message(
                renderer.is_some(),
                renderer_scene_ready,
            ),
            node_count,
            edge_count,
            native_visual_node_count,
            native_visual_edge_count,
            renderer_window_width_px: renderer_window_bounds.width_px,
            renderer_window_height_px: renderer_window_bounds.height_px,
            influence_count,
            last_error,
        }
    }

    fn with_renderer_mut<T>(
        &mut self,
        operation: impl FnOnce(&mut BibleGraphRendererApp) -> Result<T, BibleGraphRendererError>,
    ) -> Result<T, BibleGraphHostError> {
        self.start()?;
        let Some(renderer) = self.renderer.as_mut() else {
            return Err(BibleGraphHostError::Renderer(
                BibleGraphRendererError::MissingProjection.to_string(),
            ));
        };
        let result = Self::catch_renderer_panic(|| operation(renderer))?.map_err(|error| {
            let message = error.to_string();
            self.last_error = Some(message.clone());
            BibleGraphHostError::Renderer(message)
        });
        if result.is_ok() {
            self.last_error = None;
        }
        result
    }

    fn catch_renderer_panic<T>(operation: impl FnOnce() -> T) -> Result<T, BibleGraphHostError> {
        catch_unwind(AssertUnwindSafe(operation)).map_err(|_| BibleGraphHostError::RendererPanic)
    }
}

fn renderer_window_message(running: bool, scene_ready: bool) -> String {
    match (running, scene_ready) {
        (true, true) => {
            "graph renderer scene is ready; visible native window is pending implementation"
                .to_string()
        }
        (true, false) => "graph renderer lifecycle is active; scene is starting".to_string(),
        (false, _) => "floating graph renderer window is closed".to_string(),
    }
}

fn validate_renderer_window_bounds(
    width_px: u32,
    height_px: u32,
) -> Result<(), BibleGraphHostError> {
    if width_px == 0 || height_px == 0 {
        return Err(BibleGraphHostError::InvalidRendererWindowBounds {
            width_px,
            height_px,
        });
    }

    Ok(())
}
