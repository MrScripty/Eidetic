use eidetic_core::contracts::TimelineRenderProjection;
use eidetic_core::timeline::node::NodeId;

use crate::native_render::TimelineNativeWindowControl;
use crate::{
    TimelineRendererCommand, TimelineRendererError, TimelineViewport, TimelineViewportGeometry,
    TimelineViewportPoint, hit_test_projection_clip_at_point,
};

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum TimelineNativeResizeEdge {
    Start,
    End,
}

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

pub fn emit_timeline_native_selected_nudge_request(
    control: &TimelineNativeWindowControl,
    projection: &TimelineRenderProjection,
    delta_ms: i64,
) -> Result<Option<(NodeId, u64, u64)>, TimelineRendererError> {
    let Some(node_id) = projection.selected_node_id else {
        return Ok(None);
    };
    let Some(clip) = projection.clips.iter().find(|clip| clip.node_id == node_id) else {
        return Err(TimelineRendererError::UnknownNode { node_id });
    };
    let (start_ms, end_ms) = if delta_ms.is_negative() {
        let shift_ms = clip.start_ms.min(delta_ms.unsigned_abs());
        if shift_ms == 0 {
            return Ok(None);
        }
        (
            clip.start_ms.saturating_sub(shift_ms),
            clip.end_ms.saturating_sub(shift_ms),
        )
    } else {
        let shift_ms = projection
            .total_duration_ms
            .saturating_sub(clip.end_ms)
            .min(delta_ms as u64);
        if shift_ms == 0 {
            return Ok(None);
        }
        (
            clip.start_ms.saturating_add(shift_ms),
            clip.end_ms.saturating_add(shift_ms),
        )
    };

    emit_timeline_native_node_range_request(control, projection, node_id, start_ms, end_ms)?;
    Ok(Some((node_id, start_ms, end_ms)))
}

pub fn emit_timeline_native_selected_resize_request(
    control: &TimelineNativeWindowControl,
    projection: &TimelineRenderProjection,
    edge: TimelineNativeResizeEdge,
    delta_ms: i64,
) -> Result<Option<(NodeId, u64, u64)>, TimelineRendererError> {
    let Some(node_id) = projection.selected_node_id else {
        return Ok(None);
    };
    let Some(clip) = projection.clips.iter().find(|clip| clip.node_id == node_id) else {
        return Err(TimelineRendererError::UnknownNode { node_id });
    };

    let (start_ms, end_ms) = match edge {
        TimelineNativeResizeEdge::Start => {
            if clip.end_ms == 0 {
                return Err(TimelineRendererError::InvalidNodeRange {
                    start_ms: clip.start_ms,
                    end_ms: clip.end_ms,
                    duration_ms: projection.total_duration_ms,
                });
            }
            (
                offset_within_bounds(clip.start_ms, delta_ms, 0, clip.end_ms - 1),
                clip.end_ms,
            )
        }
        TimelineNativeResizeEdge::End => (
            clip.start_ms,
            offset_within_bounds(
                clip.end_ms,
                delta_ms,
                clip.start_ms.saturating_add(1),
                projection.total_duration_ms,
            ),
        ),
    };
    if start_ms == clip.start_ms && end_ms == clip.end_ms {
        return Ok(None);
    }

    emit_timeline_native_node_range_request(control, projection, node_id, start_ms, end_ms)?;
    Ok(Some((node_id, start_ms, end_ms)))
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

fn offset_within_bounds(value: u64, delta_ms: i64, min: u64, max: u64) -> u64 {
    if delta_ms.is_negative() {
        value.saturating_sub(delta_ms.unsigned_abs()).max(min)
    } else {
        value.saturating_add(delta_ms as u64).min(max)
    }
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

pub fn emit_timeline_native_create_child_from_parent_request(
    control: &TimelineNativeWindowControl,
    projection: &TimelineRenderProjection,
    node_id: NodeId,
    parent_id: NodeId,
) -> Result<(), TimelineRendererError> {
    if !projection
        .clips
        .iter()
        .any(|clip| clip.node_id == parent_id)
    {
        return Err(TimelineRendererError::UnknownNode { node_id: parent_id });
    }

    control.enqueue_command(TimelineRendererCommand::CreateChildFromParent { node_id, parent_id });
    Ok(())
}

pub fn emit_timeline_native_selected_create_child_from_parent_request(
    control: &TimelineNativeWindowControl,
    projection: &TimelineRenderProjection,
    node_id: NodeId,
) -> Result<Option<NodeId>, TimelineRendererError> {
    let Some(parent_id) = projection.selected_node_id else {
        return Ok(None);
    };
    emit_timeline_native_create_child_from_parent_request(control, projection, node_id, parent_id)?;
    Ok(Some(parent_id))
}

pub fn emit_timeline_native_playhead_request(
    control: &TimelineNativeWindowControl,
    projection: &TimelineRenderProjection,
    position_ms: u64,
) -> Result<(), TimelineRendererError> {
    if position_ms > projection.total_duration_ms {
        return Err(TimelineRendererError::InvalidPlayheadPosition {
            position_ms,
            duration_ms: projection.total_duration_ms,
        });
    }

    control.enqueue_command(TimelineRendererCommand::SetPlayhead { position_ms });
    Ok(())
}
