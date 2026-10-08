use serde::{Deserialize, Serialize};

use crate::ai::backend::ChildPlanId;
use crate::story::arc::ArcId;

/// Exact selected-clip membership. None is a known history-free baseline.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct TimelineArcMembershipInput {
    pub node_id: NodeId,
    pub arc_ids: Vec<ArcId>,
    pub revision_event_id: Option<super::ChangeEventId>,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct SetTimelineNodeArcsCommand {
    pub node_id: NodeId,
    pub arc_ids: Vec<ArcId>,
    pub expected: TimelineArcMembershipInput,
}

use crate::timeline::node::{BeatType, NodeId, StoryLevel};
use crate::timeline::relationship::{RelationshipId, RelationshipType};

/// Exact canonical title and its owned field clock; None is a known baseline.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct TimelineNodeNameRead {
    pub name: String,
    pub revision_event_id: Option<super::ChangeEventId>,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct SetTimelineNodeNameCommand {
    pub node_id: NodeId,
    pub name: String,
    pub expected: TimelineNodeNameRead,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct SetTimelineNodeRangeCommand {
    pub node_id: NodeId,
    pub start_ms: u64,
    pub end_ms: u64,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub expected: Option<TimelineNodeRangeRead>,
}

/// Canonical placement read; a missing event is a known history-free node.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct TimelineNodeRangeRead {
    pub start_ms: u64,
    pub end_ms: u64,
    pub node_revision_event_id: Option<super::ChangeEventId>,
}

#[cfg(test)]
mod placement_wire_tests {
    use super::*;

    #[test]
    fn legacy_range_payload_keeps_its_exact_signature_and_known_empty_history_round_trips() {
        let legacy = format!(
            "{{\"node_id\":\"{}\",\"start_ms\":1000,\"end_ms\":2000}}",
            uuid::Uuid::nil()
        );
        let mut command: SetTimelineNodeRangeCommand = serde_json::from_str(&legacy).unwrap();
        assert_eq!(command.expected, None);
        assert_eq!(serde_json::to_string(&command).unwrap(), legacy);
        command.expected = Some(TimelineNodeRangeRead {
            start_ms: 1000,
            end_ms: 2000,
            node_revision_event_id: None,
        });
        let wire = serde_json::to_string(&command).unwrap();
        assert!(wire.contains("\"node_revision_event_id\":null"));
        assert_eq!(
            serde_json::from_str::<SetTimelineNodeRangeCommand>(&wire).unwrap(),
            command
        );
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct SplitTimelineNodeCommand {
    pub node_id: NodeId,
    pub at_ms: u64,
    pub left_node_id: NodeId,
    pub right_node_id: NodeId,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct DeleteTimelineNodeCommand {
    pub node_id: NodeId,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct SetTimelineNodeLockCommand {
    pub node_id: NodeId,
    pub locked: bool,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct SetTimelineNodeNotesCommand {
    pub node_id: NodeId,
    pub notes: String,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub expected: Option<super::TimelineNotesInput>,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct CreateTimelineNodeCommand {
    pub node_id: NodeId,
    pub parent_id: Option<NodeId>,
    pub level: StoryLevel,
    pub name: String,
    pub start_ms: u64,
    pub end_ms: u64,
    pub beat_type: Option<BeatType>,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct CreateTimelineChildFromParentCommand {
    pub node_id: NodeId,
    pub parent_id: NodeId,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct ApplyTimelineChildrenCommand {
    pub parent_id: NodeId,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub child_plan_id: Option<ChildPlanId>,
    pub children: Vec<ApplyTimelineChildCommand>,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct ApplyTimelineChildCommand {
    pub node_id: NodeId,
    pub name: String,
    pub outline: String,
    pub weight: f32,
    pub beat_type: Option<BeatType>,
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub characters: Vec<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub location: Option<String>,
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub props: Vec<String>,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct CreateTimelineRelationshipCommand {
    pub relationship_id: RelationshipId,
    pub from_node_id: NodeId,
    pub to_node_id: NodeId,
    pub relationship_type: RelationshipType,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct DeleteTimelineRelationshipCommand {
    pub relationship_id: RelationshipId,
}
