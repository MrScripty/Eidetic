use eidetic_core::contracts::TimelineRenderProjection;
use eidetic_core::timeline::node::NodeId;
use eidetic_core::timeline::relationship::{RelationshipId, RelationshipType};

use crate::TimelineRendererError;

#[derive(Debug, Clone, PartialEq)]
pub struct TimelineRelationshipCurve {
    pub relationship_id: RelationshipId,
    pub from_node_id: NodeId,
    pub to_node_id: NodeId,
    pub relationship_type: RelationshipType,
    pub start: TimelineCurvePoint,
    pub control_a: TimelineCurvePoint,
    pub control_b: TimelineCurvePoint,
    pub end: TimelineCurvePoint,
}

#[derive(Debug, Clone, Copy, PartialEq)]
pub struct TimelineCurvePoint {
    pub x_ms: f32,
    pub y_track: f32,
}

pub fn relationship_curves(
    projection: &TimelineRenderProjection,
) -> Result<Vec<TimelineRelationshipCurve>, TimelineRendererError> {
    projection
        .relationships
        .iter()
        .map(|relationship| {
            let from = clip_point(projection, relationship.from_node_id)?;
            let to = clip_point(projection, relationship.to_node_id)?;
            let center_x = (from.x_ms + to.x_ms) / 2.0;
            Ok(TimelineRelationshipCurve {
                relationship_id: relationship.relationship_id,
                from_node_id: relationship.from_node_id,
                to_node_id: relationship.to_node_id,
                relationship_type: relationship.relationship_type.clone(),
                start: from,
                control_a: TimelineCurvePoint {
                    x_ms: center_x,
                    y_track: from.y_track,
                },
                control_b: TimelineCurvePoint {
                    x_ms: center_x,
                    y_track: to.y_track,
                },
                end: to,
            })
        })
        .collect()
}

fn clip_point(
    projection: &TimelineRenderProjection,
    node_id: NodeId,
) -> Result<TimelineCurvePoint, TimelineRendererError> {
    let Some(clip) = projection.clips.iter().find(|clip| clip.node_id == node_id) else {
        return Err(TimelineRendererError::UnknownRelationshipEndpoint { node_id });
    };
    let track_index = projection
        .tracks
        .iter()
        .position(|track| track.track_id == clip.track_id)
        .unwrap_or_default();
    Ok(TimelineCurvePoint {
        x_ms: ((clip.start_ms + clip.end_ms) / 2) as f32,
        y_track: track_index as f32,
    })
}
