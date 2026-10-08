import type { StoryArcFieldInput } from './storyArcTypes.js';
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
import type { AiBibleContextEdge, BibleFieldInput } from './childPlanningTypes.js';
import type { BibleRecallProjection, BibleRecallRequest } from './bibleRecallTypes.js';

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
  script_fact_binding?: import('./scriptFactTypes.js').ScriptFactProposalBinding | null;
  script_review_binding?: {
    arc_description_applicability_previous?: StoryArcFieldInput[] | null;
    arc_description_applicability_current?: StoryArcFieldInput[] | null;
    timeline_title_previous?: TimelineTitleInput[] | null;
    timeline_title_current?: TimelineTitleInput[] | null;
    timeline_title_absence_revisions?: [string, string][] | null;
    ancestor_notes_previous?: TimelineNotesInput[] | null;
    ancestor_notes_current?: TimelineNotesInput[] | null;
    ancestor_notes_absence_revisions?: [string, string][] | null;
    timeline_notes_previous?: TimelineNotesInput | null;
    timeline_notes_current?: TimelineNotesInput | null;
    arc_inputs?: StoryArcFieldInput[] | null;
    arc_previous_inputs?: StoryArcFieldInput[] | null;
    arc_absence_revisions?: [string, string][] | null;
    request: RequestScriptImpactProposalCommand;
    cause: ScriptImpactCause;
    target_segment_revision_event_id: string;
    script_inputs: ScriptContextBlock[];
    script_previous_inputs?: ScriptContextBlock[] | null;
    bible_context: ProjectionEnvelope<unknown>;
    bible_inputs?: BibleFieldInput[];
    bible_node_name_inputs?: { node_id: string; name: string; revision_event_id: string }[] | null;
    bible_node_name_absence_revisions?: [string, string][] | null;
    bible_relationship_inputs?:
      | {
          edge: AiBibleContextEdge;
          revision_event_id: string;
        }[]
      | null;
    bible_relationship_absence_revisions?: [string, string][] | null;
  } | null;
}

export interface TimelineNotesInput {
  node_id: string;
  notes: string;
  revision_event_id: ChangeEventId | null;
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
  recall_selection?: ScriptRecallSelection | null;
}

export interface ScriptRecallSelection {
  query: BibleRecallRequest;
  facts: {
    node_id: string;
    part_key: string;
    field_key: string;
    field_id: string;
    revision_event_id: string;
  }[];
  names: { node_id: string; name: string; revision_event_id: string }[];
  paths: BibleRecallProjection['paths'];
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

export interface TimelineTitleInput {
  node_id: string;
  name: string;
  revision_event_id: string | null;
}
