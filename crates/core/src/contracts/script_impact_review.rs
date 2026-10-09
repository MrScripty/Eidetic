use serde::{Deserialize, Serialize};

use super::{
    AiBibleContextProjection, ChangeEventId, ProjectionEnvelope, PropagationProposalId,
    ScriptBlockId, ScriptContextBlock, ScriptDocumentId, ScriptImpactCause, ScriptSegmentId,
    SemanticDependencyId,
};

/// Explicit request for a reviewable update, never authorization to apply it.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct RequestScriptImpactProposalCommand {
    pub proposal_id: PropagationProposalId,
    pub document_id: ScriptDocumentId,
    pub segment_id: ScriptSegmentId,
    pub block_id: ScriptBlockId,
    pub expected_block_revision_event_id: ChangeEventId,
    pub generation_event_id: ChangeEventId,
    pub dependency_id: SemanticDependencyId,
    #[serde(default)]
    pub story_time_ms: Option<u64>,
    /// Optional author selection, never client-supplied prompt values.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub recall_selection: Option<ScriptRecallSelection>,
}

/// Bounded supplemental evidence for one existing impact preview.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct ScriptRecallSelection {
    pub query: super::BibleRecallRequest,
    pub facts: Vec<ScriptRecallFactSelection>,
    /// Exact displayed name/path receipts; canonical reads must match these.
    pub names: Vec<super::BibleNodeNameInput>,
    pub paths: Vec<super::BibleRecallPath>,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct ScriptRecallFactSelection {
    pub node_id: super::BibleGraphNodeId,
    pub part_key: super::BibleGraphPartKey,
    pub field_key: super::BibleGraphFieldKey,
    pub field_id: super::BibleGraphFieldId,
    pub revision_event_id: ChangeEventId,
}

/// Canonical evidence captured before provider I/O and rechecked at acceptance.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct ScriptImpactProposalBinding {
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub arc_previous_inputs: Option<Vec<super::StoryArcFieldInput>>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub arc_inputs: Option<Vec<super::StoryArcFieldInput>>,
    /// Deletion custody for previously bound arc fields; not consumed values.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub arc_absence_revisions: Option<Vec<(crate::story::arc::ArcId, ChangeEventId)>>,
    pub request: RequestScriptImpactProposalCommand,
    pub cause: ScriptImpactCause,
    pub target_segment_revision_event_id: ChangeEventId,
    pub script_inputs: Vec<ScriptContextBlock>,
    pub bible_context: ProjectionEnvelope<AiBibleContextProjection>,
    #[serde(default)]
    pub bible_inputs: Vec<super::BibleFieldInput>,
    /// Names actually supplied; absent legacy receipts remain unknown.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub bible_node_name_inputs: Option<Vec<super::BibleNodeNameInput>>,
    /// Absent legacy receipts stay unknown; Some(empty) is a recorded read.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub bible_relationship_inputs: Option<Vec<super::BibleRelationshipInput>>,
    /// Negative name reads use the node's owned history to detect ABA.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub bible_node_name_absence_revisions: Option<Vec<(super::BibleGraphNodeId, ChangeEventId)>>,
    /// Negative reads for previously consumed edges, bound to owned history.
    /// These are absence custody, never fabricated consumed relationship values.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub bible_relationship_absence_revisions: Option<Vec<(super::BibleGraphEdgeId, ChangeEventId)>>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub bible_context_scope: Option<super::BibleContextScope>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub script_context_scope: Option<super::ScriptContextScope>,
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn preview_request_round_trips_and_refuses_client_supplied_apply_payloads() {
        let request = RequestScriptImpactProposalCommand {
            proposal_id: PropagationProposalId::new("review.B").unwrap(),
            document_id: ScriptDocumentId::new("main").unwrap(),
            segment_id: ScriptSegmentId::new("segment.B").unwrap(),
            block_id: ScriptBlockId::new("block.B").unwrap(),
            expected_block_revision_event_id: ChangeEventId(uuid::Uuid::new_v4()),
            generation_event_id: ChangeEventId(uuid::Uuid::new_v4()),
            dependency_id: SemanticDependencyId::new("B.input-A").unwrap(),
            story_time_ms: None,
            recall_selection: None,
        };
        let mut json = serde_json::to_value(&request).unwrap();
        assert_eq!(
            serde_json::from_value::<RequestScriptImpactProposalCommand>(json.clone()).unwrap(),
            request
        );
        json["proposed_text"] = serde_json::json!("Client supplied replacement");
        assert!(serde_json::from_value::<RequestScriptImpactProposalCommand>(json).is_err());
    }

    #[test]
    fn recall_selectors_allow_only_identity_and_expected_revision_not_client_fact_values() {
        let fact = ScriptRecallFactSelection {
            node_id: super::super::BibleGraphNodeId::new("Mara").unwrap(),
            part_key: super::super::BibleGraphPartKey::new("profile").unwrap(),
            field_key: super::super::BibleGraphFieldKey::new("tagline").unwrap(),
            field_id: super::super::BibleGraphFieldId::new("Mara.tagline").unwrap(),
            revision_event_id: ChangeEventId(uuid::Uuid::new_v4()),
        };
        let mut json = serde_json::to_value(&fact).unwrap();
        assert_eq!(
            serde_json::from_value::<ScriptRecallFactSelection>(json.clone()).unwrap(),
            fact
        );
        json["value"] = serde_json::json!("Invented fact");
        assert!(serde_json::from_value::<ScriptRecallFactSelection>(json).is_err());
    }
}
