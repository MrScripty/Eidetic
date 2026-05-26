use serde::{Deserialize, Serialize};

use crate::timeline::node::NodeId;

use super::{
    AgentRunId, AgentToolCallId, BibleGraphEdgeId, BibleGraphEdgeKind, BibleGraphFieldId,
    BibleGraphFieldKey, BibleGraphNodeId, BibleGraphPartKey, BibleGraphSchemaKey, FieldValue,
    SemanticProposalStatus,
};

#[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[serde(try_from = "String", into = "String")]
pub struct GraphProposalId(String);

impl GraphProposalId {
    pub fn new(value: impl Into<String>) -> Result<Self, GraphProposalContractError> {
        let value = value.into();
        if value.trim().is_empty() {
            return Err(GraphProposalContractError::EmptyIdentifier(
                "GraphProposalId",
            ));
        }
        Ok(Self(value))
    }

    pub fn as_str(&self) -> &str {
        &self.0
    }
}

impl TryFrom<String> for GraphProposalId {
    type Error = GraphProposalContractError;

    fn try_from(value: String) -> Result<Self, Self::Error> {
        Self::new(value)
    }
}

impl From<GraphProposalId> for String {
    fn from(value: GraphProposalId) -> Self {
        value.0
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum GraphProposalAction {
    CreateBibleNode,
    SetBibleField,
    CreateBibleEdge,
    LinkTimelineContext,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case", tag = "kind")]
pub enum GraphProposalTarget {
    BibleNode {
        node_id: BibleGraphNodeId,
        parent_id: BibleGraphNodeId,
        schema_key: BibleGraphSchemaKey,
        title: String,
    },
    BibleField {
        node_id: BibleGraphNodeId,
        part_key: BibleGraphPartKey,
        field_key: BibleGraphFieldKey,
        #[serde(default, skip_serializing_if = "Option::is_none")]
        field_id: Option<BibleGraphFieldId>,
    },
    BibleEdge {
        edge_id: BibleGraphEdgeId,
        from_node_id: BibleGraphNodeId,
        to_node_id: BibleGraphNodeId,
        edge_kind: BibleGraphEdgeKind,
        label: String,
    },
    TimelineContextLink {
        timeline_node_id: NodeId,
        bible_node_id: BibleGraphNodeId,
    },
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct GraphProposal {
    pub id: GraphProposalId,
    pub action: GraphProposalAction,
    pub target: GraphProposalTarget,
    pub status: SemanticProposalStatus,
    pub summary: String,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub proposed_value: Option<FieldValue>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub rationale: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub source_agent_run_id: Option<AgentRunId>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub source_tool_call_id: Option<AgentToolCallId>,
    pub created_at_ms: u64,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct CreateGraphProposalCommand {
    pub proposal_id: GraphProposalId,
    pub action: GraphProposalAction,
    pub target: GraphProposalTarget,
    pub summary: String,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub proposed_value: Option<FieldValue>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub rationale: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub source_agent_run_id: Option<AgentRunId>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub source_tool_call_id: Option<AgentToolCallId>,
}

impl CreateGraphProposalCommand {
    pub fn into_proposal(self, created_at_ms: u64) -> GraphProposal {
        GraphProposal {
            id: self.proposal_id,
            action: self.action,
            target: self.target,
            status: SemanticProposalStatus::Pending,
            summary: self.summary,
            proposed_value: self.proposed_value,
            rationale: self.rationale,
            source_agent_run_id: self.source_agent_run_id,
            source_tool_call_id: self.source_tool_call_id,
            created_at_ms,
        }
    }
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct GraphProposalListProjection {
    #[serde(default)]
    pub proposals: Vec<GraphProposal>,
}

#[derive(Debug, Clone, PartialEq, Eq, thiserror::Error)]
pub enum GraphProposalContractError {
    #[error("empty identifier for {0}")]
    EmptyIdentifier(&'static str),
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn graph_proposal_id_rejects_empty_values() {
        assert!(GraphProposalId::new(" ").is_err());
    }

    #[test]
    fn create_graph_proposal_command_builds_pending_node_proposal() {
        let command = CreateGraphProposalCommand {
            proposal_id: GraphProposalId::new("proposal.graph.ada").unwrap(),
            action: GraphProposalAction::CreateBibleNode,
            target: GraphProposalTarget::BibleNode {
                node_id: BibleGraphNodeId::new("node.character.ada").unwrap(),
                parent_id: BibleGraphNodeId::new("canonical.characters").unwrap(),
                schema_key: BibleGraphSchemaKey::new("canonical.character").unwrap(),
                title: "Ada".to_string(),
            },
            summary: "Create Ada as a premise character".to_string(),
            proposed_value: None,
            rationale: Some("Premise mentions Ada.".to_string()),
            source_agent_run_id: Some(AgentRunId::new()),
            source_tool_call_id: Some(AgentToolCallId::new()),
        };

        let proposal = command.into_proposal(42);

        assert_eq!(proposal.id.as_str(), "proposal.graph.ada");
        assert_eq!(proposal.status, SemanticProposalStatus::Pending);
        assert_eq!(proposal.created_at_ms, 42);
    }

    #[test]
    fn graph_proposal_target_round_trips_context_link() {
        let target = GraphProposalTarget::TimelineContextLink {
            timeline_node_id: NodeId::new(),
            bible_node_id: BibleGraphNodeId::new("node.character.ada").unwrap(),
        };

        let encoded = serde_json::to_string(&target).unwrap();
        let decoded: GraphProposalTarget = serde_json::from_str(&encoded).unwrap();

        assert_eq!(decoded, target);
    }
}
