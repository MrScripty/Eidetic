use serde::{Deserialize, Serialize};
use uuid::Uuid;

use crate::timeline::node::NodeId;

use super::{
    AgentWorkflowId, BibleGraphFieldKey, BibleGraphNodeId, BibleGraphPartKey, BibleGraphSnapshotId,
    CommandId, ScriptSegmentId,
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
pub struct AffectProjection {
    pub target: AffectTarget,
    pub values: Vec<AffectValue>,
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
}
