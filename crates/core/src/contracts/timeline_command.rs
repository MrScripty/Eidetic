use serde::{Deserialize, Serialize};

use crate::timeline::node::{BeatType, NodeId, StoryLevel};

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
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct DeleteTimelineNodeCommand {
    pub node_id: NodeId,
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
    pub children: Vec<ApplyTimelineChildCommand>,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct ApplyTimelineChildCommand {
    pub node_id: NodeId,
    pub name: String,
    pub outline: String,
    pub weight: f32,
    pub beat_type: Option<BeatType>,
}
