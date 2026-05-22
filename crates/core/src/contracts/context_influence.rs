use serde::{Deserialize, Serialize};
use uuid::Uuid;

use crate::timeline::node::{NodeId, StoryLevel};

use super::{BibleGraphEdgeId, BibleGraphNodeId};

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub struct ContextEvaluationId(pub Uuid);

impl ContextEvaluationId {
    pub fn new() -> Self {
        Self(Uuid::new_v4())
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub struct ContextInfluenceId(pub Uuid);

impl ContextInfluenceId {
    pub fn new() -> Self {
        Self(Uuid::new_v4())
    }
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct RecordContextEvaluationCommand {
    pub evaluation: ContextEvaluation,
    #[serde(default)]
    pub influences: Vec<ContextInfluenceRecord>,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct ContextEvaluation {
    pub id: ContextEvaluationId,
    pub target_node_id: NodeId,
    pub task_kind: ContextEvaluationTaskKind,
    pub summary: String,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub distilled_context: Option<String>,
    #[serde(default)]
    pub created_at_ms: u64,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum ContextEvaluationTaskKind {
    GenerateTimelineContext,
    GenerateScript,
    InspectContext,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct ContextStackProjection {
    pub target_node_id: NodeId,
    #[serde(default)]
    pub layers: Vec<ContextStackLayer>,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct ContextStackLayer {
    pub node_id: NodeId,
    pub level: StoryLevel,
    pub label: String,
    pub role: ContextLayerRole,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub distilled_context: Option<String>,
    #[serde(default)]
    pub sort_order: u32,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum ContextLayerRole {
    Target,
    Inherited,
    Sibling,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct ContextInfluenceProjection {
    pub target_node_id: NodeId,
    pub evaluation_id: ContextEvaluationId,
    pub task_kind: ContextEvaluationTaskKind,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub distilled_context: Option<String>,
    #[serde(default)]
    pub records: Vec<ContextInfluenceRecord>,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct ContextInfluenceProjectionRequest {
    pub target_node_id: NodeId,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct ContextInfluenceRecord {
    pub id: ContextInfluenceId,
    pub evaluation_id: ContextEvaluationId,
    pub timeline_node_id: NodeId,
    pub source_layer: StoryLevel,
    pub influence_kind: ContextInfluenceKind,
    pub confidence: f32,
    pub reason: String,
    pub provenance: ContextInfluenceProvenance,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub bible_node_id: Option<BibleGraphNodeId>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub bible_edge_id: Option<BibleGraphEdgeId>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub introduced_by_node_id: Option<NodeId>,
    #[serde(default)]
    pub sort_order: u32,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum ContextInfluenceKind {
    Direct,
    Inherited,
    Candidate,
    Ignored,
    Rejected,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum ContextInfluenceProvenance {
    UserSelected,
    AiSelected,
    ParentContext,
    GraphTraversal,
    Proposal,
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::contracts::{BibleGraphEdgeId, BibleGraphNodeId};

    #[test]
    fn context_influence_projection_round_trips() {
        let target_node_id = NodeId::new();
        let evaluation_id = ContextEvaluationId::new();
        let record = ContextInfluenceRecord {
            id: ContextInfluenceId::new(),
            evaluation_id,
            timeline_node_id: target_node_id,
            source_layer: StoryLevel::Scene,
            influence_kind: ContextInfluenceKind::Direct,
            confidence: 0.82,
            reason: "scene is located at the harbor".to_string(),
            provenance: ContextInfluenceProvenance::AiSelected,
            bible_node_id: Some(BibleGraphNodeId::new("node.place.harbor").unwrap()),
            bible_edge_id: Some(BibleGraphEdgeId::new("edge.scene.harbor").unwrap()),
            introduced_by_node_id: Some(target_node_id),
            sort_order: 1,
        };
        let projection = ContextInfluenceProjection {
            target_node_id,
            evaluation_id,
            task_kind: ContextEvaluationTaskKind::GenerateTimelineContext,
            distilled_context: Some("Harbor weather controls the scene tone.".to_string()),
            records: vec![record.clone()],
        };

        let json = serde_json::to_string(&projection).unwrap();
        let decoded: ContextInfluenceProjection = serde_json::from_str(&json).unwrap();

        assert_eq!(decoded, projection);
        assert_eq!(decoded.records[0].bible_node_id, record.bible_node_id);
    }

    #[test]
    fn context_stack_projection_rejects_unknown_fields() {
        let json = format!(
            r#"{{
                "target_node_id":"{}",
                "layers":[],
                "unexpected":true
            }}"#,
            NodeId::new().0
        );

        let error = serde_json::from_str::<ContextStackProjection>(&json).unwrap_err();

        assert!(error.to_string().contains("unknown field"));
    }
}
