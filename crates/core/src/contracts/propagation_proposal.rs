use serde::{Deserialize, Serialize};

use super::{
    BibleGraphFieldId, BibleGraphFieldKey, BibleGraphNodeId, BibleGraphPartKey,
    BibleGraphSnapshotFieldId, BibleGraphSnapshotId, ChangeEventId, FieldValue, ScriptBlockId,
    ScriptSegmentId, SemanticDependencyId, SemanticProposalStatus,
};

#[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[serde(try_from = "String", into = "String")]
pub struct PropagationProposalId(String);

impl PropagationProposalId {
    pub fn new(value: impl Into<String>) -> Result<Self, PropagationProposalContractError> {
        let value = value.into();
        if value.trim().is_empty() {
            return Err(PropagationProposalContractError::EmptyIdentifier(
                "PropagationProposalId",
            ));
        }
        Ok(Self(value))
    }

    pub fn as_str(&self) -> &str {
        &self.0
    }
}

impl TryFrom<String> for PropagationProposalId {
    type Error = PropagationProposalContractError;

    fn try_from(value: String) -> Result<Self, Self::Error> {
        Self::new(value)
    }
}

impl From<PropagationProposalId> for String {
    fn from(value: PropagationProposalId) -> Self {
        value.0
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case", tag = "kind")]
pub enum PropagationProposalTarget {
    BibleField {
        node_id: BibleGraphNodeId,
        part_key: BibleGraphPartKey,
        field_key: BibleGraphFieldKey,
        #[serde(default, skip_serializing_if = "Option::is_none")]
        field_id: Option<BibleGraphFieldId>,
    },
    BibleSnapshotField {
        node_id: BibleGraphNodeId,
        snapshot_id: BibleGraphSnapshotId,
        part_key: BibleGraphPartKey,
        field_key: BibleGraphFieldKey,
        field_id: BibleGraphSnapshotFieldId,
    },
    ScriptBlock {
        block_id: ScriptBlockId,
    },
    ScriptSegment {
        segment_id: ScriptSegmentId,
    },
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum PropagationProposalAction {
    SetBibleField,
    SetBibleSnapshotField,
    PatchScriptBlock,
    RegenerateScriptSegment,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct PropagationProposal {
    pub id: PropagationProposalId,
    pub action: PropagationProposalAction,
    pub target: PropagationProposalTarget,
    pub status: SemanticProposalStatus,
    pub summary: String,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub proposed_value: Option<FieldValue>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub proposed_text: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub source_dependency_id: Option<SemanticDependencyId>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub source_event_id: Option<ChangeEventId>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub rationale: Option<String>,
    pub created_at_ms: u64,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct CreatePropagationProposalCommand {
    pub proposal_id: PropagationProposalId,
    pub action: PropagationProposalAction,
    pub target: PropagationProposalTarget,
    pub summary: String,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub proposed_value: Option<FieldValue>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub proposed_text: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub source_dependency_id: Option<SemanticDependencyId>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub source_event_id: Option<ChangeEventId>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub rationale: Option<String>,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct UpdatePropagationProposalCommand {
    pub proposal_id: PropagationProposalId,
    pub action: PropagationProposalAction,
    pub target: PropagationProposalTarget,
    pub summary: String,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub proposed_value: Option<FieldValue>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub proposed_text: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub source_dependency_id: Option<SemanticDependencyId>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub source_event_id: Option<ChangeEventId>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub rationale: Option<String>,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct RejectPropagationProposalCommand {
    pub proposal_id: PropagationProposalId,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub reason: Option<String>,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct AcceptPropagationProposalCommand {
    pub proposal_id: PropagationProposalId,
}

impl CreatePropagationProposalCommand {
    pub fn into_proposal(self, created_at_ms: u64) -> PropagationProposal {
        PropagationProposal {
            id: self.proposal_id,
            action: self.action,
            target: self.target,
            status: SemanticProposalStatus::Pending,
            summary: self.summary,
            proposed_value: self.proposed_value,
            proposed_text: self.proposed_text,
            source_dependency_id: self.source_dependency_id,
            source_event_id: self.source_event_id,
            rationale: self.rationale,
            created_at_ms,
        }
    }
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct PropagationProposalListProjection {
    #[serde(default)]
    pub proposals: Vec<PropagationProposal>,
}

#[derive(Debug, Clone, PartialEq, Eq, thiserror::Error)]
pub enum PropagationProposalContractError {
    #[error("empty identifier for {0}")]
    EmptyIdentifier(&'static str),
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn propagation_proposal_id_rejects_empty_values() {
        assert!(PropagationProposalId::new(" ").is_err());
    }

    #[test]
    fn create_command_builds_pending_propagation_proposal() {
        let command = CreatePropagationProposalCommand {
            proposal_id: PropagationProposalId::new("proposal.propagation.weather").unwrap(),
            action: PropagationProposalAction::SetBibleField,
            target: PropagationProposalTarget::BibleField {
                node_id: BibleGraphNodeId::new("node.location.harbor").unwrap(),
                part_key: BibleGraphPartKey::new("weather").unwrap(),
                field_key: BibleGraphFieldKey::new("current").unwrap(),
                field_id: None,
            },
            summary: "Set harbor weather to rainy".to_string(),
            proposed_value: Some(FieldValue::Text("rainy".to_string())),
            proposed_text: None,
            source_dependency_id: Some(
                SemanticDependencyId::new("dependency.weather.scene").unwrap(),
            ),
            source_event_id: None,
            rationale: Some("Manual script edit introduced rainy weather".to_string()),
        };

        let proposal = command.into_proposal(42);

        assert_eq!(proposal.id.as_str(), "proposal.propagation.weather");
        assert_eq!(proposal.status, SemanticProposalStatus::Pending);
        assert_eq!(proposal.created_at_ms, 42);
    }

    #[test]
    fn update_command_round_trips() {
        let command = UpdatePropagationProposalCommand {
            proposal_id: PropagationProposalId::new("proposal.propagation.weather").unwrap(),
            action: PropagationProposalAction::SetBibleField,
            target: PropagationProposalTarget::BibleField {
                node_id: BibleGraphNodeId::new("node.location.harbor").unwrap(),
                part_key: BibleGraphPartKey::new("weather").unwrap(),
                field_key: BibleGraphFieldKey::new("current").unwrap(),
                field_id: None,
            },
            summary: "Set harbor weather to foggy".to_string(),
            proposed_value: Some(FieldValue::Text("foggy".to_string())),
            proposed_text: None,
            source_dependency_id: None,
            source_event_id: None,
            rationale: Some("Reviewer narrowed the propagation scope".to_string()),
        };

        let encoded = serde_json::to_string(&command).unwrap();
        let decoded: UpdatePropagationProposalCommand = serde_json::from_str(&encoded).unwrap();

        assert_eq!(decoded, command);
    }

    #[test]
    fn reject_command_round_trips() {
        let command = RejectPropagationProposalCommand {
            proposal_id: PropagationProposalId::new("proposal.propagation.weather").unwrap(),
            reason: Some("Wrong scope".to_string()),
        };

        let encoded = serde_json::to_string(&command).unwrap();
        let decoded: RejectPropagationProposalCommand = serde_json::from_str(&encoded).unwrap();

        assert_eq!(decoded, command);
    }

    #[test]
    fn accept_command_round_trips() {
        let command = AcceptPropagationProposalCommand {
            proposal_id: PropagationProposalId::new("proposal.propagation.weather").unwrap(),
        };

        let encoded = serde_json::to_string(&command).unwrap();
        let decoded: AcceptPropagationProposalCommand = serde_json::from_str(&encoded).unwrap();

        assert_eq!(decoded, command);
    }
}
