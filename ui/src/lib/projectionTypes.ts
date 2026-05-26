export type CommandId = string;
export type ChangeEventId = string;

export type ObjectKind =
  | 'project'
  | 'timeline_node'
  | 'timeline_track'
  | 'timeline_relationship'
  | 'story_arc'
  | 'bible_node'
  | 'bible_part'
  | 'bible_part_field'
  | 'bible_edge'
  | 'bible_snapshot'
  | 'script_document'
  | 'script_segment'
  | 'script_block'
  | 'script_span'
  | 'script_lock'
  | 'affect_value'
  | 'affect_dependency'
  | 'semantic_proposal'
  | 'semantic_claim'
  | 'semantic_dependency'
  | 'reference_asset'
  | 'projection';

export type FieldValue =
  | { type: 'text'; value: string }
  | { type: 'integer'; value: number }
  | { type: 'number'; value: number }
  | { type: 'bool'; value: boolean }
  | { type: 'object_ref'; value: { kind: ObjectKind; id: string } }
  | { type: 'asset_ref'; value: string };

export interface CommandEnvelope<TPayload> {
  id: CommandId;
  payload: TPayload;
}

export interface ProjectionEnvelope<TPayload> {
  version: number;
  change_event_id?: ChangeEventId;
  payload: TPayload;
}

export interface SetObjectFieldCommand {
  object_kind: ObjectKind;
  object_id: string;
  field_key: string;
  value?: FieldValue | null;
}

export interface ObjectFieldProjection {
  object_kind: ObjectKind;
  object_id: string;
  deleted: boolean;
  fields: Record<string, FieldValue>;
}

export type CommandOutcome = 'recorded' | 'already_recorded';

export interface ObjectFieldCommandResponse {
  outcome: CommandOutcome;
  projection: ProjectionEnvelope<ObjectFieldProjection>;
}
