use serde::{Deserialize, Serialize};

macro_rules! non_empty_string_id {
    ($name:ident) => {
        #[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord, Hash, Serialize, Deserialize)]
        #[serde(try_from = "String", into = "String")]
        pub struct $name(String);

        impl $name {
            pub fn new(value: impl Into<String>) -> Result<Self, ScriptContractError> {
                let value = value.into();
                if value.trim().is_empty() {
                    return Err(ScriptContractError::EmptyIdentifier(stringify!($name)));
                }
                Ok(Self(value))
            }

            pub fn as_str(&self) -> &str {
                &self.0
            }
        }

        impl TryFrom<String> for $name {
            type Error = ScriptContractError;

            fn try_from(value: String) -> Result<Self, Self::Error> {
                Self::new(value)
            }
        }

        impl From<$name> for String {
            fn from(value: $name) -> Self {
                value.0
            }
        }
    };
}

non_empty_string_id!(ScriptDocumentId);
non_empty_string_id!(ScriptSegmentId);
non_empty_string_id!(ScriptBlockId);
non_empty_string_id!(ScriptSpanId);
non_empty_string_id!(ScriptLockId);
non_empty_string_id!(ScriptPatchId);

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct ScriptDocument {
    pub id: ScriptDocumentId,
    pub title: String,
    #[serde(default)]
    pub sort_order: u32,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct ScriptSegment {
    pub id: ScriptSegmentId,
    pub document_id: ScriptDocumentId,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub source_node_id: Option<String>,
    pub start_ms: u64,
    pub end_ms: u64,
    pub status: ScriptSegmentStatus,
    #[serde(default)]
    pub sort_order: u32,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum ScriptSegmentStatus {
    Current,
    Stale,
    Regenerating,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct ScriptBlock {
    pub id: ScriptBlockId,
    pub segment_id: ScriptSegmentId,
    pub block_kind: ScriptBlockKind,
    pub text: String,
    #[serde(default)]
    pub sort_order: u32,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum ScriptBlockKind {
    SceneHeading,
    Action,
    Character,
    Parenthetical,
    Dialogue,
    Transition,
    Shot,
    Note,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct ScriptSpan {
    pub id: ScriptSpanId,
    pub block_id: ScriptBlockId,
    pub start_byte: u32,
    pub end_byte: u32,
    pub provenance: ScriptSpanProvenance,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum ScriptSpanProvenance {
    AiGenerated,
    UserEdited,
    Imported,
    System,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct ScriptLock {
    pub id: ScriptLockId,
    pub span_id: ScriptSpanId,
    pub reason: String,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct ScriptPatch {
    pub id: ScriptPatchId,
    pub document_id: ScriptDocumentId,
    #[serde(default)]
    pub segments: Vec<ScriptSegmentProjection>,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct ScriptSegmentProjection {
    pub segment: ScriptSegment,
    #[serde(default)]
    pub blocks: Vec<ScriptBlockProjection>,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct ScriptBlockProjection {
    pub block: ScriptBlock,
    #[serde(default)]
    pub spans: Vec<ScriptSpan>,
    #[serde(default)]
    pub locks: Vec<ScriptLock>,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct ScriptDocumentProjection {
    pub document: ScriptDocument,
    #[serde(default)]
    pub segments: Vec<ScriptSegmentProjection>,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct SetScriptBlockCommand {
    pub document_id: ScriptDocumentId,
    pub document_title: String,
    #[serde(default)]
    pub document_sort_order: u32,
    pub segment_id: ScriptSegmentId,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub source_node_id: Option<String>,
    pub segment_start_ms: u64,
    pub segment_end_ms: u64,
    pub segment_status: ScriptSegmentStatus,
    #[serde(default)]
    pub segment_sort_order: u32,
    pub block_id: ScriptBlockId,
    pub block_kind: ScriptBlockKind,
    pub text: String,
    #[serde(default = "default_script_span_provenance")]
    pub span_provenance: ScriptSpanProvenance,
    #[serde(default)]
    pub sort_order: u32,
}

fn default_script_span_provenance() -> ScriptSpanProvenance {
    ScriptSpanProvenance::UserEdited
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct SetScriptLockCommand {
    pub lock_id: ScriptLockId,
    pub span_id: ScriptSpanId,
    pub reason: String,
}

#[derive(Debug, Clone, PartialEq, Eq, thiserror::Error)]
pub enum ScriptContractError {
    #[error("{0} must not be empty")]
    EmptyIdentifier(&'static str),
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn script_identifiers_reject_empty_values() {
        let error = ScriptDocumentId::new("  ").unwrap_err();

        assert_eq!(
            error,
            ScriptContractError::EmptyIdentifier("ScriptDocumentId")
        );
    }

    #[test]
    fn script_document_projection_round_trips() {
        let projection = ScriptDocumentProjection {
            document: ScriptDocument {
                id: ScriptDocumentId::new("script.document.main").unwrap(),
                title: "Pilot".to_string(),
                sort_order: 0,
            },
            segments: vec![ScriptSegmentProjection {
                segment: ScriptSegment {
                    id: ScriptSegmentId::new("script.segment.beat-1").unwrap(),
                    document_id: ScriptDocumentId::new("script.document.main").unwrap(),
                    source_node_id: Some("node.beat.opening".to_string()),
                    start_ms: 1_000,
                    end_ms: 5_000,
                    status: ScriptSegmentStatus::Current,
                    sort_order: 1,
                },
                blocks: vec![ScriptBlockProjection {
                    block: ScriptBlock {
                        id: ScriptBlockId::new("script.block.heading-1").unwrap(),
                        segment_id: ScriptSegmentId::new("script.segment.beat-1").unwrap(),
                        block_kind: ScriptBlockKind::SceneHeading,
                        text: "INT. KITCHEN - MORNING".to_string(),
                        sort_order: 1,
                    },
                    spans: vec![ScriptSpan {
                        id: ScriptSpanId::new("script.span.heading-1").unwrap(),
                        block_id: ScriptBlockId::new("script.block.heading-1").unwrap(),
                        start_byte: 0,
                        end_byte: 22,
                        provenance: ScriptSpanProvenance::AiGenerated,
                    }],
                    locks: vec![ScriptLock {
                        id: ScriptLockId::new("script.lock.heading-1").unwrap(),
                        span_id: ScriptSpanId::new("script.span.heading-1").unwrap(),
                        reason: "User approved location wording".to_string(),
                    }],
                }],
            }],
        };

        let json = serde_json::to_string(&projection).unwrap();
        let round_trip: ScriptDocumentProjection = serde_json::from_str(&json).unwrap();

        assert_eq!(round_trip, projection);
    }

    #[test]
    fn script_block_command_round_trips() {
        let command = SetScriptBlockCommand {
            document_id: ScriptDocumentId::new("script.document.main").unwrap(),
            document_title: "Pilot".to_string(),
            document_sort_order: 0,
            segment_id: ScriptSegmentId::new("script.segment.beat-1").unwrap(),
            source_node_id: Some("node.beat.opening".to_string()),
            segment_start_ms: 1_000,
            segment_end_ms: 5_000,
            segment_status: ScriptSegmentStatus::Current,
            segment_sort_order: 1,
            block_id: ScriptBlockId::new("script.block.action-1").unwrap(),
            block_kind: ScriptBlockKind::Action,
            text: "Ada enters with a wet umbrella.".to_string(),
            span_provenance: ScriptSpanProvenance::UserEdited,
            sort_order: 2,
        };

        let json = serde_json::to_string(&command).unwrap();
        let round_trip: SetScriptBlockCommand = serde_json::from_str(&json).unwrap();

        assert_eq!(round_trip, command);
    }

    #[test]
    fn script_lock_command_round_trips() {
        let command = SetScriptLockCommand {
            lock_id: ScriptLockId::new("script.lock.action-1").unwrap(),
            span_id: ScriptSpanId::new("script.span.action-1").unwrap(),
            reason: "Manual edit".to_string(),
        };

        let json = serde_json::to_string(&command).unwrap();
        let round_trip: SetScriptLockCommand = serde_json::from_str(&json).unwrap();

        assert_eq!(round_trip, command);
    }
}
