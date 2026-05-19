use serde::{Deserialize, Serialize};

use crate::ai::backend::ChildPlanId;
use crate::timeline::node::{BeatType, NodeId, StoryLevel};
use crate::timeline::relationship::{RelationshipId, RelationshipType};

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct SetTimelineNodeRangeCommand {
    pub node_id: NodeId,
    pub start_ms: u64,
    pub end_ms: u64,
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
