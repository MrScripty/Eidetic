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
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub impact: Option<ScriptImpactProjection>,
    #[serde(default)]
    pub blocks: Vec<ScriptBlockProjection>,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct ScriptBlockProjection {
    pub block: ScriptBlock,
    /// Exact canonical write identity; absent only on unpersisted projections.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub revision_event_id: Option<super::ChangeEventId>,
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
#[serde(deny_unknown_fields)]
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

/// Append user-authored screenplay to a captured timeline context. The backend
/// assigns persistent block/segment IDs and ordering from canonical state.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct CreateScriptBlockCommand {
    pub document_id: ScriptDocumentId,
    pub source_node_id: crate::timeline::node::NodeId,
    pub expected_start_ms: u64,
    pub expected_end_ms: u64,
    pub block_kind: ScriptBlockKind,
    pub text: String,
}

/// A manual edit preserves the canonical block's kind, placement and ownership.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct EditScriptBlockCommand {
    pub document_id: ScriptDocumentId,
    pub block_id: ScriptBlockId,
    pub expected_revision_event_id: super::ChangeEventId,
    pub text: String,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct ScriptContextBlock {
    pub document_id: ScriptDocumentId,
    pub segment_id: ScriptSegmentId,
    pub block_id: ScriptBlockId,
    pub source_node_id: Option<String>,
    pub revision_event_id: super::ChangeEventId,
    pub segment_revision_event_id: super::ChangeEventId,
    pub start_ms: u64,
    pub end_ms: u64,
    pub text: String,
}

/// Complete continuity-window selection captured with the screenplay inputs.
/// None on an older generation means completeness was not recorded.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct ScriptContextScope {
    pub node_id: crate::timeline::node::NodeId,
    pub start_ms: u64,
    pub end_ms: u64,
    pub segment_ids: Vec<ScriptSegmentId>,
    pub revision_event_id: Option<super::ChangeEventId>,
}

/// Canonical untimed field presence on scoped entities, including retained
/// relevance outside the resolver window. Actual supplied values/revisions remain
/// exclusively in BibleFieldInput. Relevance requires consumption or assignment.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct BibleContextScope {
    pub node_id: crate::timeline::node::NodeId,
    pub node_ids: Vec<super::BibleGraphNodeId>,
    pub field_ids: Vec<super::BibleGraphFieldId>,
    pub revision_event_id: Option<super::ChangeEventId>,
}

/// Canonical target custody captured before external generation. Existing
/// revisions also detect edit-and-restore ABA; absence preserves legacy replay.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct ScriptGenerationTarget {
    pub node_id: crate::timeline::node::NodeId,
    pub start_ms: u64,
    pub end_ms: u64,
    pub notes: String,
    pub node_revision_event_id: Option<super::ChangeEventId>,
    pub segment_revision_event_id: Option<super::ChangeEventId>,
    pub block_revision_event_id: Option<super::ChangeEventId>,
}

/// Internal generation commit: its captured evidence is part of the replay signature.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct GenerateScriptBlockCommand {
    pub block: SetScriptBlockCommand,
    pub script_inputs: Option<Vec<ScriptContextBlock>>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub bible_inputs: Option<Vec<super::BibleFieldInput>>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub bible_relationship_inputs: Option<Vec<super::BibleRelationshipInput>>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub bible_context_scope: Option<BibleContextScope>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub script_context_scope: Option<ScriptContextScope>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub target_binding: Option<ScriptGenerationTarget>,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct ScriptImpactProjection {
    pub generation_event_id: super::ChangeEventId,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub output_block_id: Option<ScriptBlockId>,
    pub lineage_available: bool,
    pub needs_review: bool,
    pub causes: Vec<ScriptImpactCause>,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct ScriptImpactCause {
    pub dependency_id: super::SemanticDependencyId,
    pub input: super::SemanticDependencyEndpoint,
    pub consumed_revision_event_id: super::ChangeEventId,
    pub current_revision_event_id: Option<super::ChangeEventId>,
    pub reason: ScriptImpactReason,
    pub input_excerpt: Option<String>,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum ScriptImpactReason {
    Changed,
    Deleted,
    ContextChanged,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
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
        let mut projection = ScriptDocumentProjection {
            document: ScriptDocument {
                id: ScriptDocumentId::new("script.document.main").unwrap(),
                title: "Pilot".to_string(),
                sort_order: 0,
            },
            segments: vec![ScriptSegmentProjection {
                impact: None,
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
                    revision_event_id: None,
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

        projection.segments[0].impact = Some(ScriptImpactProjection {
            generation_event_id: super::super::ChangeEventId(uuid::Uuid::new_v4()),
            output_block_id: Some(ScriptBlockId::new("script.block.heading-1").unwrap()),
            lineage_available: true,
            needs_review: true,
            causes: vec![ScriptImpactCause {
                dependency_id: super::super::SemanticDependencyId::new("generation.input").unwrap(),
                input: super::super::SemanticDependencyEndpoint::ScriptBlock {
                    block_id: ScriptBlockId::new("deleted.source").unwrap(),
                },
                consumed_revision_event_id: super::super::ChangeEventId(uuid::Uuid::new_v4()),
                current_revision_event_id: None,
                reason: ScriptImpactReason::Deleted,
                input_excerpt: Some("  Source — 雨\n\n".into()),
            }],
        });
        let json = serde_json::to_value(&projection).unwrap();
        assert_eq!(
            json["segments"][0]["impact"]["causes"][0]["reason"],
            "deleted"
        );
        assert!(json["segments"][0]["impact"]["causes"][0]["current_revision_event_id"].is_null());
        assert_eq!(
            serde_json::from_value::<ScriptDocumentProjection>(json).unwrap(),
            projection
        );
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

    #[test]
    fn complete_context_scope_round_trips_and_absent_legacy_scope_stays_absent() {
        let mut json = serde_json::json!({
            "block": {
                "document_id": "script.document.main", "document_title": "Pilot",
                "document_sort_order": 0, "segment_id": "segment.B",
                "source_node_id": "00000000-0000-0000-0000-000000000005",
                "segment_start_ms": 4000, "segment_end_ms": 5000,
                "segment_status": "current", "segment_sort_order": 0,
                "block_id": "block.B", "block_kind": "action", "text": "Exact B\n\n",
                "span_provenance": "ai_generated", "sort_order": 0
            }, "script_inputs": []
        });
        let legacy: GenerateScriptBlockCommand = serde_json::from_value(json.clone()).unwrap();
        assert!(legacy.script_context_scope.is_none());
        assert!(
            serde_json::to_value(legacy)
                .unwrap()
                .get("script_context_scope")
                .is_none()
        );
        json["script_context_scope"] = serde_json::json!({
            "node_id": "00000000-0000-0000-0000-000000000005", "start_ms": 4000,
            "end_ms": 5000, "segment_ids": [], "revision_event_id": null
        });
        let complete: GenerateScriptBlockCommand = serde_json::from_value(json).unwrap();
        assert_eq!(
            complete
                .script_context_scope
                .as_ref()
                .unwrap()
                .segment_ids
                .len(),
            0
        );
        assert_eq!(
            serde_json::from_value::<GenerateScriptBlockCommand>(
                serde_json::to_value(&complete).unwrap()
            )
            .unwrap(),
            complete
        );
    }
}

#[cfg(test)]
mod creation_contract_tests {
    use super::*;

    #[test]
    fn generation_target_round_trips_without_backfilling_legacy_history() {
        let old = serde_json::json!({
            "block": {"document_id":"script.document.main", "document_title":"Story", "segment_id":"segment.B",
            "source_node_id":"B", "segment_start_ms":1000,"segment_end_ms":2000,"segment_status":"current",
            "block_id":"block.B","block_kind":"action","text":"B","span_provenance":"ai_generated"},
            "script_inputs":null
        });
        let legacy: GenerateScriptBlockCommand = serde_json::from_value(old.clone()).unwrap();
        assert!(legacy.target_binding.is_none());
        assert!(
            serde_json::to_value(legacy)
                .unwrap()
                .get("target_binding")
                .is_none()
        );
        let mut json = old;
        json["target_binding"] = serde_json::json!({
            "node_id":uuid::Uuid::new_v4(),"start_ms":1000,"end_ms":2000,"notes":"Exact notes — 雨",
            "node_revision_event_id":uuid::Uuid::new_v4(),"segment_revision_event_id":null,"block_revision_event_id":null
        });
        let command: GenerateScriptBlockCommand = serde_json::from_value(json).unwrap();
        assert_eq!(
            command,
            serde_json::from_value(serde_json::to_value(&command).unwrap()).unwrap()
        );
    }

    #[test]
    fn bible_membership_round_trips_and_legacy_absence_is_not_backfilled() {
        let old = serde_json::json!({"block":{"document_id":"script.document.main","document_title":"Story","segment_id":"segment.B","source_node_id":"B","segment_start_ms":1000,"segment_end_ms":2000,"segment_status":"current","block_id":"block.B","block_kind":"action","text":"Exact B\n\n","span_provenance":"ai_generated"},"script_inputs":null});
        let legacy: GenerateScriptBlockCommand = serde_json::from_value(old.clone()).unwrap();
        assert!(legacy.bible_context_scope.is_none());
        assert!(
            serde_json::to_value(&legacy)
                .unwrap()
                .get("bible_context_scope")
                .is_none()
        );
        let mut json = old;
        json["bible_context_scope"] = serde_json::json!({"node_id":uuid::Uuid::new_v4(),"node_ids":["Mara"],"field_ids":["Mara.tagline"],"revision_event_id":uuid::Uuid::new_v4()});
        let complete: GenerateScriptBlockCommand = serde_json::from_value(json).unwrap();
        assert_eq!(
            complete,
            serde_json::from_value(serde_json::to_value(&complete).unwrap()).unwrap()
        );
    }

    #[test]
    fn manual_creation_round_trip_preserves_intent_and_refuses_client_owned_ids() {
        let command = CreateScriptBlockCommand {
            document_id: ScriptDocumentId::new("script.document.main").unwrap(),
            source_node_id: crate::timeline::node::NodeId(uuid::Uuid::from_u128(42)),
            expected_start_ms: 1000,
            expected_end_ms: 2000,
            block_kind: ScriptBlockKind::SceneHeading,
            text: "  INT. CAFE — 雨\n\n".into(),
        };
        let mut json = serde_json::to_value(&command).unwrap();
        assert_eq!(
            serde_json::from_value::<CreateScriptBlockCommand>(json.clone()).unwrap(),
            command
        );
        json["block_id"] = serde_json::json!("client.chosen.block");
        assert!(serde_json::from_value::<CreateScriptBlockCommand>(json).is_err());
    }
}
