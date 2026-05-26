use std::panic::{AssertUnwindSafe, catch_unwind};

use eidetic_bevy_timeline::{TimelineRendererApp, TimelineRendererCommand, TimelineRendererError};
use eidetic_core::contracts::TimelineRenderProjection;

use crate::renderer_window::DesktopRendererWindowKind;

#[derive(Debug, Clone, PartialEq, Eq, serde::Serialize)]
pub struct TimelineHostStatus {
    pub renderer_window_kind: DesktopRendererWindowKind,
    pub running: bool,
    pub renderer_scene_ready: bool,
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
}

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

    pub fn start(&mut self) -> TimelineHostStatus {
        if self.renderer.is_none() {
            self.renderer = Some(TimelineRendererApp::new());
        }
        self.last_error = None;
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
            renderer_scene_ready: self.renderer.is_some(),
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

fn error_label(error: &TimelineHostError) -> String {
    match error {
        TimelineHostError::Renderer(message) => message.clone(),
        TimelineHostError::RendererPanic => "timeline renderer panicked".to_string(),
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
        assert!(status.renderer_scene_ready);
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
        assert_eq!(status.track_count, 0);
        assert_eq!(status.clip_count, 0);
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
}
