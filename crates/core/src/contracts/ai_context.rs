use serde::{Deserialize, Serialize};

use crate::timeline::node::NodeId;

use super::{
    BibleGraphEdgeId, BibleGraphEdgeKind, BibleGraphFieldId, BibleGraphFieldKey, BibleGraphNodeId,
    BibleGraphPartKey, BibleGraphSchemaKey, ChangeEventId, FieldValue,
};

/// Exact untimed field evidence consumed by generation. Timed overrides are
/// resolved separately and must never be rebound to their baseline field.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct BibleFieldInput {
    pub node_id: BibleGraphNodeId,
    pub part_key: BibleGraphPartKey,
    pub field_key: BibleGraphFieldKey,
    pub field_id: BibleGraphFieldId,
    pub revision_event_id: ChangeEventId,
    pub value: FieldValue,
}

/// Exact untimed relationship supplied by the Bible resolver. Repeated incoming
/// and outgoing appearances refer to one consumed edge identity and revision.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct BibleRelationshipInput {
    pub edge: AiBibleContextEdge,
    pub revision_event_id: ChangeEventId,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct AiBibleContextProjection {
    pub target_node_id: NodeId,
    /// Explicit fictional time; never inferred from narrative placement or edit time.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub story_time_ms: Option<u64>,
    #[serde(default)]
    pub nodes: Vec<AiBibleContextNode>,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct AiBibleContextNode {
    pub node_id: BibleGraphNodeId,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub parent_id: Option<BibleGraphNodeId>,
    pub schema_key: BibleGraphSchemaKey,
    pub name: String,
    #[serde(default)]
    pub fields: Vec<AiBibleContextField>,
    #[serde(default)]
    pub snapshots: Vec<AiBibleContextSnapshot>,
    /// Timed fields omitted because their effective value is not established.
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub unresolved_timed_fields: Vec<AiBibleContextFieldRef>,
    #[serde(default)]
    pub incoming_edges: Vec<AiBibleContextEdge>,
    #[serde(default)]
    pub outgoing_edges: Vec<AiBibleContextEdge>,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct AiBibleContextFieldRef {
    pub part_key: BibleGraphPartKey,
    pub field_key: BibleGraphFieldKey,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct AiBibleContextField {
    pub part_key: BibleGraphPartKey,
    pub part_name: String,
    pub field_key: BibleGraphFieldKey,
    pub value: FieldValue,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct AiBibleContextSnapshot {
    pub label: String,
    pub at_ms: u64,
    #[serde(default)]
    pub fields: Vec<AiBibleContextField>,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct AiBibleContextEdge {
    pub edge_id: BibleGraphEdgeId,
    pub from_node_id: BibleGraphNodeId,
    pub to_node_id: BibleGraphNodeId,
    pub edge_kind: BibleGraphEdgeKind,
    pub label: String,
    pub directed: bool,
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn ai_bible_context_projection_round_trips() {
        let target_node_id = NodeId::new();
        let projection = AiBibleContextProjection {
            target_node_id,
            story_time_ms: None,
            nodes: vec![AiBibleContextNode {
                node_id: BibleGraphNodeId::new("node.character.ada").unwrap(),
                parent_id: None,
                schema_key: BibleGraphSchemaKey::new("character").unwrap(),
                name: "Ada".to_string(),
                fields: vec![AiBibleContextField {
                    part_key: BibleGraphPartKey::new("profile").unwrap(),
                    part_name: "Profile".to_string(),
                    field_key: BibleGraphFieldKey::new("tagline").unwrap(),
                    value: FieldValue::Text("Reluctant detective".to_string()),
                }],
                snapshots: Vec::new(),
                unresolved_timed_fields: Vec::new(),
                incoming_edges: Vec::new(),
                outgoing_edges: Vec::new(),
            }],
        };

        let encoded = serde_json::to_string(&projection).unwrap();
        let decoded: AiBibleContextProjection = serde_json::from_str(&encoded).unwrap();

        assert_eq!(decoded, projection);
    }
}
