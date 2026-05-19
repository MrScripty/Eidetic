use serde::{Deserialize, Serialize};

use crate::timeline::node::NodeId;

use super::{
    BibleGraphFieldId, BibleGraphFieldKey, BibleGraphNodeId, BibleGraphPartKey, ScriptBlockId,
    ScriptSegmentId,
};

#[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[serde(try_from = "String", into = "String")]
pub struct SemanticDependencyId(String);

impl SemanticDependencyId {
    pub fn new(value: impl Into<String>) -> Result<Self, SemanticDependencyContractError> {
        let value = value.into();
        if value.trim().is_empty() {
            return Err(SemanticDependencyContractError::EmptyIdentifier(
                "SemanticDependencyId",
            ));
        }
        Ok(Self(value))
    }

    pub fn as_str(&self) -> &str {
        &self.0
    }
}

impl TryFrom<String> for SemanticDependencyId {
    type Error = SemanticDependencyContractError;

    fn try_from(value: String) -> Result<Self, Self::Error> {
        Self::new(value)
    }
}

impl From<SemanticDependencyId> for String {
    fn from(value: SemanticDependencyId) -> Self {
        value.0
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case", tag = "kind")]
pub enum SemanticDependencyEndpoint {
    TimelineNode {
        node_id: NodeId,
    },
    BibleNode {
        node_id: BibleGraphNodeId,
    },
    BibleField {
        node_id: BibleGraphNodeId,
        part_key: BibleGraphPartKey,
        field_key: BibleGraphFieldKey,
        #[serde(default, skip_serializing_if = "Option::is_none")]
        field_id: Option<BibleGraphFieldId>,
    },
    ScriptSegment {
        segment_id: ScriptSegmentId,
    },
    ScriptBlock {
        block_id: ScriptBlockId,
    },
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum SemanticDependencyKind {
    Mentions,
    UsesFact,
    ContradictsFact,
    DerivesFrom,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct SemanticDependency {
    pub id: SemanticDependencyId,
    pub source: SemanticDependencyEndpoint,
    pub target: SemanticDependencyEndpoint,
    pub kind: SemanticDependencyKind,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub rationale: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub confidence: Option<f32>,
    pub created_at_ms: u64,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct RecordSemanticDependencyCommand {
    pub dependency: SemanticDependency,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct SemanticDependencyProjection {
    #[serde(default)]
    pub dependencies: Vec<SemanticDependency>,
}

#[derive(Debug, Clone, PartialEq, Eq, thiserror::Error)]
pub enum SemanticDependencyContractError {
    #[error("empty identifier for {0}")]
    EmptyIdentifier(&'static str),
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn semantic_dependency_id_rejects_empty_values() {
        assert!(SemanticDependencyId::new(" ").is_err());
    }

    #[test]
    fn semantic_dependency_round_trips_bible_field_endpoint() {
        let dependency = SemanticDependency {
            id: SemanticDependencyId::new("dependency.weather.scene").unwrap(),
            source: SemanticDependencyEndpoint::ScriptSegment {
                segment_id: ScriptSegmentId::new("script.segment.scene-1").unwrap(),
            },
            target: SemanticDependencyEndpoint::BibleField {
                node_id: BibleGraphNodeId::new("node.location.harbor").unwrap(),
                part_key: BibleGraphPartKey::new("weather").unwrap(),
                field_key: BibleGraphFieldKey::new("current").unwrap(),
                field_id: None,
            },
            kind: SemanticDependencyKind::UsesFact,
            rationale: Some("Scene mentions rainy weather".to_string()),
            confidence: Some(0.91),
            created_at_ms: 42,
        };

        let encoded = serde_json::to_string(&dependency).unwrap();
        let decoded: SemanticDependency = serde_json::from_str(&encoded).unwrap();

        assert_eq!(decoded, dependency);
    }
}
