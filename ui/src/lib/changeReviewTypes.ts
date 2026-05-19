import type {
  ChangeEventId,
  CommandId,
  FieldValue,
  ObjectKind,
  ProjectionEnvelope,
} from './projectionTypes.js';

export type ObjectRevisionId = string;

export type ChangeEventKind =
  | 'user_edit'
  | 'ai_proposal_created'
  | 'ai_proposal_accepted'
  | 'ai_proposal_rejected'
  | 'propagation'
  | 'undo'
  | 'redo'
  | 'import'
  | 'system_repair';

export type RevisionOperation = 'create' | 'update' | 'delete';

export interface ChangeEvent {
  id: ChangeEventId;
  command_id: CommandId;
  kind: ChangeEventKind;
  summary: string;
  created_at_ms: number;
}

export interface FieldDelta {
  field_key: string;
  old_value?: FieldValue | null;
  new_value?: FieldValue | null;
  sort_order: number;
}

export interface ObjectRevision {
  id: ObjectRevisionId;
  object_kind: ObjectKind;
  object_id: string;
  change_event_id: ChangeEventId;
  base_revision_id?: ObjectRevisionId | null;
  operation: RevisionOperation;
  fields: FieldDelta[];
}

export interface ChangeReviewChange {
  event: ChangeEvent;
  revisions: ObjectRevision[];
}

export interface ChangeReviewProjection {
  changes: ChangeReviewChange[];
}

export type ChangeReviewProjectionEnvelope = ProjectionEnvelope<ChangeReviewProjection>;
