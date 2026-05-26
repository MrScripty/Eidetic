use eidetic_core::contracts::TimelineRenderProjection;
use eidetic_core::timeline::node::{BeatType, NodeId, StoryLevel};

use crate::native_render::TimelineNativeWindowControl;
use crate::{
    TimelineRendererCommand, TimelineRendererError, TimelineViewport, TimelineViewportGeometry,
    TimelineViewportPoint, hit_test_projection_clip_at_point,
};

pub(crate) fn emit_timeline_native_clip_selection(
    control: &TimelineNativeWindowControl,
    projection: &TimelineRenderProjection,
    viewport: TimelineViewport,
    geometry: TimelineViewportGeometry,
    point: TimelineViewportPoint,
) -> Result<Option<NodeId>, TimelineRendererError> {
    let node_id = hit_test_projection_clip_at_point(projection, viewport, geometry, point)?;
    if let Some(node_id) = node_id {
        control.enqueue_command(TimelineRendererCommand::SelectNode { node_id });
    }
    Ok(node_id)
}

pub fn emit_timeline_native_node_range_request(
    control: &TimelineNativeWindowControl,
    projection: &TimelineRenderProjection,
    node_id: NodeId,
    start_ms: u64,
    end_ms: u64,
) -> Result<(), TimelineRendererError> {
    if !projection.clips.iter().any(|clip| clip.node_id == node_id) {
        return Err(TimelineRendererError::UnknownNode { node_id });
    }
    if start_ms >= end_ms || end_ms > projection.total_duration_ms {
        return Err(TimelineRendererError::InvalidNodeRange {
            start_ms,
            end_ms,
            duration_ms: projection.total_duration_ms,
        });
    }

    control.enqueue_command(TimelineRendererCommand::SetNodeRange {
        node_id,
        start_ms,
        end_ms,
    });
    Ok(())
}

pub fn emit_timeline_native_delete_node_request(
    control: &TimelineNativeWindowControl,
    projection: &TimelineRenderProjection,
    node_id: NodeId,
) -> Result<(), TimelineRendererError> {
    if !projection.clips.iter().any(|clip| clip.node_id == node_id) {
        return Err(TimelineRendererError::UnknownNode { node_id });
    }

    control.enqueue_command(TimelineRendererCommand::DeleteNode { node_id });
    Ok(())
}

pub fn emit_timeline_native_selected_delete_request(
    control: &TimelineNativeWindowControl,
    projection: &TimelineRenderProjection,
) -> Result<Option<NodeId>, TimelineRendererError> {
    let Some(node_id) = projection.selected_node_id else {
        return Ok(None);
    };
    emit_timeline_native_delete_node_request(control, projection, node_id)?;
    Ok(Some(node_id))
}

pub fn emit_timeline_native_split_node_request(
    control: &TimelineNativeWindowControl,
    projection: &TimelineRenderProjection,
    node_id: NodeId,
    at_ms: u64,
    left_node_id: NodeId,
    right_node_id: NodeId,
) -> Result<(), TimelineRendererError> {
    let Some(clip) = projection.clips.iter().find(|clip| clip.node_id == node_id) else {
        return Err(TimelineRendererError::UnknownNode { node_id });
    };
    if at_ms <= clip.start_ms || at_ms >= clip.end_ms {
        return Err(TimelineRendererError::InvalidNodeSplit {
            at_ms,
            start_ms: clip.start_ms,
            end_ms: clip.end_ms,
        });
    }
    let output_ids_are_available = left_node_id != right_node_id
        && !projection
            .clips
            .iter()
            .any(|clip| clip.node_id == left_node_id || clip.node_id == right_node_id);
    if !output_ids_are_available {
        return Err(TimelineRendererError::InvalidSplitOutputNodeIds {
            left_node_id,
            right_node_id,
        });
    }

    control.enqueue_command(TimelineRendererCommand::SplitNode {
        node_id,
        at_ms,
        left_node_id,
        right_node_id,
    });
    Ok(())
}

pub fn emit_timeline_native_selected_split_request(
    control: &TimelineNativeWindowControl,
    projection: &TimelineRenderProjection,
    at_ms: u64,
    left_node_id: NodeId,
    right_node_id: NodeId,
) -> Result<Option<NodeId>, TimelineRendererError> {
    let Some(node_id) = projection.selected_node_id else {
        return Ok(None);
    };
    emit_timeline_native_split_node_request(
        control,
        projection,
        node_id,
        at_ms,
        left_node_id,
        right_node_id,
    )?;
    Ok(Some(node_id))
}

pub fn emit_timeline_native_create_node_request(
    control: &TimelineNativeWindowControl,
    projection: &TimelineRenderProjection,
    node_id: NodeId,
    parent_id: Option<NodeId>,
    level: StoryLevel,
    name: String,
    start_ms: u64,
    end_ms: u64,
    beat_type: Option<BeatType>,
) -> Result<(), TimelineRendererError> {
    if let Some(parent_id) = parent_id
        && !projection
            .clips
            .iter()
            .any(|clip| clip.node_id == parent_id)
    {
        return Err(TimelineRendererError::UnknownNode { node_id: parent_id });
    }
    if start_ms >= end_ms || end_ms > projection.total_duration_ms {
        return Err(TimelineRendererError::InvalidNodeRange {
            start_ms,
            end_ms,
            duration_ms: projection.total_duration_ms,
        });
    }

    control.enqueue_command(TimelineRendererCommand::CreateNode {
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
