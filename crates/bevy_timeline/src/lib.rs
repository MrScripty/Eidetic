use bevy::prelude::{App, Resource};
use eidetic_core::contracts::TimelineRenderProjection;
use eidetic_core::timeline::node::NodeId;
use eidetic_core::timeline::track::TrackId;
use serde::Serialize;
use thiserror::Error;

mod scene;
mod viewport;
#[cfg(target_arch = "wasm32")]
mod wasm;

pub use scene::{
    TimelineClipEntity, TimelineSceneStats, TimelineTrackEntity, rebuild_timeline_scene,
};
pub use viewport::TimelineViewport;
#[cfg(target_arch = "wasm32")]
pub use wasm::WasmTimelineRenderer;

#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
#[serde(tag = "type", rename_all = "snake_case")]
pub enum TimelineRendererCommand {
    SelectNode {
        node_id: NodeId,
    },
    SetNodeRange {
        node_id: NodeId,
        start_ms: u64,
        end_ms: u64,
    },
    SplitNode {
        node_id: NodeId,
        at_ms: u64,
    },
}

#[derive(Debug, Error, PartialEq, Eq)]
pub enum TimelineRendererError {
    #[error("timeline projection has not been loaded")]
    MissingProjection,
    #[error("timeline projection does not contain node {node_id:?}")]
    UnknownNode { node_id: NodeId },
    #[error("timeline projection has no clip on track {track_id:?} at {time_ms}ms")]
    NoClipAtTime { track_id: TrackId, time_ms: u64 },
    #[error("invalid node range {start_ms}ms..{end_ms}ms for duration {duration_ms}ms")]
    InvalidNodeRange {
        start_ms: u64,
        end_ms: u64,
        duration_ms: u64,
    },
    #[error("invalid split at {at_ms}ms for node range {start_ms}ms..{end_ms}ms")]
    InvalidNodeSplit {
        at_ms: u64,
        start_ms: u64,
        end_ms: u64,
    },
    #[error("invalid viewport range {start_ms}ms..{end_ms}ms for duration {duration_ms}ms")]
    InvalidViewportRange {
        start_ms: u64,
        end_ms: u64,
        duration_ms: u64,
    },
    #[error("viewport zoom factor must be finite and greater than zero")]
    InvalidZoomFactor,
}

#[derive(Resource, Default)]
struct TimelineRenderState {
    projection: Option<TimelineRenderProjection>,
}

#[derive(Resource, Default)]
struct TimelineRendererCommandQueue {
    commands: Vec<TimelineRendererCommand>,
}

pub struct TimelineRendererApp {
    app: App,
}

impl Default for TimelineRendererApp {
    fn default() -> Self {
        Self::new()
    }
}

impl TimelineRendererApp {
    pub fn new() -> Self {
        let mut app = App::new();
        app.insert_resource(TimelineRenderState::default());
        app.insert_resource(TimelineRendererCommandQueue::default());
        app.insert_resource(TimelineSceneStats::default());
        app.insert_resource(TimelineViewport::default());
        Self { app }
    }

    pub fn set_projection(&mut self, projection: TimelineRenderProjection) {
        self.app
            .world_mut()
            .resource_mut::<TimelineViewport>()
            .set_duration(projection.total_duration_ms);
        self.app
            .world_mut()
            .resource_mut::<TimelineRenderState>()
            .projection = Some(projection.clone());
        rebuild_timeline_scene(self.app.world_mut(), &projection);
    }

    pub fn projection_clip_count(&self) -> usize {
        self.app
            .world()
            .resource::<TimelineRenderState>()
            .projection
            .as_ref()
            .map(|projection| projection.clips.len())
            .unwrap_or_default()
    }

    pub fn scene_counts(&self) -> (usize, usize) {
        let stats = self.app.world().resource::<TimelineSceneStats>();
        (stats.track_count, stats.clip_count)
    }

    pub fn viewport(&self) -> TimelineViewport {
        *self.app.world().resource::<TimelineViewport>()
    }

    pub fn set_viewport(
        &mut self,
        start_ms: u64,
        end_ms: u64,
    ) -> Result<(), TimelineRendererError> {
        let duration_ms = self.viewport().duration_ms;
        if start_ms >= end_ms || end_ms > duration_ms {
            return Err(TimelineRendererError::InvalidViewportRange {
                start_ms,
                end_ms,
                duration_ms,
            });
        }

        self.app
            .world_mut()
            .resource_mut::<TimelineViewport>()
            .set_range(start_ms, end_ms);
        Ok(())
    }

    pub fn pan_viewport(&mut self, delta_ms: i64) {
        self.app
            .world_mut()
            .resource_mut::<TimelineViewport>()
            .pan_by(delta_ms);
    }

    pub fn zoom_viewport_around(
        &mut self,
        center_ms: u64,
        factor: f32,
    ) -> Result<(), TimelineRendererError> {
        if !factor.is_finite() || factor <= 0.0 {
            return Err(TimelineRendererError::InvalidZoomFactor);
        }

        self.app
            .world_mut()
            .resource_mut::<TimelineViewport>()
            .zoom_around(center_ms, factor);
        Ok(())
    }

    pub fn select_node(&mut self, node_id: NodeId) -> Result<(), TimelineRendererError> {
        let has_node = {
            let state = self.app.world().resource::<TimelineRenderState>();
            let projection = state
                .projection
                .as_ref()
                .ok_or(TimelineRendererError::MissingProjection)?;
            projection.clips.iter().any(|clip| clip.node_id == node_id)
        };

        if !has_node {
            return Err(TimelineRendererError::UnknownNode { node_id });
        }

        self.app
            .world_mut()
            .resource_mut::<TimelineRendererCommandQueue>()
            .commands
            .push(TimelineRendererCommand::SelectNode { node_id });
        Ok(())
    }

    pub fn hit_test_clip_at_time(
        &self,
        track_id: TrackId,
        time_ms: u64,
    ) -> Result<Option<NodeId>, TimelineRendererError> {
        let state = self.app.world().resource::<TimelineRenderState>();
        let projection = state
            .projection
            .as_ref()
            .ok_or(TimelineRendererError::MissingProjection)?;

        Ok(projection
            .clips
            .iter()
            .filter(|clip| {
                clip.track_id == track_id && clip.start_ms <= time_ms && time_ms < clip.end_ms
            })
            .max_by_key(|clip| (clip.sort_order, clip.start_ms))
            .map(|clip| clip.node_id))
    }

    pub fn select_clip_at_time(
        &mut self,
        track_id: TrackId,
        time_ms: u64,
    ) -> Result<(), TimelineRendererError> {
        let node_id = self
            .hit_test_clip_at_time(track_id, time_ms)?
            .ok_or(TimelineRendererError::NoClipAtTime { track_id, time_ms })?;

        self.app
            .world_mut()
            .resource_mut::<TimelineRendererCommandQueue>()
            .commands
            .push(TimelineRendererCommand::SelectNode { node_id });
        Ok(())
    }

    pub fn request_node_range(
        &mut self,
        node_id: NodeId,
        start_ms: u64,
        end_ms: u64,
    ) -> Result<(), TimelineRendererError> {
        let duration_ms = {
            let state = self.app.world().resource::<TimelineRenderState>();
            let projection = state
                .projection
                .as_ref()
                .ok_or(TimelineRendererError::MissingProjection)?;
            if !projection.clips.iter().any(|clip| clip.node_id == node_id) {
                return Err(TimelineRendererError::UnknownNode { node_id });
            }
            projection.total_duration_ms
        };

        if start_ms >= end_ms || end_ms > duration_ms {
            return Err(TimelineRendererError::InvalidNodeRange {
                start_ms,
                end_ms,
                duration_ms,
            });
        }

        self.app
            .world_mut()
            .resource_mut::<TimelineRendererCommandQueue>()
            .commands
            .push(TimelineRendererCommand::SetNodeRange {
                node_id,
                start_ms,
                end_ms,
            });
        Ok(())
    }

    pub fn request_split_node(
        &mut self,
        node_id: NodeId,
        at_ms: u64,
    ) -> Result<(), TimelineRendererError> {
        let (start_ms, end_ms) = {
            let state = self.app.world().resource::<TimelineRenderState>();
            let projection = state
                .projection
                .as_ref()
                .ok_or(TimelineRendererError::MissingProjection)?;
            let Some(clip) = projection.clips.iter().find(|clip| clip.node_id == node_id) else {
                return Err(TimelineRendererError::UnknownNode { node_id });
            };
            (clip.start_ms, clip.end_ms)
        };

        if at_ms <= start_ms || at_ms >= end_ms {
            return Err(TimelineRendererError::InvalidNodeSplit {
                at_ms,
                start_ms,
                end_ms,
            });
        }

        self.app
            .world_mut()
            .resource_mut::<TimelineRendererCommandQueue>()
            .commands
            .push(TimelineRendererCommand::SplitNode { node_id, at_ms });
        Ok(())
    }

    pub fn drain_commands(&mut self) -> Vec<TimelineRendererCommand> {
        std::mem::take(
            &mut self
                .app
                .world_mut()
                .resource_mut::<TimelineRendererCommandQueue>()
                .commands,
        )
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use eidetic_core::contracts::{TimelineRenderClip, TimelineRenderTrack};
    use eidetic_core::timeline::node::{ContentStatus, StoryLevel};
    use eidetic_core::timeline::track::TrackId;

    #[test]
    fn renderer_app_receives_projection_and_emits_validated_selection_command() {
        let node_id = NodeId::new();
        let mut renderer = TimelineRendererApp::new();

        renderer.set_projection(projection_with_node(node_id));

        assert_eq!(renderer.projection_clip_count(), 1);
        assert_eq!(renderer.select_node(node_id), Ok(()));
        assert_eq!(
            renderer.drain_commands(),
            vec![TimelineRendererCommand::SelectNode { node_id }]
        );
        assert!(renderer.drain_commands().is_empty());
    }

    #[test]
    fn renderer_app_rebuilds_scene_entities_from_projection() {
        let node_id = NodeId::new();
        let mut renderer = TimelineRendererApp::new();

        renderer.set_projection(projection_with_node(node_id));

        assert_eq!(renderer.scene_counts(), (1, 1));

        renderer.set_projection(TimelineRenderProjection {
            total_duration_ms: 10_000,
            tracks: Vec::new(),
            clips: Vec::new(),
            relationships: Vec::new(),
        });

        assert_eq!(renderer.scene_counts(), (0, 0));
    }

    #[test]
    fn renderer_app_rejects_selection_before_projection_load() {
        let mut renderer = TimelineRendererApp::new();
        let node_id = NodeId::new();

        assert_eq!(
            renderer.select_node(node_id),
            Err(TimelineRendererError::MissingProjection)
        );
        assert!(renderer.drain_commands().is_empty());
    }

    #[test]
    fn renderer_app_rejects_unknown_node_selection() {
        let mut renderer = TimelineRendererApp::new();
        let known_node_id = NodeId::new();
        let unknown_node_id = NodeId::new();
        renderer.set_projection(projection_with_node(known_node_id));

        assert_eq!(
            renderer.select_node(unknown_node_id),
            Err(TimelineRendererError::UnknownNode {
                node_id: unknown_node_id
            })
        );
        assert!(renderer.drain_commands().is_empty());
    }

    #[test]
    fn renderer_app_hit_tests_and_selects_clip_by_track_and_time() {
        let node_id = NodeId::new();
        let track_id = TrackId::new();
        let mut renderer = TimelineRendererApp::new();
        renderer.set_projection(projection_with_clip(node_id, track_id, 1_000, 4_000));

        assert_eq!(
            renderer.hit_test_clip_at_time(track_id, 2_000),
            Ok(Some(node_id))
        );
        assert_eq!(renderer.select_clip_at_time(track_id, 2_000), Ok(()));
        assert_eq!(
            renderer.drain_commands(),
            vec![TimelineRendererCommand::SelectNode { node_id }]
        );
    }

    #[test]
    fn renderer_app_hit_test_misses_empty_time() {
        let node_id = NodeId::new();
        let track_id = TrackId::new();
        let mut renderer = TimelineRendererApp::new();
        renderer.set_projection(projection_with_clip(node_id, track_id, 1_000, 4_000));

        assert_eq!(renderer.hit_test_clip_at_time(track_id, 4_000), Ok(None));
        assert_eq!(
            renderer.select_clip_at_time(track_id, 4_000),
            Err(TimelineRendererError::NoClipAtTime {
                track_id,
                time_ms: 4_000
            })
        );
        assert!(renderer.drain_commands().is_empty());
    }

    #[test]
    fn renderer_app_keeps_transient_viewport_inside_projection_duration() {
        let node_id = NodeId::new();
        let mut renderer = TimelineRendererApp::new();
        renderer.set_projection(projection_with_node(node_id));

        assert_eq!(
            renderer.viewport(),
            TimelineViewport {
                start_ms: 0,
                end_ms: 10_000,
                duration_ms: 10_000
            }
        );

        assert_eq!(renderer.set_viewport(2_000, 6_000), Ok(()));
        renderer.pan_viewport(10_000);
        assert_eq!(
            renderer.viewport(),
            TimelineViewport {
                start_ms: 6_000,
                end_ms: 10_000,
                duration_ms: 10_000
            }
        );

        renderer.pan_viewport(-10_000);
        assert_eq!(
            renderer.viewport(),
            TimelineViewport {
                start_ms: 0,
                end_ms: 4_000,
                duration_ms: 10_000
            }
        );
    }

    #[test]
    fn renderer_app_zooms_viewport_around_time() {
        let node_id = NodeId::new();
        let mut renderer = TimelineRendererApp::new();
        renderer.set_projection(projection_with_node(node_id));

        assert_eq!(renderer.zoom_viewport_around(5_000, 2.0), Ok(()));
        assert_eq!(
            renderer.viewport(),
            TimelineViewport {
                start_ms: 2_500,
                end_ms: 7_500,
                duration_ms: 10_000
            }
        );

        assert_eq!(
            renderer.zoom_viewport_around(5_000, 0.0),
            Err(TimelineRendererError::InvalidZoomFactor)
        );
    }

    #[test]
    fn renderer_app_emits_validated_node_range_command() {
        let node_id = NodeId::new();
        let mut renderer = TimelineRendererApp::new();
        renderer.set_projection(projection_with_node(node_id));

        assert_eq!(renderer.request_node_range(node_id, 2_000, 5_000), Ok(()));
        assert_eq!(
            renderer.drain_commands(),
            vec![TimelineRendererCommand::SetNodeRange {
                node_id,
                start_ms: 2_000,
                end_ms: 5_000
            }]
        );
    }

    #[test]
    fn renderer_app_rejects_invalid_node_range_command() {
        let node_id = NodeId::new();
        let mut renderer = TimelineRendererApp::new();
        renderer.set_projection(projection_with_node(node_id));

        assert_eq!(
            renderer.request_node_range(node_id, 5_000, 2_000),
            Err(TimelineRendererError::InvalidNodeRange {
                start_ms: 5_000,
                end_ms: 2_000,
                duration_ms: 10_000
            })
        );
        assert!(renderer.drain_commands().is_empty());
    }

    #[test]
    fn renderer_app_emits_validated_split_node_command() {
        let node_id = NodeId::new();
        let mut renderer = TimelineRendererApp::new();
        renderer.set_projection(projection_with_node(node_id));

        assert_eq!(renderer.request_split_node(node_id, 2_500), Ok(()));
        assert_eq!(
            renderer.drain_commands(),
            vec![TimelineRendererCommand::SplitNode {
                node_id,
                at_ms: 2_500
            }]
        );
    }

    #[test]
    fn renderer_app_rejects_split_node_command_outside_clip() {
        let node_id = NodeId::new();
        let mut renderer = TimelineRendererApp::new();
        renderer.set_projection(projection_with_node(node_id));

        assert_eq!(
            renderer.request_split_node(node_id, 4_000),
            Err(TimelineRendererError::InvalidNodeSplit {
                at_ms: 4_000,
                start_ms: 1_000,
                end_ms: 4_000
            })
        );
        assert!(renderer.drain_commands().is_empty());
    }

    fn projection_with_node(node_id: NodeId) -> TimelineRenderProjection {
        let track_id = TrackId::new();
        projection_with_clip(node_id, track_id, 1_000, 4_000)
    }

    fn projection_with_clip(
        node_id: NodeId,
        track_id: TrackId,
        start_ms: u64,
        end_ms: u64,
    ) -> TimelineRenderProjection {
        TimelineRenderProjection {
            total_duration_ms: 10_000,
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
                start_ms,
                end_ms,
                sort_order: 10,
                locked: false,
                content_status: ContentStatus::NotesOnly,
                beat_type: None,
                arc_ids: Vec::new(),
            }],
            relationships: Vec::new(),
        }
    }
}
