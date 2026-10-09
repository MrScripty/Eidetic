import type { BibleGraphEdgeKind } from './bibleGraphTypes.js';
import type { ChangeEventId, FieldValue } from './projectionTypes.js';

export interface BibleRecallRequest {
  anchor_node_id: string;
  story_time_ms: number | null;
  direction: 'incoming' | 'outgoing' | 'both';
  edge_kinds: BibleGraphEdgeKind[];
  neighbor_limit: number;
}
export type BibleRecallFieldSource =
  | { kind: 'baseline'; field_id: string; revision_event_id: ChangeEventId }
  | {
      kind: 'snapshot';
      snapshot_id: string;
      snapshot_field_id: string;
      snapshot_revision_event_id: ChangeEventId;
      field_revision_event_id: ChangeEventId;
      at_ms: number;
      label: string;
    };
export interface BibleRecallNode {
  node_id: string;
  name: string;
  schema_key: string;
  name_revision_event_id: ChangeEventId | null;
  fields: {
    part_key: string;
    field_key: string;
    value: FieldValue;
    source: BibleRecallFieldSource;
  }[];
  unresolved_timed_fields: { part_key: string; field_key: string }[];
  omitted_fields: number;
}
export interface BibleRecallProjection {
  request: BibleRecallRequest;
  nodes: BibleRecallNode[];
  paths: {
    neighbor_node_id: string;
    relationship: {
      revision_event_id: ChangeEventId;
      edge: {
        edge_id: string;
        from_node_id: string;
        to_node_id: string;
        edge_kind: BibleGraphEdgeKind;
        label: string;
        directed: boolean;
      };
    };
  }[];
  omitted_neighbors: number;
  omitted_edges: number;
  relationships_untimed: boolean;
}
