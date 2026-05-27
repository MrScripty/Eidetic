use eidetic_core::timeline::node::NodeId;
use eidetic_core::timeline::relationship::{RelationshipId, RelationshipType};
use eidetic_core::timeline::track::TrackId;
use serde::Serialize;
use thiserror::Error;

mod app;
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

pub use app::TimelineRendererApp;
pub use geometry::{TimelineViewportGeometry, TimelineViewportPoint};
pub use hit_test::{
    hit_test_clip_at_point as hit_test_projection_clip_at_point,
    hit_test_clip_at_time as hit_test_projection_clip_at_time,
};
#[cfg(feature = "native_render")]
pub use native_command::{
    emit_timeline_native_create_child_from_parent_request,
    emit_timeline_native_create_relationship_request, emit_timeline_native_delete_node_request,
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
    CreateChildFromParent {
        node_id: NodeId,
        parent_id: NodeId,
    },
    CreateRelationship {
        relationship_id: RelationshipId,
        from_node_id: NodeId,
        to_node_id: NodeId,
        relationship_type: RelationshipType,
    },
    SetPlayhead {
        position_ms: u64,
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

#[cfg(test)]
mod tests;
