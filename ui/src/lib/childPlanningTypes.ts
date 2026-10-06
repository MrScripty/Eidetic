import type { BeatType, NodeId, StoryLevel } from './timelineTypes.js';
import type { ScriptContextBlock } from './scriptTypes.js';
import type {
  BibleGraphNodeId,
  BibleGraphPartKey,
  BibleGraphFieldKey,
  BibleGraphFieldId,
  BibleGraphSchemaKey,
  BibleGraphEdgeId,
  BibleGraphEdgeKind,
} from './bibleGraphTypes.js';
import type { ChangeEventId, FieldValue, ProjectionEnvelope } from './projectionTypes.js';

export interface BibleFieldInput {
  node_id: BibleGraphNodeId;
  part_key: BibleGraphPartKey;
  field_key: BibleGraphFieldKey;
  field_id: BibleGraphFieldId;
  revision_event_id: ChangeEventId;
  value: FieldValue;
}

export interface AiBibleContextField {
  part_key: BibleGraphPartKey;
  part_name: string;
  field_key: BibleGraphFieldKey;
  value: FieldValue;
}

export interface AiBibleContextEdge {
  edge_id: BibleGraphEdgeId;
  from_node_id: BibleGraphNodeId;
  to_node_id: BibleGraphNodeId;
  edge_kind: BibleGraphEdgeKind;
  label: string;
  directed: boolean;
}

export interface AiBibleContextNode {
  node_id: BibleGraphNodeId;
  parent_id?: BibleGraphNodeId | null;
  schema_key: BibleGraphSchemaKey;
  name: string;
  fields: AiBibleContextField[];
  snapshots: { label: string; at_ms: number; fields: AiBibleContextField[] }[];
  unresolved_timed_fields?: { part_key: BibleGraphPartKey; field_key: BibleGraphFieldKey }[];
  incoming_edges: AiBibleContextEdge[];
  outgoing_edges: AiBibleContextEdge[];
}

export interface AiBibleContextProjection {
  target_node_id: NodeId;
  story_time_ms?: number | null;
  nodes: AiBibleContextNode[];
}

export interface ChildPlanBibleContext {
  context: ProjectionEnvelope<AiBibleContextProjection>;
  inputs: BibleFieldInput[];
}

export interface ChildProposal {
  name: string;
  beat_type: BeatType | null;
  outline: string;
  weight: number;
  characters?: string[];
  location?: string | null;
  props?: string[];
}

export interface ChildPlan {
  id: string;
  parent_node_id: NodeId;
  target_child_level: StoryLevel;
  children: ChildProposal[];
  script_context?: ScriptContextBlock[] | null;
  bible_context?: ChildPlanBibleContext | null;
}

export interface ChildPlanRecord {
  plan: ChildPlan;
  status: 'pending' | 'applied' | 'rejected';
  created_at_ms: number;
}

export interface ChildPlanListProjection {
  plans: ChildPlanRecord[];
}
