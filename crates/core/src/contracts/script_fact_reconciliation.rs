//! Explicit reconciliation of a saved manual edit with one consumed baseline fact.
use serde::{Deserialize, Serialize};

use super::*;

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct ScriptFactField {
    pub dependency_id: SemanticDependencyId,
    pub node_id: BibleGraphNodeId,
    pub part_key: BibleGraphPartKey,
    pub field_key: BibleGraphFieldKey,
    pub field_id: BibleGraphFieldId,
    pub consumed_revision_event_id: ChangeEventId,
    pub consumed_text: String,
    pub revision_event_id: ChangeEventId,
    pub text: String,
}

/// Read-only evidence derived from canonical edit history and generation lineage.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct ScriptFactEditEvidence {
    pub document_id: ScriptDocumentId,
    pub segment_id: ScriptSegmentId,
    pub block_id: ScriptBlockId,
    pub before_revision_event_id: ChangeEventId,
    pub revision_event_id: ChangeEventId,
    pub segment_revision_event_id: ChangeEventId,
    pub before_text: String,
    pub text: String,
    pub start_ms: u64,
    pub end_ms: u64,
    pub generation_event_id: ChangeEventId,
    pub facts: Vec<ScriptFactField>,
}

/// Identities only: the server owns all text, fact values and eligibility checks.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct RequestScriptFactProposalCommand {
    pub proposal_id: PropagationProposalId,
    pub document_id: ScriptDocumentId,
    pub segment_id: ScriptSegmentId,
    pub block_id: ScriptBlockId,
    pub expected_block_revision_event_id: ChangeEventId,
    pub expected_field_revision_event_id: ChangeEventId,
    pub generation_event_id: ChangeEventId,
    pub dependency_id: SemanticDependencyId,
}

/// Immutable proposal evidence, rechecked under the SQLite writer lock.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct ScriptFactProposalBinding {
    pub request: RequestScriptFactProposalCommand,
    /// Contains only the selected fact; unrelated fields are not analysis inputs.
    pub edit: ScriptFactEditEvidence,
    pub context_dependencies: Vec<SemanticDependency>,
}
