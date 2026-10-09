//! Explicit evidence inspection; no generation or world-update authority.
use serde::{Deserialize, Serialize};

use super::{
    AiBibleContextFieldRef, BibleGraphEdgeKind, BibleGraphFieldId, BibleGraphFieldKey,
    BibleGraphNodeId, BibleGraphPartKey, BibleGraphSchemaKey, BibleGraphSnapshotFieldId,
    BibleGraphSnapshotId, BibleRelationshipInput, ChangeEventId, FieldValue,
};

pub const BIBLE_RECALL_MAX_NEIGHBORS: u32 = 8;
pub const BIBLE_RECALL_MAX_EDGES: usize = 32;
pub const BIBLE_RECALL_MAX_BYTES: usize = 32 * 1024;

#[derive(Debug, Default, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum BibleRecallDirection {
    Incoming,
    Outgoing,
    #[default]
    Both,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct BibleRecallRequest {
    pub anchor_node_id: BibleGraphNodeId,
    #[serde(default)]
    pub story_time_ms: Option<u64>,
    #[serde(default)]
    pub direction: BibleRecallDirection,
    #[serde(default)]
    pub edge_kinds: Vec<BibleGraphEdgeKind>,
    #[serde(default = "default_limit")]
    pub neighbor_limit: u32,
}

fn default_limit() -> u32 {
    BIBLE_RECALL_MAX_NEIGHBORS
}

impl BibleRecallRequest {
    pub fn validate(&self) -> Result<(), &'static str> {
        if !(1..=BIBLE_RECALL_MAX_NEIGHBORS).contains(&self.neighbor_limit) {
            return Err("neighbor_limit must be between 1 and 8");
        }
        Ok(())
    }
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct BibleRecallProjection {
    pub request: BibleRecallRequest,
    pub nodes: Vec<BibleRecallNode>,
    pub paths: Vec<BibleRecallPath>,
    pub omitted_neighbors: u64,
    pub omitted_edges: u64,
    /// Edges are stored associations, with no established fictional-time validity.
    pub relationships_untimed: bool,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct BibleRecallNode {
    pub node_id: BibleGraphNodeId,
    pub name: String,
    pub schema_key: BibleGraphSchemaKey,
    /// Missing history stays explicitly unknown.
    pub name_revision_event_id: Option<ChangeEventId>,
    pub fields: Vec<BibleRecallField>,
    pub unresolved_timed_fields: Vec<AiBibleContextFieldRef>,
    /// Whole records omitted by the evidence budget, never cut field values.
    pub omitted_fields: u64,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct BibleRecallField {
    pub part_key: BibleGraphPartKey,
    pub field_key: BibleGraphFieldKey,
    pub value: FieldValue,
    pub source: BibleRecallFieldSource,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(tag = "kind", rename_all = "snake_case")]
pub enum BibleRecallFieldSource {
    Baseline {
        field_id: BibleGraphFieldId,
        revision_event_id: ChangeEventId,
    },
    Snapshot {
        snapshot_id: BibleGraphSnapshotId,
        snapshot_field_id: BibleGraphSnapshotFieldId,
        snapshot_revision_event_id: ChangeEventId,
        field_revision_event_id: ChangeEventId,
        at_ms: u64,
        label: String,
    },
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct BibleRecallPath {
    pub neighbor_node_id: BibleGraphNodeId,
    pub relationship: BibleRelationshipInput,
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn request_defaults_are_explicit_bounded_and_reject_unknown_inputs() {
        let request: BibleRecallRequest =
            serde_json::from_str(r#"{"anchor_node_id":"Mara"}"#).unwrap();
        assert_eq!(request.neighbor_limit, 8);
        assert_eq!(request.direction, BibleRecallDirection::Both);
        assert_eq!(request.story_time_ms, None);
        assert!(request.validate().is_ok());
        assert!(
            serde_json::from_str::<BibleRecallRequest>(
                r#"{"anchor_node_id":"Mara","consume":true}"#
            )
            .is_err()
        );
    }
    #[test]
    fn agent_read_kind_respects_the_smaller_of_recall_and_workflow_limits() {
        use crate::contracts::{AgentToolArguments, AgentToolBudget, AgentToolKind};
        let mut query: BibleRecallRequest =
            serde_json::from_str(r#"{"anchor_node_id":"Mara"}"#).unwrap();
        let budget = AgentToolBudget {
            max_graph_read_limit: 2,
            ..AgentToolBudget::default()
        };
        assert!(
            AgentToolArguments::ReadBibleRecall {
                query: query.clone()
            }
            .validate(&budget)
            .is_err()
        );
        query.neighbor_limit = 2;
        let read = AgentToolArguments::ReadBibleRecall {
            query: query.clone(),
        };
        assert_eq!(read.kind(), AgentToolKind::GraphRead);
        assert!(read.validate(&budget).is_ok());
        query.neighbor_limit = 9;
        assert!(
            AgentToolArguments::ReadBibleRecall { query }
                .validate(&AgentToolBudget::default())
                .is_err()
        );
    }
}
