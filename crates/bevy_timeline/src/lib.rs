use bevy::prelude::{App, Resource};
use eidetic_core::contracts::TimelineRenderProjection;
use eidetic_core::timeline::node::{BeatType, NodeId, StoryLevel};
use eidetic_core::timeline::track::TrackId;
use serde::Serialize;
use thiserror::Error;

mod geometry;
mod hit_test;
#[cfg(feature = "native_render")]
mod native_command;
#[cfg(feature = "native_render")]
mod native_input;
#[cfg(feature = "native_render")]
mod native_render;
#[cfg(feature = "native_render")]
mod native_style;
#[cfg(feature = "native_render")]
mod native_visual;
#[cfg(feature = "native_render")]
mod native_window_control;
mod playhead;
mod relationship_curve;
mod scene;
mod viewport;

pub use geometry::{TimelineViewportGeometry, TimelineViewportPoint};
pub use hit_test::{
    hit_test_clip_at_point as hit_test_projection_clip_at_point,
    hit_test_clip_at_time as hit_test_projection_clip_at_time,
};
#[cfg(feature = "native_render")]
pub use native_command::{
    emit_timeline_native_create_node_request, emit_timeline_native_delete_node_request,
    emit_timeline_native_node_range_request, emit_timeline_native_split_node_request,
};
#[cfg(feature = "native_render")]
pub use native_render::{
    TimelineNativeRenderConfig, TimelineNativeWindowControl, TimelineNativeWindowControlHandle,
    TimelineNativeWindowProjectionUpdateError, TimelineNativeWindowRunnerConfig,
    configure_controlled_minimal_timeline_native_window_app,
    configure_minimal_timeline_native_window_app, nudge_timeline_native_playhead,
    pan_timeline_native_viewport, run_controlled_minimal_timeline_native_window,
    run_minimal_timeline_native_window, set_timeline_native_playhead, set_timeline_native_viewport,
    zoom_timeline_native_viewport,
};
pub use playhead::TimelinePlayhead;
pub use relationship_curve::{TimelineCurvePoint, TimelineRelationshipCurve, relationship_curves};
pub use scene::{
    TimelineAffectOverlayEntity, TimelineClipEntity, TimelineRelationshipEntity,
    TimelineSceneStats, TimelineTrackEntity, rebuild_timeline_scene,
};
pub use viewport::TimelineViewport;

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
        left_node_id: NodeId,
        right_node_id: NodeId,
    },
    DeleteNode {
        node_id: NodeId,
    },
    CreateNode {
        node_id: NodeId,
        parent_id: Option<NodeId>,
        level: StoryLevel,
        name: String,
        start_ms: u64,
        end_ms: u64,
        beat_type: Option<BeatType>,
    },
}

#[derive(Debug, Error, PartialEq, Eq)]
pub enum TimelineRendererError {
    #[error("timeline projection has not been loaded")]
    MissingProjection,
    #[error("timeline projection does not contain node {node_id:?}")]
    UnknownNode { node_id: NodeId },
    #[error("timeline relationship endpoint does not contain node {node_id:?}")]
    UnknownRelationshipEndpoint { node_id: NodeId },
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
    #[error("split output node ids must be distinct new node ids")]
    InvalidSplitOutputNodeIds {
        left_node_id: NodeId,
        right_node_id: NodeId,
    },
    #[error("invalid viewport range {start_ms}ms..{end_ms}ms for duration {duration_ms}ms")]
    InvalidViewportRange {
        start_ms: u64,
        end_ms: u64,
        duration_ms: u64,
    },
    #[error("invalid playhead position {position_ms}ms for duration {duration_ms}ms")]
    InvalidPlayheadPosition { position_ms: u64, duration_ms: u64 },
    #[error("viewport zoom factor must be finite and greater than zero")]
    InvalidZoomFactor,
    #[error(
        "invalid viewport geometry {width_px}px x {height_px}px with {track_height_px}px tracks"
    )]
    InvalidViewportGeometry {
        width_px: u32,
        height_px: u32,
        track_height_px: u32,
    },
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
        app.insert_resource(TimelinePlayhead::default());
        Self { app }
    }

    pub fn set_projection(&mut self, projection: TimelineRenderProjection) {
        self.app
            .world_mut()
            .resource_mut::<TimelineViewport>()
            .set_duration(projection.total_duration_ms);
        self.app
            .world_mut()
            .resource_mut::<TimelinePlayhead>()
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

    pub fn scene_relationship_count(&self) -> usize {
        self.app
            .world()
            .resource::<TimelineSceneStats>()
            .relationship_count
    }

    pub fn scene_affect_overlay_count(&self) -> usize {
        self.app
            .world()
            .resource::<TimelineSceneStats>()
            .affect_overlay_count
    }

    pub fn relationship_curves(
        &self,
    ) -> Result<Vec<TimelineRelationshipCurve>, TimelineRendererError> {
        let state = self.app.world().resource::<TimelineRenderState>();
        let projection = state
            .projection
            .as_ref()
            .ok_or(TimelineRendererError::MissingProjection)?;
        relationship_curves(projection)
    }

    pub fn viewport(&self) -> TimelineViewport {
        *self.app.world().resource::<TimelineViewport>()
    }

    pub fn playhead(&self) -> TimelinePlayhead {
        *self.app.world().resource::<TimelinePlayhead>()
    }

    pub fn set_playhead(&mut self, position_ms: u64) -> Result<(), TimelineRendererError> {
        let duration_ms = self.playhead().duration_ms;
        if position_ms > duration_ms {
            return Err(TimelineRendererError::InvalidPlayheadPosition {
                position_ms,
                duration_ms,
            });
        }
        self.app
            .world_mut()
            .resource_mut::<TimelinePlayhead>()
            .set_position(position_ms);
        Ok(())
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

        Ok(hit_test::hit_test_clip_at_time(
            projection, track_id, time_ms,
        ))
    }

    pub fn hit_test_clip_at_point(
        &self,
        geometry: TimelineViewportGeometry,
        point: TimelineViewportPoint,
    ) -> Result<Option<NodeId>, TimelineRendererError> {
        let state = self.app.world().resource::<TimelineRenderState>();
        let projection = state
            .projection
            .as_ref()
            .ok_or(TimelineRendererError::MissingProjection)?;
        hit_test::hit_test_clip_at_point(projection, self.viewport(), geometry, point)
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

    pub fn request_create_node(
        &mut self,
        node_id: NodeId,
        parent_id: Option<NodeId>,
        level: StoryLevel,
        name: String,
        start_ms: u64,
        end_ms: u64,
        beat_type: Option<BeatType>,
    ) -> Result<(), TimelineRendererError> {
        let duration_ms = {
            let state = self.app.world().resource::<TimelineRenderState>();
            let projection = state
                .projection
                .as_ref()
                .ok_or(TimelineRendererError::MissingProjection)?;
            if let Some(parent_id) = parent_id
                && !projection
                    .clips
                    .iter()
                    .any(|clip| clip.node_id == parent_id)
            {
                return Err(TimelineRendererError::UnknownNode { node_id: parent_id });
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
            .push(TimelineRendererCommand::CreateNode {
                node_id,
                parent_id,
                level,
                name,
                start_ms,
                end_ms,
                beat_type,
            });
        Ok(())
    }

    pub fn request_split_node(
        &mut self,
        node_id: NodeId,
        at_ms: u64,
        left_node_id: NodeId,
        right_node_id: NodeId,
    ) -> Result<(), TimelineRendererError> {
        let (start_ms, end_ms, output_ids_are_available) = {
            let state = self.app.world().resource::<TimelineRenderState>();
            let projection = state
                .projection
                .as_ref()
                .ok_or(TimelineRendererError::MissingProjection)?;
            let Some(clip) = projection.clips.iter().find(|clip| clip.node_id == node_id) else {
                return Err(TimelineRendererError::UnknownNode { node_id });
            };
            let output_ids_are_available = left_node_id != right_node_id
                && !projection
                    .clips
                    .iter()
                    .any(|clip| clip.node_id == left_node_id || clip.node_id == right_node_id);
            (clip.start_ms, clip.end_ms, output_ids_are_available)
        };

        if at_ms <= start_ms || at_ms >= end_ms {
            return Err(TimelineRendererError::InvalidNodeSplit {
                at_ms,
                start_ms,
                end_ms,
            });
        }
        if !output_ids_are_available {
            return Err(TimelineRendererError::InvalidSplitOutputNodeIds {
                left_node_id,
                right_node_id,
            });
        }

        self.app
            .world_mut()
            .resource_mut::<TimelineRendererCommandQueue>()
            .commands
            .push(TimelineRendererCommand::SplitNode {
                node_id,
                at_ms,
                left_node_id,
                right_node_id,
            });
        Ok(())
    }

    pub fn request_delete_node(&mut self, node_id: NodeId) -> Result<(), TimelineRendererError> {
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
            .push(TimelineRendererCommand::DeleteNode { node_id });
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

    pub fn queued_command_count(&self) -> usize {
        self.app
            .world()
            .resource::<TimelineRendererCommandQueue>()
            .commands
            .len()
    }
}

#[cfg(test)]
mod tests;
