import type { BibleGraphNodeId, BibleGraphSchemaKey } from './bibleGraphTypes.js';
import type { CommandOutcome, ProjectionEnvelope } from './projectionTypes.js';
import type { NodeId } from './timelineTypes.js';

export type SemanticProposalId = string;
export type BibleReferenceKind = 'character' | 'location' | 'prop';
export type SemanticProposalStatus = 'pending' | 'accepted' | 'rejected';

export interface BibleReferenceProposal {
  id: SemanticProposalId;
  source_node_id: NodeId;
  child_name: string;
  reference_kind: BibleReferenceKind;
  reference_text: string;
  proposed_schema_key: BibleGraphSchemaKey;
  status: SemanticProposalStatus;
  rationale?: string | null;
  created_at_ms: number;
}

export interface CreateBibleReferenceProposalCommand {
  proposal_id: SemanticProposalId;
  source_node_id: NodeId;
  child_name: string;
  reference_kind: BibleReferenceKind;
  reference_text: string;
  rationale?: string | null;
}

export interface RejectBibleReferenceProposalCommand {
  proposal_id: SemanticProposalId;
  reason?: string | null;
}

export interface AcceptBibleReferenceProposalCommand {
  proposal_id: SemanticProposalId;
  node_id: BibleGraphNodeId;
  parent_id?: BibleGraphNodeId | null;
  name?: string | null;
  sort_order?: number;
}

export interface BibleReferenceProposalListProjection {
  proposals: BibleReferenceProposal[];
}

export interface BibleReferenceProposalCommandResponse {
  outcome: CommandOutcome;
  projection: ProjectionEnvelope<BibleReferenceProposalListProjection>;
}
