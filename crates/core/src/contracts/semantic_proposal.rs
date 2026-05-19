use serde::{Deserialize, Serialize};

use crate::timeline::node::NodeId;

use super::{BibleGraphNodeId, BibleGraphSchemaKey};

#[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[serde(try_from = "String", into = "String")]
pub struct SemanticProposalId(String);

impl SemanticProposalId {
    pub fn new(value: impl Into<String>) -> Result<Self, SemanticProposalContractError> {
        let value = value.into();
        if value.trim().is_empty() {
            return Err(SemanticProposalContractError::EmptyIdentifier(
                "SemanticProposalId",
            ));
        }
        Ok(Self(value))
    }

    pub fn as_str(&self) -> &str {
        &self.0
    }
}

impl TryFrom<String> for SemanticProposalId {
    type Error = SemanticProposalContractError;

    fn try_from(value: String) -> Result<Self, Self::Error> {
        Self::new(value)
    }
}

impl From<SemanticProposalId> for String {
    fn from(value: SemanticProposalId) -> Self {
        value.0
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum BibleReferenceKind {
    Character,
    Location,
    Prop,
}

impl BibleReferenceKind {
    pub fn proposed_schema_key(&self) -> BibleGraphSchemaKey {
        BibleGraphSchemaKey::new(match self {
            Self::Character => "character",
            Self::Location => "location",
            Self::Prop => "prop",
        })
        .expect("built-in bible reference schema keys are non-empty")
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum SemanticProposalStatus {
    Pending,
    Accepted,
    Rejected,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct BibleReferenceProposal {
    pub id: SemanticProposalId,
    pub source_node_id: NodeId,
    pub child_name: String,
    pub reference_kind: BibleReferenceKind,
    pub reference_text: String,
    pub proposed_schema_key: BibleGraphSchemaKey,
    pub status: SemanticProposalStatus,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub rationale: Option<String>,
    pub created_at_ms: u64,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct CreateBibleReferenceProposalCommand {
    pub proposal_id: SemanticProposalId,
    pub source_node_id: NodeId,
    pub child_name: String,
    pub reference_kind: BibleReferenceKind,
    pub reference_text: String,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub rationale: Option<String>,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct RejectBibleReferenceProposalCommand {
    pub proposal_id: SemanticProposalId,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub reason: Option<String>,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct AcceptBibleReferenceProposalCommand {
    pub proposal_id: SemanticProposalId,
    pub node_id: BibleGraphNodeId,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub parent_id: Option<BibleGraphNodeId>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub name: Option<String>,
    #[serde(default)]
    pub sort_order: u32,
}

impl CreateBibleReferenceProposalCommand {
    pub fn into_proposal(self, created_at_ms: u64) -> BibleReferenceProposal {
        let proposed_schema_key = self.reference_kind.proposed_schema_key();
        BibleReferenceProposal {
            id: self.proposal_id,
            source_node_id: self.source_node_id,
            child_name: self.child_name,
            reference_kind: self.reference_kind,
            reference_text: self.reference_text,
            proposed_schema_key,
            status: SemanticProposalStatus::Pending,
            rationale: self.rationale,
            created_at_ms,
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct BibleReferenceProposalListProjection {
    pub proposals: Vec<BibleReferenceProposal>,
}

#[derive(Debug, Clone, PartialEq, Eq, thiserror::Error)]
pub enum SemanticProposalContractError {
    #[error("empty identifier for {0}")]
    EmptyIdentifier(&'static str),
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn semantic_proposal_id_rejects_empty_values() {
        assert!(SemanticProposalId::new(" ").is_err());
    }

    #[test]
    fn bible_reference_kind_maps_to_proposed_schema_key() {
        assert_eq!(
            BibleReferenceKind::Location.proposed_schema_key().as_str(),
            "location"
        );
    }

    #[test]
    fn create_command_builds_pending_proposal() {
        let command = CreateBibleReferenceProposalCommand {
            proposal_id: SemanticProposalId::new("proposal.scene.location").unwrap(),
            source_node_id: crate::timeline::node::NodeId::new(),
            child_name: "Scene 3".to_string(),
            reference_kind: BibleReferenceKind::Location,
            reference_text: "Storm Harbor".to_string(),
            rationale: Some("Referenced by the generated scene outline".to_string()),
        };

        let proposal = command.into_proposal(42);

        assert_eq!(proposal.id.as_str(), "proposal.scene.location");
        assert_eq!(proposal.proposed_schema_key.as_str(), "location");
        assert_eq!(proposal.status, SemanticProposalStatus::Pending);
        assert_eq!(proposal.created_at_ms, 42);
    }

    #[test]
    fn reject_command_round_trips() {
        let command = RejectBibleReferenceProposalCommand {
            proposal_id: SemanticProposalId::new("proposal.scene.location").unwrap(),
            reason: Some("Not a durable location".to_string()),
        };

        let json = serde_json::to_string(&command).unwrap();
        let decoded: RejectBibleReferenceProposalCommand = serde_json::from_str(&json).unwrap();

        assert_eq!(decoded, command);
    }

    #[test]
    fn accept_command_round_trips() {
        let command = AcceptBibleReferenceProposalCommand {
            proposal_id: SemanticProposalId::new("proposal.scene.location").unwrap(),
            node_id: BibleGraphNodeId::new("node.location.harbor").unwrap(),
            parent_id: Some(BibleGraphNodeId::new("canonical.places").unwrap()),
            name: Some("Storm Harbor".to_string()),
            sort_order: 7,
        };

        let json = serde_json::to_string(&command).unwrap();
        let decoded: AcceptBibleReferenceProposalCommand = serde_json::from_str(&json).unwrap();

        assert_eq!(decoded, command);
    }
}
