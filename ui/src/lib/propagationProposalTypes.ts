import type {
  BibleGraphFieldKey,
  BibleGraphNodeId,
  BibleGraphPartKey,
  BibleGraphSnapshotFieldId,
  BibleGraphSnapshotId,
} from './bibleGraphTypes.js';
import type { CommandOutcome, FieldValue, ProjectionEnvelope } from './projectionTypes.js';
import type { ScriptBlockId, ScriptPatch, ScriptSegmentId } from './scriptTypes.js';
import type { ScriptContextBlock, ScriptImpactCause } from './scriptTypes.js';
import type { SemanticProposalStatus } from './semanticProposalTypes.js';
import type { AiBibleContextEdge } from './childPlanningTypes.js';

export type PropagationProposalId = string;
export type BibleGraphFieldId = string;
export type ChangeEventId = string;
export type SemanticDependencyId = string;

export type PropagationProposalAction =
  | 'set_bible_field'
  | 'set_bible_snapshot_field'
  | 'patch_script_block'
  | 'regenerate_script_segment';

export type PropagationProposalTarget =
  | {
      kind: 'bible_field';
      node_id: BibleGraphNodeId;
      part_key: BibleGraphPartKey;
      field_key: BibleGraphFieldKey;
      field_id?: BibleGraphFieldId | null;
    }
  | {
      kind: 'bible_snapshot_field';
      node_id: BibleGraphNodeId;
      snapshot_id: BibleGraphSnapshotId;
      part_key: BibleGraphPartKey;
      field_key: BibleGraphFieldKey;
      field_id: BibleGraphSnapshotFieldId;
    }
  | {
      kind: 'script_block';
      block_id: ScriptBlockId;
    }
  | {
      kind: 'script_segment';
      segment_id: ScriptSegmentId;
    };

export interface PropagationProposal {
  id: PropagationProposalId;
  action: PropagationProposalAction;
  target: PropagationProposalTarget;
  status: SemanticProposalStatus;
  summary: string;
  proposed_value?: FieldValue | null;
  proposed_text?: string | null;
  proposed_script_patch?: ScriptPatch | null;
  source_dependency_id?: SemanticDependencyId | null;
  source_event_id?: ChangeEventId | null;
  rationale?: string | null;
  created_at_ms: number;
  script_review_binding?: {
    request: RequestScriptImpactProposalCommand;
    cause: ScriptImpactCause;
    target_segment_revision_event_id: string;
    script_inputs: ScriptContextBlock[];
    bible_context: ProjectionEnvelope<unknown>;
    bible_relationship_inputs?:
      | {
          edge: AiBibleContextEdge;
          revision_event_id: string;
        }[]
      | null;
  } | null;
}

export interface RequestScriptImpactProposalCommand {
  proposal_id: string;
  document_id: string;
  segment_id: string;
  block_id: string;
  expected_block_revision_event_id: string;
  generation_event_id: string;
  dependency_id: string;
  story_time_ms?: number | null;
}

export interface CreatePropagationProposalCommand {
  proposal_id: PropagationProposalId;
  action: PropagationProposalAction;
  target: PropagationProposalTarget;
  summary: string;
  proposed_value?: FieldValue | null;
  proposed_text?: string | null;
  proposed_script_patch?: ScriptPatch | null;
  source_dependency_id?: SemanticDependencyId | null;
  source_event_id?: ChangeEventId | null;
  rationale?: string | null;
}

export interface UpdatePropagationProposalCommand {
  proposal_id: PropagationProposalId;
  action: PropagationProposalAction;
  target: PropagationProposalTarget;
  summary: string;
  proposed_value?: FieldValue | null;
  proposed_text?: string | null;
  proposed_script_patch?: ScriptPatch | null;
  source_dependency_id?: SemanticDependencyId | null;
  source_event_id?: ChangeEventId | null;
  rationale?: string | null;
}

export interface RejectPropagationProposalCommand {
  proposal_id: PropagationProposalId;
  reason?: string | null;
}

export interface AcceptPropagationProposalCommand {
  proposal_id: PropagationProposalId;
}

export interface PropagationProposalListProjection {
  proposals: PropagationProposal[];
}

export interface PropagationProposalCommandResponse {
  outcome: CommandOutcome;
  projection: ProjectionEnvelope<PropagationProposalListProjection>;
}
