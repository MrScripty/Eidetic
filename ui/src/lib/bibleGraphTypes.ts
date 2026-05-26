import type { CommandOutcome, FieldValue, ProjectionEnvelope } from './projectionTypes.js';

export type BibleGraphNodeId = string;
export type BibleGraphPartId = string;
export type BibleGraphFieldId = string;
export type BibleGraphEdgeId = string;
export type BibleGraphSnapshotId = string;
export type BibleGraphSnapshotFieldId = string;
export type BibleGraphSchemaKey = string;
export type BibleGraphPartKey = string;
export type BibleGraphFieldKey = string;
export type BibleGraphNodeCategory =
  | 'character'
  | 'location'
  | 'prop'
  | 'culture'
  | 'theme'
  | 'event'
  | 'rule'
  | 'reference'
  | 'canonical'
  | 'other';

export interface CreateBibleGraphNodeCommand {
  node_id?: BibleGraphNodeId;
  parent_id?: BibleGraphNodeId | null;
  schema_key: BibleGraphSchemaKey;
  name: string;
  sort_order?: number;
}

export interface SetBibleGraphFieldCommand {
  node_id: BibleGraphNodeId;
  part_id: BibleGraphPartId;
  part_key: BibleGraphPartKey;
  part_name: string;
  part_sort_order: number;
  field_id: BibleGraphFieldId;
  field_key: BibleGraphFieldKey;
  value?: FieldValue | null;
  field_sort_order: number;
}

export interface SetBibleGraphEdgeCommand {
  edge_id?: BibleGraphEdgeId;
  from_node_id: BibleGraphNodeId;
  to_node_id: BibleGraphNodeId;
  edge_kind: BibleGraphEdgeKind;
  label: string;
  directed?: boolean;
  sort_order?: number;
}

export interface SetBibleGraphSnapshotFieldCommand {
  snapshot_id?: BibleGraphSnapshotId;
  node_id: BibleGraphNodeId;
  at_ms: number;
  label: string;
  snapshot_sort_order?: number;
  field_id?: BibleGraphSnapshotFieldId;
  part_key: BibleGraphPartKey;
  part_name: string;
  field_key: BibleGraphFieldKey;
  value?: FieldValue | null;
  field_sort_order?: number;
}

export interface EnsureCanonicalBibleRootsCommand {}

export interface BibleGraphNode {
  id: BibleGraphNodeId;
  parent_id?: BibleGraphNodeId | null;
  schema_key: BibleGraphSchemaKey;
  name: string;
  system_owned: boolean;
  sort_order: number;
}

export interface BibleGraphPart {
  id: BibleGraphPartId;
  node_id: BibleGraphNodeId;
  part_key: BibleGraphPartKey;
  name: string;
  system_owned: boolean;
  sort_order: number;
}

export interface BibleGraphField {
  id: BibleGraphFieldId;
  part_id: BibleGraphPartId;
  field_key: BibleGraphFieldKey;
  value?: FieldValue | null;
  sort_order: number;
}

export type BibleGraphEdgeKind =
  | 'references'
  | 'located_in'
  | 'owns'
  | 'member_of'
  | 'conflicts_with'
  | 'supports_theme'
  | { custom: string };

export interface BibleGraphEdge {
  id: BibleGraphEdgeId;
  from_node_id: BibleGraphNodeId;
  to_node_id: BibleGraphNodeId;
  edge_kind: BibleGraphEdgeKind;
  label: string;
  directed: boolean;
  sort_order: number;
}

export interface BibleGraphSnapshot {
  id: BibleGraphSnapshotId;
  node_id: BibleGraphNodeId;
  at_ms: number;
  label: string;
  sort_order: number;
}

export interface BibleGraphSnapshotField {
  id: BibleGraphSnapshotFieldId;
  snapshot_id: BibleGraphSnapshotId;
  part_key: BibleGraphPartKey;
  part_name: string;
  field_key: BibleGraphFieldKey;
  value?: FieldValue | null;
  sort_order: number;
}

export interface BibleGraphPartProjection {
  part: BibleGraphPart;
  fields: BibleGraphField[];
}

export interface BibleGraphSnapshotProjection {
  snapshot: BibleGraphSnapshot;
  fields: BibleGraphSnapshotField[];
}

export interface BibleNodeDetailProjection {
  node: BibleGraphNode;
  parts: BibleGraphPartProjection[];
  incoming_edges: BibleGraphEdge[];
  outgoing_edges: BibleGraphEdge[];
  snapshots: BibleGraphSnapshotProjection[];
}

export interface BibleGraphNodeListProjection {
  nodes: BibleGraphNode[];
}

export interface BibleRenderGraphProjectionRequest {
  focused_root_id?: BibleGraphNodeId | null;
  selected_node_id?: BibleGraphNodeId | null;
  selected_timeline_node_id?: string | null;
  active_timeline_ms?: number | null;
  search?: string | null;
  edge_kinds?: BibleGraphEdgeKind[];
  neighborhood_depth?: number;
  max_nodes?: number;
  max_edges?: number;
}

export interface BibleRenderGraphProjection {
  focused_root_id?: BibleGraphNodeId | null;
  selected_node_id?: BibleGraphNodeId | null;
  selected_timeline_node_id?: string | null;
  active_timeline_ms?: number | null;
  nodes: BibleRenderGraphNode[];
  edges: BibleRenderGraphEdge[];
  neighborhoods: BibleRenderGraphNeighborhood[];
  influences: BibleRenderGraphInfluence[];
}

export interface BibleRenderGraphNode {
  node_id: BibleGraphNodeId;
  parent_id?: BibleGraphNodeId | null;
  schema_key: BibleGraphSchemaKey;
  category: BibleGraphNodeCategory;
  label: string;
  system_owned: boolean;
  sort_order: number;
  depth: number;
  position: BibleRenderGraphPosition;
}

export interface BibleRenderGraphPosition {
  x: number;
  y: number;
  z: number;
}

export interface BibleRenderGraphEdge {
  edge_id: BibleGraphEdgeId;
  from_node_id: BibleGraphNodeId;
  to_node_id: BibleGraphNodeId;
  edge_kind: BibleGraphEdgeKind;
  label: string;
  directed: boolean;
  sort_order: number;
}

export interface BibleRenderGraphNeighborhood {
  node_id: BibleGraphNodeId;
  connected_node_ids: BibleGraphNodeId[];
  edge_ids: BibleGraphEdgeId[];
}

export interface BibleRenderGraphInfluence {
  influence_id: string;
  timeline_node_id: string;
  source_layer: string;
  influence_kind: string;
  confidence: number;
  reason: string;
  provenance: string;
  bible_node_id?: BibleGraphNodeId | null;
  bible_edge_id?: BibleGraphEdgeId | null;
  sort_order: number;
}

export interface BibleGraphNodeCommandResponse {
  outcome: CommandOutcome;
  projection: ProjectionEnvelope<BibleNodeDetailProjection>;
}

export interface BibleGraphRootsCommandResponse {
  outcome: CommandOutcome;
  projection: ProjectionEnvelope<BibleGraphNodeListProjection>;
}
