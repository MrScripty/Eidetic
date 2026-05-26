use std::panic::{AssertUnwindSafe, catch_unwind};

use eidetic_bevy_timeline::{TimelineRendererApp, TimelineRendererCommand, TimelineRendererError};
use eidetic_core::contracts::TimelineRenderProjection;

use crate::renderer_window::{
    DesktopRendererRunnerLifecycle, DesktopRendererThreadingModel, DesktopRendererWindowCapability,
    DesktopRendererWindowCapabilityReason, DesktopRendererWindowKind,
    DesktopRendererWindowLifecycle, DesktopRendererWindowPlatform, DesktopRendererWindowStrategy,
    DesktopRendererWindowStrategyStatus,
};
use crate::timeline_renderer_platform_strategy::{
    TimelineRendererPlatformStrategy, TimelineRendererRunnerStartupPlan,
};

pub use crate::bevy_timeline_owner::{
    DesktopTimelineRendererOwner, TIMELINE_RENDERER_COMMAND_QUEUE_CAPACITY,
    TIMELINE_RENDERER_REPLY_TIMEOUT_MS,
};

#[derive(Debug, Clone, PartialEq, Eq, serde::Serialize)]
pub struct TimelineHostStatus {
    pub renderer_window_kind: DesktopRendererWindowKind,
    pub running: bool,
    pub renderer_window_open: bool,
    pub renderer_scene_ready: bool,
    pub renderer_window_lifecycle: DesktopRendererWindowLifecycle,
    pub renderer_runner_lifecycle: DesktopRendererRunnerLifecycle,
    pub renderer_runner_threading_model: DesktopRendererThreadingModel,
    pub renderer_window_strategy: DesktopRendererWindowStrategy,
    pub renderer_window_platform: DesktopRendererWindowPlatform,
    pub renderer_window_capability: DesktopRendererWindowCapability,
    pub renderer_window_capability_reason: DesktopRendererWindowCapabilityReason,
    pub renderer_window_visible: bool,
    pub renderer_window_ready: bool,
    pub renderer_window_verified_support: bool,
    pub renderer_window_visible_supported: bool,
    pub renderer_window_focus_supported: bool,
    pub renderer_window_message: String,
    pub track_count: usize,
    pub clip_count: usize,
    pub relationship_count: usize,
    pub affect_overlay_count: usize,
    pub queued_command_count: usize,
    pub last_error: Option<String>,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum TimelineHostError {
    Renderer(String),
    RendererPanic,
    QueueFull,
    OwnerReplyTimeout,
    OwnerStopped,
}

pub(crate) type TimelineHostResult<T> = Result<T, TimelineHostError>;

pub struct DesktopTimelineHost {
    renderer: Option<TimelineRendererApp>,
    last_error: Option<String>,
}

impl Default for DesktopTimelineHost {
    fn default() -> Self {
        Self::new()
    }
}

impl DesktopTimelineHost {
    pub fn new() -> Self {
        Self {
            renderer: None,
            last_error: None,
        }
    }

    pub fn renderer_unavailable_status(message: String) -> TimelineHostStatus {
        let window_strategy = DesktopRendererWindowStrategyStatus::runner_error_current_platform();
        TimelineHostStatus {
            renderer_window_kind: DesktopRendererWindowKind::Timeline,
            running: false,
            renderer_window_open: false,
            renderer_scene_ready: false,
            renderer_window_lifecycle: DesktopRendererWindowLifecycle::Closed,
            renderer_runner_lifecycle: DesktopRendererRunnerLifecycle::Closed,
            renderer_runner_threading_model: DesktopRendererThreadingModel::Unsupported,
            renderer_window_strategy: window_strategy.strategy,
            renderer_window_platform: window_strategy.platform,
            renderer_window_capability: window_strategy.capability,
            renderer_window_capability_reason: window_strategy.capability_reason,
            renderer_window_visible: false,
            renderer_window_ready: false,
            renderer_window_verified_support: window_strategy.verified_support,
            renderer_window_visible_supported: window_strategy.visible_window_supported,
            renderer_window_focus_supported: false,
            renderer_window_message: "timeline renderer native window is unavailable".to_string(),
            track_count: 0,
            clip_count: 0,
            relationship_count: 0,
            affect_overlay_count: 0,
            queued_command_count: 0,
            last_error: Some(message),
        }
    }

    pub fn start(&mut self) -> TimelineHostStatus {
        if self.renderer.is_none() {
            self.renderer = Some(TimelineRendererApp::new());
        }
        self.last_error = None;
        self.status()
    }

    pub fn focus(&mut self) -> TimelineHostStatus {
        self.status()
    }

    pub fn stop(&mut self) -> TimelineHostStatus {
        self.renderer = None;
        self.last_error = None;
        self.status()
    }

    pub fn set_projection(
        &mut self,
        projection: TimelineRenderProjection,
    ) -> Result<TimelineHostStatus, TimelineHostError> {
        self.start();
        let Some(renderer) = self.renderer.as_mut() else {
            return Err(TimelineHostError::Renderer(
                TimelineRendererError::MissingProjection.to_string(),
            ));
        };

        Self::catch_renderer_panic(|| renderer.set_projection(projection)).map_err(|error| {
            self.last_error = Some(error_label(&error));
            error
        })?;
        self.last_error = None;
        Ok(self.status())
    }

    pub fn drain_commands(&mut self) -> Vec<TimelineRendererCommand> {
        self.renderer
            .as_mut()
            .map(TimelineRendererApp::drain_commands)
            .unwrap_or_default()
    }

    pub fn status(&self) -> TimelineHostStatus {
        let platform_strategy = TimelineRendererPlatformStrategy::current();
        let window_strategy = platform_strategy.status();
        let renderer_runner_threading_model = match platform_strategy.runner_startup_plan() {
            TimelineRendererRunnerStartupPlan::MinimalWindowProofCandidate {
                threading_model,
                ..
            }
            | TimelineRendererRunnerStartupPlan::PendingOnly { threading_model } => threading_model,
        };
        let (track_count, clip_count) = self
            .renderer
            .as_ref()
            .map(TimelineRendererApp::scene_counts)
            .unwrap_or_default();
        let relationship_count = self
            .renderer
            .as_ref()
            .map(TimelineRendererApp::scene_relationship_count)
            .unwrap_or_default();
        let affect_overlay_count = self
            .renderer
            .as_ref()
            .map(TimelineRendererApp::scene_affect_overlay_count)
            .unwrap_or_default();
        let queued_command_count = self
            .renderer
            .as_ref()
            .map(TimelineRendererApp::queued_command_count)
            .unwrap_or_default();

        TimelineHostStatus {
            renderer_window_kind: DesktopRendererWindowKind::Timeline,
            running: self.renderer.is_some(),
            renderer_window_open: self.renderer.is_some(),
            renderer_scene_ready: self.renderer.is_some(),
            renderer_window_lifecycle: DesktopRendererWindowLifecycle::from_state(
                self.renderer.is_some(),
                self.renderer.is_some(),
                false,
            ),
            renderer_runner_lifecycle: DesktopRendererRunnerLifecycle::Closed,
            renderer_runner_threading_model,
            renderer_window_strategy: window_strategy.strategy,
            renderer_window_platform: window_strategy.platform,
            renderer_window_capability: window_strategy.capability,
            renderer_window_capability_reason: window_strategy.capability_reason,
            renderer_window_visible: false,
            renderer_window_ready: false,
            renderer_window_verified_support: window_strategy.verified_support,
            renderer_window_visible_supported: window_strategy.visible_window_supported,
            renderer_window_focus_supported: false,
            renderer_window_message: timeline_renderer_window_message(self.renderer.is_some()),
            track_count,
            clip_count,
            relationship_count,
            affect_overlay_count,
            queued_command_count,
            last_error: self.last_error.clone(),
        }
    }

    fn catch_renderer_panic<T>(operation: impl FnOnce() -> T) -> Result<T, TimelineHostError> {
        catch_unwind(AssertUnwindSafe(operation)).map_err(|_| TimelineHostError::RendererPanic)
    }
}

fn timeline_renderer_window_message(running: bool) -> String {
    if running {
        "timeline renderer scene is ready; native window is not connected".to_string()
    } else {
        "floating timeline renderer window is closed".to_string()
    }
}

fn error_label(error: &TimelineHostError) -> String {
    match error {
        TimelineHostError::Renderer(message) => message.clone(),
        TimelineHostError::RendererPanic => "timeline renderer panicked".to_string(),
        TimelineHostError::QueueFull => "timeline renderer command queue is full".to_string(),
        TimelineHostError::OwnerReplyTimeout => {
            "timeline renderer owner reply timed out".to_string()
        }
        TimelineHostError::OwnerStopped => "timeline renderer owner stopped".to_string(),
    }
}

#[cfg(test)]
mod tests {
    use eidetic_core::contracts::{
        TimelineRenderClip, TimelineRenderProjection, TimelineRenderTrack,
    };
    use eidetic_core::timeline::node::{ContentStatus, NodeId, StoryLevel};
    use eidetic_core::timeline::track::TrackId;

    use super::*;

    #[test]
    fn timeline_host_ingests_projection_and_reports_scene_counts() {
        let node_id = NodeId::new();
        let mut host = DesktopTimelineHost::new();

        let status = host.set_projection(projection_with_node(node_id)).unwrap();

        assert_eq!(
            status.renderer_window_kind,
            DesktopRendererWindowKind::Timeline
        );
        assert!(status.running);
        assert!(status.renderer_window_open);
        assert!(status.renderer_scene_ready);
        assert_eq!(
            status.renderer_window_lifecycle,
            DesktopRendererWindowLifecycle::SceneReadyPendingNativeRunner
        );
        assert_eq!(
            status.renderer_runner_lifecycle,
            DesktopRendererRunnerLifecycle::Closed
        );
        assert_eq!(
            status.renderer_runner_threading_model,
            expected_threading_model()
        );
        assert_eq!(
            status.renderer_window_strategy,
            DesktopRendererWindowStrategy::BevyWinitFloatingWindow
        );
        assert_eq!(
            status.renderer_window_capability,
            expected_strategy_status().capability
        );
        assert_eq!(
            status.renderer_window_capability_reason,
            expected_strategy_status().capability_reason
        );
        assert_eq!(
            status.renderer_window_verified_support,
            expected_strategy_status().verified_support
        );
        assert_eq!(
            status.renderer_window_visible_supported,
            expected_strategy_status().visible_window_supported
        );
        assert!(!status.renderer_window_visible);
        assert!(!status.renderer_window_ready);
        assert!(!status.renderer_window_focus_supported);
        assert_eq!(
            status.renderer_window_message,
            "timeline renderer scene is ready; native window is not connected"
        );
        assert_eq!(status.track_count, 1);
        assert_eq!(status.clip_count, 1);
        assert_eq!(status.relationship_count, 0);
        assert_eq!(status.affect_overlay_count, 0);
        assert_eq!(status.queued_command_count, 0);
    }

    #[test]
    fn timeline_host_drain_commands_is_empty_without_renderer_commands() {
        let mut host = DesktopTimelineHost::new();

        assert!(host.drain_commands().is_empty());
        assert!(!host.status().running);
    }

    #[test]
    fn timeline_host_stop_clears_projection_state() {
        let node_id = NodeId::new();
        let mut host = DesktopTimelineHost::new();
        host.set_projection(projection_with_node(node_id)).unwrap();

        let status = host.stop();

        assert!(!status.running);
        assert!(!status.renderer_window_open);
        assert!(!status.renderer_scene_ready);
        assert_eq!(
            status.renderer_window_lifecycle,
            DesktopRendererWindowLifecycle::Closed
        );
        assert_eq!(
            status.renderer_runner_lifecycle,
            DesktopRendererRunnerLifecycle::Closed
        );
        assert_eq!(
            status.renderer_window_capability,
            expected_strategy_status().capability
        );
        assert!(!status.renderer_window_visible);
        assert!(!status.renderer_window_ready);
        assert!(!status.renderer_window_focus_supported);
        assert_eq!(
            status.renderer_window_message,
            "floating timeline renderer window is closed"
        );
        assert_eq!(status.track_count, 0);
        assert_eq!(status.clip_count, 0);
    }

    #[test]
    fn timeline_owner_uses_bounded_command_queue() {
        assert_eq!(TIMELINE_RENDERER_COMMAND_QUEUE_CAPACITY, 128);
        assert_eq!(TIMELINE_RENDERER_REPLY_TIMEOUT_MS, 2_000);
    }

    #[test]
    fn timeline_owner_ingests_projection_on_renderer_thread() {
        let node_id = NodeId::new();
        let owner = DesktopTimelineRendererOwner::start().unwrap();

        let status = owner.set_projection(projection_with_node(node_id)).unwrap();
        let stopped = owner.stop().unwrap();

        assert!(status.running);
        assert_eq!(status.clip_count, 1);
        assert!(!stopped.running);
    }

    #[test]
    fn timeline_owner_focus_reports_current_lifecycle_without_starting_renderer() {
        let owner = DesktopTimelineRendererOwner::start().unwrap();

        let status = owner.focus_renderer().unwrap();
        let stopped = owner.stop().unwrap();

        assert!(!status.running);
        assert_eq!(
            status.renderer_window_lifecycle,
            DesktopRendererWindowLifecycle::Closed
        );
        assert_eq!(
            status.renderer_runner_lifecycle,
            DesktopRendererRunnerLifecycle::Closed
        );
        assert_eq!(
            status.renderer_window_capability,
            expected_strategy_status().capability
        );
        assert!(!status.renderer_window_focus_supported);
        assert!(!stopped.running);
    }

    fn projection_with_node(node_id: NodeId) -> TimelineRenderProjection {
        let track_id = TrackId::new();
        TimelineRenderProjection {
            total_duration_ms: 10_000,
            structure_segments: Vec::new(),
            tracks: vec![TimelineRenderTrack {
                track_id,
                level: StoryLevel::Scene,
                label: "Scenes".to_string(),
                sort_order: 30,
                collapsed: false,
            }],
            clips: vec![TimelineRenderClip {
                node_id,
                parent_id: None,
                track_id,
                level: StoryLevel::Scene,
                name: "Beach argument".to_string(),
                start_ms: 1_000,
                end_ms: 4_000,
                sort_order: 10,
                locked: false,
                content_status: ContentStatus::NotesOnly,
                beat_type: None,
                arc_ids: Vec::new(),
            }],
            relationships: Vec::new(),
            gaps: Vec::new(),
            affect_overlays: Vec::new(),
        }
    }

    fn expected_strategy_status() -> DesktopRendererWindowStrategyStatus {
        TimelineRendererPlatformStrategy::current().status()
    }

    fn expected_threading_model() -> DesktopRendererThreadingModel {
        match TimelineRendererPlatformStrategy::current().runner_startup_plan() {
            TimelineRendererRunnerStartupPlan::MinimalWindowProofCandidate {
                threading_model,
                ..
            }
            | TimelineRendererRunnerStartupPlan::PendingOnly { threading_model } => threading_model,
        }
    }
}
