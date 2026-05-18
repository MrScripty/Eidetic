use serde::{Deserialize, Serialize};

use crate::timeline::node::NodeId;

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
