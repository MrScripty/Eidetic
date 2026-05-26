use serde::{Deserialize, Serialize};
use uuid::Uuid;

use crate::timeline::node::NodeId;

use super::{
    AgentWorkflowId, BibleGraphFieldKey, BibleGraphNodeId, BibleGraphPartKey, BibleGraphSnapshotId,
    ChangeEventId, CommandId, ScriptSegmentId, SemanticProposalStatus,
};

const VALENCE_MIN: i16 = -1000;
const VALENCE_MAX: i16 = 1000;
const ZERO_TO_ONE_MIN: u16 = 0;
const ZERO_TO_ONE_MAX: u16 = 1000;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub struct AffectValueId(pub Uuid);

impl AffectValueId {
    pub fn new() -> Self {
        Self(Uuid::new_v4())
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub struct AffectDependencyId(pub Uuid);

impl AffectDependencyId {
    pub fn new() -> Self {
        Self(Uuid::new_v4())
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[serde(try_from = "String", into = "String")]
pub struct AffectProposalId(String);

impl AffectProposalId {
    pub fn new(value: impl Into<String>) -> Result<Self, AffectContractError> {
        let value = value.into();
        if value.trim().is_empty() {
            return Err(AffectContractError::EmptyProposalId);
        }
        Ok(Self(value))
    }

    pub fn as_str(&self) -> &str {
        &self.0
    }
}

impl TryFrom<String> for AffectProposalId {
    type Error = AffectContractError;

    fn try_from(value: String) -> Result<Self, Self::Error> {
        Self::new(value)
    }
}

impl From<AffectProposalId> for String {
    fn from(value: AffectProposalId) -> Self {
        value.0
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(transparent)]
pub struct Valence(i16);

impl Valence {
    pub fn new(value: i16) -> Result<Self, AffectContractError> {
        validate_i16_range("valence", value, VALENCE_MIN, VALENCE_MAX)?;
        Ok(Self(value))
    }

    pub fn basis_points(self) -> i16 {
        self.0
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(transparent)]
pub struct Arousal(u16);

impl Arousal {
    pub fn new(value: u16) -> Result<Self, AffectContractError> {
        validate_u16_range("arousal", value, ZERO_TO_ONE_MIN, ZERO_TO_ONE_MAX)?;
        Ok(Self(value))
    }

    pub fn basis_points(self) -> u16 {
        self.0
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(transparent)]
pub struct EmotionalIntensity(u16);

impl EmotionalIntensity {
    pub fn new(value: u16) -> Result<Self, AffectContractError> {
        validate_u16_range(
            "emotional_intensity",
            value,
            ZERO_TO_ONE_MIN,
            ZERO_TO_ONE_MAX,
        )?;
        Ok(Self(value))
    }

    pub fn basis_points(self) -> u16 {
        self.0
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(transparent)]
pub struct AffectConfidence(u16);

impl AffectConfidence {
    pub fn new(value: u16) -> Result<Self, AffectContractError> {
        validate_u16_range("confidence", value, ZERO_TO_ONE_MIN, ZERO_TO_ONE_MAX)?;
        Ok(Self(value))
    }

    pub fn basis_points(self) -> u16 {
        self.0
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[serde(try_from = "String", into = "String")]
pub struct MoodLabel(String);

impl MoodLabel {
    pub fn new(value: impl Into<String>) -> Result<Self, AffectContractError> {
        let value = value.into();
        if value.trim().is_empty() {
            return Err(AffectContractError::EmptyMoodLabel);
        }
        Ok(Self(value))
    }

    pub fn as_str(&self) -> &str {
        &self.0
    }
}

impl TryFrom<String> for MoodLabel {
    type Error = AffectContractError;

    fn try_from(value: String) -> Result<Self, Self::Error> {
        Self::new(value)
    }
}

impl From<MoodLabel> for String {
    fn from(value: MoodLabel) -> Self {
        value.0
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(tag = "type", rename_all = "snake_case", deny_unknown_fields)]
pub enum AffectTarget {
    Project,
    TimelineNode { node_id: NodeId },
    ScriptSegment { segment_id: ScriptSegmentId },
    BibleNode { node_id: BibleGraphNodeId },
    BibleSnapshot { snapshot_id: BibleGraphSnapshotId },
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum AffectProvenance {
    UserAuthored,
    AgentProposed,
    ScriptEditDetected,
    Imported,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum AffectTraitKind {
    Valence,
    Arousal,
    EmotionalIntensity,
    Confidence,
    MoodLabel,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(tag = "type", rename_all = "snake_case", deny_unknown_fields)]
pub enum AffectDependencyEndpoint {
    TimelineNode {
        node_id: NodeId,
    },
    ScriptSegment {
        segment_id: ScriptSegmentId,
    },
    BibleNode {
        node_id: BibleGraphNodeId,
    },
    BibleField {
        node_id: BibleGraphNodeId,
        part_key: BibleGraphPartKey,
        field_key: BibleGraphFieldKey,
    },
    GenerationPrompt {
        workflow_id: AgentWorkflowId,
    },
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct AffectDependency {
    pub id: AffectDependencyId,
    pub affect_id: AffectValueId,
    pub trait_kind: AffectTraitKind,
    pub source: AffectDependencyEndpoint,
    pub target: AffectDependencyEndpoint,
    pub reason: String,
}

impl AffectDependency {
    pub fn validate(&self) -> Result<(), AffectContractError> {
        if self.reason.trim().is_empty() {
            return Err(AffectContractError::EmptyDependencyReason);
        }
        Ok(())
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct AffectValue {
    pub id: AffectValueId,
    pub target: AffectTarget,
    pub valence: Valence,
    pub arousal: Arousal,
    pub intensity: EmotionalIntensity,
    pub confidence: AffectConfidence,
    pub mood_labels: Vec<MoodLabel>,
    pub provenance: AffectProvenance,
    pub rationale: Option<String>,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum AffectProposalSource {
    ManualScriptEdit,
    AgentAnalysis,
    UserDraft,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct AffectProposal {
    pub id: AffectProposalId,
    pub status: SemanticProposalStatus,
    pub source: AffectProposalSource,
    pub proposed_value: AffectValue,
    pub summary: String,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub rationale: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub source_event_id: Option<ChangeEventId>,
    pub created_at_ms: u64,
}

impl AffectProposal {
    pub fn validate(&self) -> Result<(), AffectContractError> {
        validate_proposal_text(&self.summary, AffectContractError::EmptyProposalSummary)?;
        if self
            .rationale
            .as_deref()
            .is_some_and(|value| value.trim().is_empty())
        {
            return Err(AffectContractError::EmptyProposalRationale);
        }
        self.proposed_value.validate()
    }
}

impl AffectValue {
    pub fn validate(&self) -> Result<(), AffectContractError> {
        if self.mood_labels.is_empty() {
            return Err(AffectContractError::MissingMoodLabel);
        }
        if self
            .rationale
            .as_deref()
            .is_some_and(|value| value.trim().is_empty())
        {
            return Err(AffectContractError::EmptyRationale);
        }
        Ok(())
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct SetAffectValueCommand {
    pub command_id: CommandId,
    pub affect_id: AffectValueId,
    pub target: AffectTarget,
    pub valence: Valence,
    pub arousal: Arousal,
    pub intensity: EmotionalIntensity,
    pub confidence: AffectConfidence,
    pub mood_labels: Vec<MoodLabel>,
    pub provenance: AffectProvenance,
    pub rationale: Option<String>,
}

impl SetAffectValueCommand {
    pub fn validate(&self) -> Result<(), AffectContractError> {
        AffectValue {
            id: self.affect_id,
            target: self.target.clone(),
            valence: self.valence,
            arousal: self.arousal,
            intensity: self.intensity,
            confidence: self.confidence,
            mood_labels: self.mood_labels.clone(),
            provenance: self.provenance.clone(),
            rationale: self.rationale.clone(),
        }
        .validate()
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct DeleteAffectValueCommand {
    pub command_id: CommandId,
    pub affect_id: AffectValueId,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct RecordAffectDependencyCommand {
    pub command_id: CommandId,
    pub dependency: AffectDependency,
}

impl RecordAffectDependencyCommand {
    pub fn validate(&self) -> Result<(), AffectContractError> {
        self.dependency.validate()
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct CreateAffectProposalCommand {
    pub proposal_id: AffectProposalId,
    pub source: AffectProposalSource,
    pub proposed_value: AffectValue,
    pub summary: String,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub rationale: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub source_event_id: Option<ChangeEventId>,
}

impl CreateAffectProposalCommand {
    pub fn validate(&self) -> Result<(), AffectContractError> {
        validate_proposal_text(&self.summary, AffectContractError::EmptyProposalSummary)?;
        if self
            .rationale
            .as_deref()
            .is_some_and(|value| value.trim().is_empty())
        {
            return Err(AffectContractError::EmptyProposalRationale);
        }
        self.proposed_value.validate()
    }

    pub fn into_proposal(self, created_at_ms: u64) -> AffectProposal {
        AffectProposal {
            id: self.proposal_id,
            status: SemanticProposalStatus::Pending,
            source: self.source,
            proposed_value: self.proposed_value,
            summary: self.summary,
            rationale: self.rationale,
            source_event_id: self.source_event_id,
            created_at_ms,
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct RejectAffectProposalCommand {
    pub proposal_id: AffectProposalId,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub reason: Option<String>,
}

impl RejectAffectProposalCommand {
    pub fn validate(&self) -> Result<(), AffectContractError> {
        if self
            .reason
            .as_deref()
            .is_some_and(|value| value.trim().is_empty())
        {
            return Err(AffectContractError::EmptyProposalRationale);
        }
        Ok(())
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct AcceptAffectProposalCommand {
    pub proposal_id: AffectProposalId,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct AffectProjection {
    pub target: AffectTarget,
    pub values: Vec<AffectValue>,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct AffectProposalListProjection {
    #[serde(default)]
    pub proposals: Vec<AffectProposal>,
}

#[derive(Debug, Clone, PartialEq, Eq, thiserror::Error)]
pub enum AffectContractError {
    #[error("{field} value {value} is outside inclusive range {min}..={max}")]
    Range {
        field: &'static str,
        value: i32,
        min: i32,
        max: i32,
    },
    #[error("mood label cannot be empty")]
    EmptyMoodLabel,
    #[error("affect value must include at least one mood label")]
    MissingMoodLabel,
    #[error("affect rationale cannot be empty when provided")]
    EmptyRationale,
    #[error("affect dependency reason cannot be empty")]
    EmptyDependencyReason,
    #[error("affect proposal id cannot be empty")]
    EmptyProposalId,
    #[error("affect proposal summary cannot be empty")]
    EmptyProposalSummary,
    #[error("affect proposal rationale cannot be empty when provided")]
    EmptyProposalRationale,
}

fn validate_i16_range(
    field: &'static str,
    value: i16,
    min: i16,
    max: i16,
) -> Result<(), AffectContractError> {
    if value < min || value > max {
        return Err(AffectContractError::Range {
            field,
            value: i32::from(value),
            min: i32::from(min),
            max: i32::from(max),
        });
    }
    Ok(())
}

fn validate_u16_range(
    field: &'static str,
    value: u16,
    min: u16,
    max: u16,
) -> Result<(), AffectContractError> {
    if value < min || value > max {
        return Err(AffectContractError::Range {
            field,
            value: i32::from(value),
            min: i32::from(min),
            max: i32::from(max),
        });
    }
    Ok(())
}

fn validate_proposal_text(
    value: &str,
    error: AffectContractError,
) -> Result<(), AffectContractError> {
    if value.trim().is_empty() {
        return Err(error);
    }
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn affect_domain_values_validate_ranges() {
        assert_eq!(Valence::new(-1000).unwrap().basis_points(), -1000);
        assert_eq!(Valence::new(1000).unwrap().basis_points(), 1000);
        assert_eq!(Arousal::new(1000).unwrap().basis_points(), 1000);
        assert_eq!(EmotionalIntensity::new(0).unwrap().basis_points(), 0);
        assert_eq!(AffectConfidence::new(750).unwrap().basis_points(), 750);
        assert!(Valence::new(-1001).is_err());
        assert!(Arousal::new(1001).is_err());
        assert!(EmotionalIntensity::new(1001).is_err());
        assert!(AffectConfidence::new(1001).is_err());
    }

    #[test]
    fn affect_command_rejects_empty_mood_and_rationale() {
        let command = SetAffectValueCommand {
            command_id: CommandId::new(),
            affect_id: AffectValueId::new(),
            target: AffectTarget::TimelineNode {
                node_id: NodeId::new(),
            },
            valence: Valence::new(250).unwrap(),
            arousal: Arousal::new(700).unwrap(),
            intensity: EmotionalIntensity::new(850).unwrap(),
            confidence: AffectConfidence::new(900).unwrap(),
            mood_labels: Vec::new(),
            provenance: AffectProvenance::UserAuthored,
            rationale: Some(" ".to_string()),
        };

        assert_eq!(
            command.validate(),
            Err(AffectContractError::MissingMoodLabel)
        );
        assert!(MoodLabel::new(" ").is_err());
    }

    #[test]
    fn affect_contract_round_trips_with_typed_target() {
        let command = SetAffectValueCommand {
            command_id: CommandId::new(),
            affect_id: AffectValueId::new(),
            target: AffectTarget::Project,
            valence: Valence::new(-125).unwrap(),
            arousal: Arousal::new(500).unwrap(),
            intensity: EmotionalIntensity::new(640).unwrap(),
            confidence: AffectConfidence::new(800).unwrap(),
            mood_labels: vec![MoodLabel::new("uneasy").unwrap()],
            provenance: AffectProvenance::AgentProposed,
            rationale: Some("Premise mood trends downward.".to_string()),
        };

        command.validate().unwrap();
        let encoded = serde_json::to_string(&command).unwrap();
        let decoded: SetAffectValueCommand = serde_json::from_str(&encoded).unwrap();

        assert_eq!(decoded, command);
    }

    #[test]
    fn affect_dependency_contract_round_trips_and_validates_reason() {
        let command = RecordAffectDependencyCommand {
            command_id: CommandId::new(),
            dependency: AffectDependency {
                id: AffectDependencyId::new(),
                affect_id: AffectValueId::new(),
                trait_kind: AffectTraitKind::Valence,
                source: AffectDependencyEndpoint::TimelineNode {
                    node_id: NodeId::new(),
                },
                target: AffectDependencyEndpoint::GenerationPrompt {
                    workflow_id: AgentWorkflowId::new("workflow.scene.graph_context").unwrap(),
                },
                reason: "Scene valence constrains the generation prompt.".to_string(),
            },
        };

        command.validate().unwrap();
        let encoded = serde_json::to_string(&command).unwrap();
        let decoded: RecordAffectDependencyCommand = serde_json::from_str(&encoded).unwrap();

        assert_eq!(decoded, command);

        let mut invalid = command;
        invalid.dependency.reason = " ".to_string();
        assert_eq!(
            invalid.validate(),
            Err(AffectContractError::EmptyDependencyReason)
        );
    }

    #[test]
    fn affect_proposal_command_builds_reviewable_pending_proposal() {
        let proposed_value = AffectValue {
            id: AffectValueId::new(),
            target: AffectTarget::TimelineNode {
                node_id: NodeId::new(),
            },
            valence: Valence::new(-250).unwrap(),
            arousal: Arousal::new(650).unwrap(),
            intensity: EmotionalIntensity::new(700).unwrap(),
            confidence: AffectConfidence::new(900).unwrap(),
            mood_labels: vec![MoodLabel::new("uneasy").unwrap()],
            provenance: AffectProvenance::ScriptEditDetected,
            rationale: Some("Manual edit changed the scene mood.".to_string()),
        };
        let command = CreateAffectProposalCommand {
            proposal_id: AffectProposalId::new("proposal.affect.scene-mood").unwrap(),
            source: AffectProposalSource::ManualScriptEdit,
            proposed_value: proposed_value.clone(),
            summary: "Scene mood became uneasy".to_string(),
            rationale: Some("The edited action implies a darker tone.".to_string()),
            source_event_id: Some(ChangeEventId::new()),
        };

        command.validate().unwrap();
        let proposal = command.into_proposal(42);

        assert_eq!(proposal.status, SemanticProposalStatus::Pending);
        assert_eq!(proposal.proposed_value, proposed_value);
        assert_eq!(proposal.created_at_ms, 42);
    }

    #[test]
    fn affect_proposal_commands_reject_empty_review_text() {
        let proposed_value = AffectValue {
            id: AffectValueId::new(),
            target: AffectTarget::Project,
            valence: Valence::new(0).unwrap(),
            arousal: Arousal::new(500).unwrap(),
            intensity: EmotionalIntensity::new(500).unwrap(),
            confidence: AffectConfidence::new(500).unwrap(),
            mood_labels: vec![MoodLabel::new("neutral").unwrap()],
            provenance: AffectProvenance::AgentProposed,
            rationale: None,
        };
        let command = CreateAffectProposalCommand {
            proposal_id: AffectProposalId::new("proposal.affect.empty-summary").unwrap(),
            source: AffectProposalSource::AgentAnalysis,
            proposed_value,
            summary: " ".to_string(),
            rationale: None,
            source_event_id: None,
        };

        assert_eq!(
            command.validate(),
            Err(AffectContractError::EmptyProposalSummary)
        );
        assert!(AffectProposalId::new(" ").is_err());
        assert_eq!(
            RejectAffectProposalCommand {
                proposal_id: AffectProposalId::new("proposal.affect.reject").unwrap(),
                reason: Some(" ".to_string()),
            }
            .validate(),
            Err(AffectContractError::EmptyProposalRationale)
        );
    }
}
