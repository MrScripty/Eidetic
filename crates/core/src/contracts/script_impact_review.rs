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
}

/// Canonical evidence captured before provider I/O and rechecked at acceptance.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct ScriptImpactProposalBinding {
    pub request: RequestScriptImpactProposalCommand,
    pub cause: ScriptImpactCause,
    pub target_segment_revision_event_id: ChangeEventId,
    pub script_inputs: Vec<ScriptContextBlock>,
    pub bible_context: ProjectionEnvelope<AiBibleContextProjection>,
    #[serde(default)]
    pub bible_inputs: Vec<super::BibleFieldInput>,
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
        };
        let mut json = serde_json::to_value(&request).unwrap();
        assert_eq!(
            serde_json::from_value::<RequestScriptImpactProposalCommand>(json.clone()).unwrap(),
            request
        );
        json["proposed_text"] = serde_json::json!("Client supplied replacement");
        assert!(serde_json::from_value::<RequestScriptImpactProposalCommand>(json).is_err());
    }
}
