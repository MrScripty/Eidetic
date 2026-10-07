export interface ScriptFactField {
  dependency_id: string;
  node_id: string;
  part_key: string;
  field_key: string;
  field_id: string;
  consumed_revision_event_id: string;
  consumed_text: string;
  revision_event_id: string;
  text: string;
}
export interface ScriptFactEditEvidence {
  document_id: string;
  segment_id: string;
  block_id: string;
  before_revision_event_id: string;
  revision_event_id: string;
  segment_revision_event_id: string;
  before_text: string;
  text: string;
  start_ms: number;
  end_ms: number;
  generation_event_id: string;
  facts: ScriptFactField[];
}
export interface RequestScriptFactProposalCommand {
  proposal_id: string;
  document_id: string;
  segment_id: string;
  block_id: string;
  expected_block_revision_event_id: string;
  expected_field_revision_event_id: string;
  generation_event_id: string;
  dependency_id: string;
}
export interface ScriptFactProposalBinding {
  request: RequestScriptFactProposalCommand;
  edit: ScriptFactEditEvidence;
  context_dependencies: {
    id: string;
    target: import('./scriptTypes.js').ScriptImpactCause['input'];
    revision_binding?: {
      source_revision_event_id: string;
      target_revision_event_id: string;
    } | null;
  }[];
}
