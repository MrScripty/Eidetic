use serde::{Deserialize, Serialize};

use crate::story::arc::ArcId;
use crate::timeline::Timeline;
use crate::timeline::node::{BeatType, ContentStatus, NodeId, StoryLevel};
use crate::timeline::relationship::{RelationshipId, RelationshipType};
use crate::timeline::structure::SegmentType;
use crate::timeline::timing::TimeRange;
use crate::timeline::track::TrackId;

const TIMELINE_RENDER_GAP_THRESHOLD_MS: u64 = 30_000;

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct TimelineRenderProjection {
    pub total_duration_ms: u64,
    #[serde(default)]
    pub structure_segments: Vec<TimelineRenderStructureSegment>,
    #[serde(default)]
    pub tracks: Vec<TimelineRenderTrack>,
    #[serde(default)]
    pub clips: Vec<TimelineRenderClip>,
    #[serde(default)]
    pub relationships: Vec<TimelineRenderRelationship>,
    #[serde(default)]
    pub gaps: Vec<TimelineRenderGap>,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct TimelineRenderStructureSegment {
    pub segment_type: SegmentType,
    pub time_range: TimeRange,
    pub label: String,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct TimelineRenderTrack {
    pub track_id: TrackId,
    pub level: StoryLevel,
    pub label: String,
    pub sort_order: u32,
    #[serde(default)]
    pub collapsed: bool,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct TimelineRenderClip {
    pub node_id: NodeId,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub parent_id: Option<NodeId>,
    pub track_id: TrackId,
    pub level: StoryLevel,
    pub name: String,
    pub start_ms: u64,
    pub end_ms: u64,
    pub sort_order: u32,
    #[serde(default)]
    pub locked: bool,
    pub content_status: ContentStatus,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub beat_type: Option<BeatType>,
    #[serde(default)]
    pub arc_ids: Vec<ArcId>,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct TimelineRenderRelationship {
    pub relationship_id: RelationshipId,
    pub from_node_id: NodeId,
    pub to_node_id: NodeId,
    pub relationship_type: RelationshipType,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct TimelineRenderGap {
    pub level: StoryLevel,
    pub time_range: TimeRange,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub preceding_node_id: Option<NodeId>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub following_node_id: Option<NodeId>,
}

impl TimelineRenderProjection {
    pub fn from_timeline(timeline: &Timeline) -> Self {
        let structure_segments = timeline
            .structure
            .segments
            .iter()
            .map(|segment| TimelineRenderStructureSegment {
                segment_type: segment.segment_type,
                time_range: segment.time_range,
                label: segment.label.clone(),
            })
            .collect();

        let mut tracks: Vec<TimelineRenderTrack> = timeline
            .tracks
            .iter()
            .map(|track| TimelineRenderTrack {
                track_id: track.id,
                level: track.level,
                label: track.label.clone(),
                sort_order: track.sort_order,
                collapsed: track.collapsed,
            })
            .collect();
        tracks.sort_by_key(|track| track.sort_order);

        let mut clips: Vec<TimelineRenderClip> = timeline
            .nodes
            .iter()
            .map(|node| TimelineRenderClip {
                node_id: node.id,
                parent_id: node.parent_id,
                track_id: timeline
                    .track_for_level(node.level)
                    .map(|track| track.id)
                    .expect("timeline nodes must have a track for their story level"),
                level: node.level,
                name: node.name.clone(),
                start_ms: node.time_range.start_ms,
                end_ms: node.time_range.end_ms,
                sort_order: node.sort_order,
                locked: node.locked,
                content_status: node.content.status,
                beat_type: node.beat_type.clone(),
                arc_ids: timeline
                    .node_arcs
                    .iter()
                    .filter(|node_arc| node_arc.node_id == node.id)
                    .map(|node_arc| node_arc.arc_id)
                    .collect(),
            })
            .collect();
        clips.sort_by_key(|clip| (clip.start_ms, clip.sort_order, clip.level));

        let mut relationships: Vec<TimelineRenderRelationship> = timeline
            .relationships
            .iter()
            .map(|relationship| TimelineRenderRelationship {
                relationship_id: relationship.id,
                from_node_id: relationship.from_node,
                to_node_id: relationship.to_node,
                relationship_type: relationship.relationship_type.clone(),
            })
            .collect();
        relationships.sort_by_key(|relationship| relationship.relationship_id.0);

        let mut gaps = Vec::new();
        for track in &tracks {
            gaps.extend(
                timeline
                    .find_gaps(track.level, TIMELINE_RENDER_GAP_THRESHOLD_MS)
                    .into_iter()
                    .map(|gap| TimelineRenderGap {
                        level: gap.level,
                        time_range: gap.time_range,
                        preceding_node_id: gap.preceding_node_id,
                        following_node_id: gap.following_node_id,
                    }),
            );
        }

        Self {
            total_duration_ms: timeline.total_duration_ms,
            structure_segments,
            tracks,
            clips,
            relationships,
            gaps,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::story::arc::ArcId;
    use crate::timeline::node::{NodeArc, StoryNode};
    use crate::timeline::relationship::{Relationship, RelationshipType};
    use crate::timeline::structure::EpisodeStructure;
    use crate::timeline::timing::TimeRange;

    #[test]
    fn timeline_render_projection_maps_tracks_clips_arcs_and_relationships() {
        let mut timeline = Timeline::new(100_000, EpisodeStructure::standard_30_min());
        let mut scene = StoryNode::new(
            "Beach argument",
            StoryLevel::Scene,
            TimeRange::new(1_000, 4_000).unwrap(),
        );
        scene.content.status = ContentStatus::NotesOnly;
        let scene_id = scene.id;
        let arc_id = ArcId::new();
        timeline.nodes.push(scene);
        timeline.node_arcs.push(NodeArc {
            node_id: scene_id,
            arc_id,
        });
        timeline.relationships.push(Relationship::new(
            scene_id,
            scene_id,
            RelationshipType::Thematic,
        ));

        let projection = TimelineRenderProjection::from_timeline(&timeline);

        assert_eq!(projection.total_duration_ms, 100_000);
        assert_eq!(projection.structure_segments[0].label, "Cold Open");
        assert_eq!(projection.tracks[0].level, StoryLevel::Premise);
        assert_eq!(projection.clips.len(), 1);
        assert_eq!(projection.clips[0].node_id, scene_id);
        assert_eq!(projection.clips[0].content_status, ContentStatus::NotesOnly);
        assert_eq!(projection.clips[0].arc_ids, vec![arc_id]);
        assert_eq!(projection.relationships.len(), 1);
        assert_eq!(projection.relationships[0].from_node_id, scene_id);
        assert!(
            projection
                .gaps
                .iter()
                .any(|gap| gap.level == StoryLevel::Scene)
        );
    }
}
