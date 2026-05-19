use serde::{Deserialize, Serialize};

use crate::timeline::node::NodeId;

use super::{
    BibleGraphEdgeId, BibleGraphEdgeKind, BibleGraphFieldKey, BibleGraphNodeId, BibleGraphPartKey,
    BibleGraphSchemaKey, FieldValue,
};

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct AiBibleContextProjection {
    pub target_node_id: NodeId,
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
    #[serde(default)]
    pub incoming_edges: Vec<AiBibleContextEdge>,
    #[serde(default)]
    pub outgoing_edges: Vec<AiBibleContextEdge>,
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
                incoming_edges: Vec::new(),
                outgoing_edges: Vec::new(),
            }],
        };

        let encoded = serde_json::to_string(&projection).unwrap();
        let decoded: AiBibleContextProjection = serde_json::from_str(&encoded).unwrap();

        assert_eq!(decoded, projection);
    }
}
