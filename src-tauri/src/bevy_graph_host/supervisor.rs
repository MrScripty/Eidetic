use super::{
    BibleGraphRendererWindowCapability, BibleGraphRendererWindowCapabilityReason,
    NativeRendererPlatformStrategy, NativeRendererRunner, NativeRendererRunnerLifecycle,
    NativeRendererRunnerStartupPlan, NativeRendererRunnerStatus, NativeRendererWindowThreadHandle,
    NativeRendererWindowThreadResult,
};
use eidetic_bevy_bible_graph::{
    BibleGraphCameraCommand, BibleGraphNativeTextEditorSettings, BibleGraphNativeVisualStatus,
    BibleGraphNativeWindowRunnerConfig, BibleGraphRendererCommand,
    BibleGraphWorkspaceTimelineVisualSnapshot,
};
use eidetic_core::contracts::BibleRenderGraphProjection;
use std::time::Duration;

const NATIVE_RENDERER_WINDOW_STOP_TIMEOUT: Duration = Duration::from_millis(2_000);

type NativeRendererWindowThreadStart =
    fn(BibleGraphNativeWindowRunnerConfig) -> std::io::Result<NativeRendererWindowThreadHandle>;

pub use crate::renderer_window::DesktopRendererSupervisorLifecycle as NativeRendererSupervisorLifecycle;

#[derive(Debug)]
pub struct NativeRendererSupervisor {
    strategy: NativeRendererPlatformStrategy,
    startup_plan: NativeRendererRunnerStartupPlan,
    window_thread_start: NativeRendererWindowThreadStart,
    window_thread: Option<NativeRendererWindowThreadHandle>,
    window_ready: bool,
    native_visual_counts: BibleGraphNativeVisualStatus,
    latest_projection: Option<BibleRenderGraphProjection>,
    latest_workspace_timeline_visual_snapshot: Option<BibleGraphWorkspaceTimelineVisualSnapshot>,
    text_editor_settings: BibleGraphNativeTextEditorSettings,
    lifecycle: NativeRendererSupervisorLifecycle,
    last_error: Option<String>,
}

impl NativeRendererSupervisor {
    pub fn for_strategy(strategy: NativeRendererPlatformStrategy) -> Self {
        Self::for_strategy_with_window_thread_start(
            strategy,
            NativeRendererWindowThreadHandle::start,
        )
    }

    pub(crate) fn for_strategy_with_window_thread_start(
        strategy: NativeRendererPlatformStrategy,
        window_thread_start: NativeRendererWindowThreadStart,
    ) -> Self {
        Self {
            strategy,
            startup_plan: strategy.runner_startup_plan(),
            window_thread_start,
            window_thread: None,
            window_ready: false,
            native_visual_counts: BibleGraphNativeVisualStatus::default(),
            latest_projection: None,
            latest_workspace_timeline_visual_snapshot: None,
            text_editor_settings: BibleGraphNativeTextEditorSettings::default(),
            lifecycle: NativeRendererSupervisorLifecycle::NotStarted,
            last_error: None,
        }
    }

    pub fn failed_current_platform_status(message: String) -> NativeRendererRunnerStatus {
        Self {
            strategy: NativeRendererPlatformStrategy::current(),
            startup_plan: NativeRendererPlatformStrategy::current().runner_startup_plan(),
            window_thread_start: NativeRendererWindowThreadHandle::start,
            window_thread: None,
            window_ready: false,
            native_visual_counts: BibleGraphNativeVisualStatus::default(),
            latest_projection: None,
            latest_workspace_timeline_visual_snapshot: None,
            text_editor_settings: BibleGraphNativeTextEditorSettings::default(),
            lifecycle: NativeRendererSupervisorLifecycle::Failed,
            last_error: Some(message),
        }
        .status()
    }

    pub fn startup_plan(&self) -> &NativeRendererRunnerStartupPlan {
        &self.startup_plan
    }

    pub fn lifecycle(&self) -> NativeRendererSupervisorLifecycle {
        self.lifecycle
    }

    #[cfg(test)]
    pub(crate) fn latest_projection_node_count(&self) -> Option<usize> {
        self.latest_projection
            .as_ref()
            .map(|projection| projection.nodes.len())
    }

    #[cfg(test)]
    pub(crate) fn latest_workspace_timeline_visual_clip_count(&self) -> Option<usize> {
        self.latest_workspace_timeline_visual_snapshot
            .as_ref()
            .map(|snapshot| snapshot.clips.len())
    }

    fn runner_lifecycle(&self) -> NativeRendererRunnerLifecycle {
        match self.lifecycle {
            NativeRendererSupervisorLifecycle::NotStarted
            | NativeRendererSupervisorLifecycle::Closed
            | NativeRendererSupervisorLifecycle::Failed => NativeRendererRunnerLifecycle::Closed,
            NativeRendererSupervisorLifecycle::Starting
            | NativeRendererSupervisorLifecycle::Closing => {
                NativeRendererRunnerLifecycle::OpenRequested
            }
            NativeRendererSupervisorLifecycle::Running => NativeRendererRunnerLifecycle::Visible,
        }
    }

    pub fn refresh_status(&mut self) -> NativeRendererRunnerStatus {
        self.refresh_window_thread();
        self.status()
    }

    pub fn shutdown(&mut self) -> NativeRendererRunnerStatus {
        self.shutdown_window_thread();
        self.status()
    }

    fn refresh_window_thread(&mut self) {
        let Some(window_thread) = self.window_thread.as_mut() else {
            return;
        };
        let status = window_thread.join_completed();
        self.window_ready = status.ready;
        self.native_visual_counts = status.native_visual_counts;
        if status.running {
            return;
        }

        self.window_thread = None;
        self.window_ready = false;
        self.native_visual_counts = BibleGraphNativeVisualStatus::default();
        match status.result {
            Some(NativeRendererWindowThreadResult::Completed) if status.close_requested => {
                self.lifecycle = NativeRendererSupervisorLifecycle::Closed;
                self.last_error = None;
            }
            Some(NativeRendererWindowThreadResult::Completed) => {
                self.lifecycle = NativeRendererSupervisorLifecycle::Failed;
                self.last_error = Some("native renderer window closed unexpectedly".to_string());
            }
            Some(NativeRendererWindowThreadResult::Panicked) => {
                self.lifecycle = NativeRendererSupervisorLifecycle::Failed;
                self.last_error = Some("native renderer window thread panicked".to_string());
            }
            None => {}
        }
    }

    fn open_minimal_window(
        &mut self,
        config: BibleGraphNativeWindowRunnerConfig,
    ) -> NativeRendererRunnerStatus {
        self.refresh_window_thread();
        if let Some(window_thread) = self.window_thread.as_ref() {
            window_thread.request_show();
            self.lifecycle = NativeRendererSupervisorLifecycle::Running;
            self.last_error = None;
            return self.status();
        }

        self.lifecycle = NativeRendererSupervisorLifecycle::Starting;
        match (self.window_thread_start)(config) {
            Ok(window_thread) => {
                window_thread.set_text_editor_settings(self.text_editor_settings.clone());
                if let Some(projection) = self.latest_projection.clone() {
                    window_thread.set_projection(projection);
                }
                if let Some(snapshot) = self.latest_workspace_timeline_visual_snapshot.clone() {
                    window_thread.set_workspace_timeline_visual_snapshot(snapshot);
                }
                self.window_thread = Some(window_thread);
                self.window_ready = false;
                self.lifecycle = NativeRendererSupervisorLifecycle::Running;
                self.last_error = None;
            }
            Err(error) => {
                self.lifecycle = NativeRendererSupervisorLifecycle::Failed;
                self.last_error = Some(format!("failed to start native renderer window: {error}"));
            }
        }
        self.status()
    }

    fn close_window_thread(&mut self) {
        let Some(window_thread) = self.window_thread.as_ref() else {
            self.lifecycle = NativeRendererSupervisorLifecycle::Closed;
            self.window_ready = false;
            self.native_visual_counts = BibleGraphNativeVisualStatus::default();
            self.last_error = None;
            return;
        };

        window_thread.request_hide();
        self.lifecycle = NativeRendererSupervisorLifecycle::Closed;
        self.last_error = None;
    }

    fn shutdown_window_thread(&mut self) {
        let Some(mut window_thread) = self.window_thread.take() else {
            self.lifecycle = NativeRendererSupervisorLifecycle::Closed;
            self.window_ready = false;
            self.native_visual_counts = BibleGraphNativeVisualStatus::default();
            self.last_error = None;
            return;
        };

        self.lifecycle = NativeRendererSupervisorLifecycle::Closing;
        let status = window_thread.stop(NATIVE_RENDERER_WINDOW_STOP_TIMEOUT);
        if status.running {
            self.lifecycle = NativeRendererSupervisorLifecycle::Failed;
            self.window_ready = status.ready;
            self.native_visual_counts = status.native_visual_counts;
            self.last_error =
                Some("native renderer window did not stop before timeout".to_string());
            self.window_thread = Some(window_thread);
            return;
        }

        match status.result {
            Some(NativeRendererWindowThreadResult::Completed) => {
                self.lifecycle = NativeRendererSupervisorLifecycle::Closed;
                self.window_ready = false;
                self.native_visual_counts = BibleGraphNativeVisualStatus::default();
                self.last_error = None;
            }
            Some(NativeRendererWindowThreadResult::Panicked) => {
                self.lifecycle = NativeRendererSupervisorLifecycle::Failed;
                self.window_ready = false;
                self.native_visual_counts = BibleGraphNativeVisualStatus::default();
                self.last_error = Some("native renderer window thread panicked".to_string());
            }
            None => {
                self.lifecycle = NativeRendererSupervisorLifecycle::Failed;
                self.window_ready = false;
                self.native_visual_counts = BibleGraphNativeVisualStatus::default();
                self.last_error =
                    Some("native renderer window stop completed without a result".to_string());
            }
        }
    }
}

impl NativeRendererRunner for NativeRendererSupervisor {
    fn open(&mut self) -> NativeRendererRunnerStatus {
        match self.startup_plan.clone() {
            NativeRendererRunnerStartupPlan::MinimalWindowProofCandidate { config, .. } => {
                return self.open_minimal_window(config);
            }
            NativeRendererRunnerStartupPlan::PendingOnly { .. } => {
                self.lifecycle = NativeRendererSupervisorLifecycle::Closed;
                self.last_error = None;
            }
        }
        self.status()
    }

    fn close(&mut self) -> NativeRendererRunnerStatus {
        self.close_window_thread();
        self.status()
    }

    fn focus(&mut self) -> NativeRendererRunnerStatus {
        self.refresh_status()
    }

    fn set_projection(
        &mut self,
        projection: BibleRenderGraphProjection,
    ) -> NativeRendererRunnerStatus {
        self.latest_projection = Some(projection.clone());
        if let Some(window_thread) = self.window_thread.as_ref() {
            window_thread.set_projection(projection);
        }
        self.refresh_status()
    }

    fn set_workspace_timeline_visual_snapshot(
        &mut self,
        snapshot: BibleGraphWorkspaceTimelineVisualSnapshot,
    ) -> NativeRendererRunnerStatus {
        self.latest_workspace_timeline_visual_snapshot = Some(snapshot.clone());
        if let Some(window_thread) = self.window_thread.as_ref() {
            window_thread.set_workspace_timeline_visual_snapshot(snapshot);
        }
        self.refresh_status()
    }

    fn apply_camera_command(
        &mut self,
        command: BibleGraphCameraCommand,
    ) -> NativeRendererRunnerStatus {
        if let Some(window_thread) = self.window_thread.as_ref()
            && let Err(error) = window_thread.apply_camera_command(command)
        {
            self.lifecycle = NativeRendererSupervisorLifecycle::Failed;
            self.last_error = Some(format!("native renderer camera command failed: {error}"));
        }
        self.refresh_status()
    }

    fn apply_text_editor_settings(
        &mut self,
        settings: BibleGraphNativeTextEditorSettings,
    ) -> NativeRendererRunnerStatus {
        self.text_editor_settings = settings.clone();
        if let Some(window_thread) = self.window_thread.as_ref() {
            window_thread.set_text_editor_settings(settings);
        }
        self.refresh_status()
    }

    fn drain_commands(&mut self) -> Vec<BibleGraphRendererCommand> {
        self.refresh_window_thread();
        self.window_thread
            .as_ref()
            .map(NativeRendererWindowThreadHandle::drain_commands)
            .unwrap_or_default()
    }

    fn status(&self) -> NativeRendererRunnerStatus {
        let strategy = self.strategy.status();
        let (capability, capability_reason) =
            if self.lifecycle == NativeRendererSupervisorLifecycle::Failed {
                (
                    BibleGraphRendererWindowCapability::RunnerError,
                    BibleGraphRendererWindowCapabilityReason::RunnerError,
                )
            } else {
                (strategy.capability, strategy.capability_reason)
            };

        let visible_window_supported = if capability.verified_support() {
            strategy.visible_window_supported
        } else {
            false
        };
        let window_running = self.window_thread.is_some()
            && matches!(self.lifecycle, NativeRendererSupervisorLifecycle::Running);

        NativeRendererRunnerStatus {
            strategy: strategy.strategy,
            platform: strategy.platform,
            lifecycle: self.runner_lifecycle(),
            supervisor_lifecycle: self.lifecycle,
            threading_model: self.strategy.threading_model(),
            capability,
            capability_reason,
            verified_support: capability.verified_support(),
            visible_window_supported,
            window_visible: window_running && self.window_thread_visible(),
            window_ready: window_running && self.window_ready,
            focus_supported: false,
            native_visual_node_count: self.native_visual_counts.node_count,
            native_visual_edge_count: self.native_visual_counts.edge_count,
            last_error: self.last_error.clone(),
        }
    }
}

impl NativeRendererSupervisor {
    fn window_thread_visible(&self) -> bool {
        matches!(self.lifecycle, NativeRendererSupervisorLifecycle::Running)
    }
}
