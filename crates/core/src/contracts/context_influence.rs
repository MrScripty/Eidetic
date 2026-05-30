use serde::{Deserialize, Serialize};
use uuid::Uuid;

use crate::timeline::node::{NodeId, StoryLevel, StoryNode};

use super::{BibleGraphEdgeId, BibleGraphNodeId};

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub struct ContextEvaluationId(pub Uuid);

impl ContextEvaluationId {
    pub fn new() -> Self {
        Self(Uuid::new_v4())
    }
}

impl Default for ContextEvaluationId {
    fn default() -> Self {
        Self::new()
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub struct ContextInfluenceId(pub Uuid);

impl ContextInfluenceId {
    pub fn new() -> Self {
        Self(Uuid::new_v4())
    }
}

impl Default for ContextInfluenceId {
    fn default() -> Self {
        Self::new()
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

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct ContextStackProjectionRequest {
    pub target_node_id: NodeId,
}

impl ContextStackProjection {
    pub fn from_nodes(nodes: &[StoryNode], target_node_id: NodeId) -> Option<Self> {
        let mut layers = Vec::new();
        let mut current_id = target_node_id;

        for _ in 0..nodes.len() {
            let node = nodes.iter().find(|node| node.id == current_id)?;
            layers.push(ContextStackLayer {
                node_id: node.id,
                level: node.level,
                label: node.name.clone(),
                role: if node.id == target_node_id {
                    ContextLayerRole::Target
                } else {
                    ContextLayerRole::Inherited
                },
                distilled_context: node.content.scene_recap.clone(),
                sort_order: node.sort_order,
            });
            let Some(parent_id) = node.parent_id else {
                break;
            };
            current_id = parent_id;
        }

        if layers.is_empty() {
            return None;
        }

        layers.reverse();
        Some(Self {
            target_node_id,
            layers,
        })
    }
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

    #[test]
    fn context_stack_projection_follows_timeline_ancestors() {
        let premise = StoryNode::new(
            "Premise",
            StoryLevel::Premise,
            crate::timeline::timing::TimeRange::new(0, 10).unwrap(),
        );
        let mut act = StoryNode::new(
            "Act",
            StoryLevel::Act,
            crate::timeline::timing::TimeRange::new(0, 10).unwrap(),
        );
        act.parent_id = Some(premise.id);
        let mut scene = StoryNode::new(
            "Scene",
            StoryLevel::Scene,
            crate::timeline::timing::TimeRange::new(0, 10).unwrap(),
        );
        scene.parent_id = Some(act.id);
        scene.content.scene_recap = Some("Rain at the harbor.".to_string());
        let scene_id = scene.id;

        let projection =
            ContextStackProjection::from_nodes(&[scene, act, premise], scene_id).unwrap();

        assert_eq!(projection.layers.len(), 3);
        assert_eq!(projection.layers[0].level, StoryLevel::Premise);
        assert_eq!(projection.layers[2].role, ContextLayerRole::Target);
        assert_eq!(
            projection.layers[2].distilled_context.as_deref(),
            Some("Rain at the harbor.")
        );
    }
}
