import type { BibleGraphEdgeId, BibleGraphNodeId } from './bibleGraphTypes.js';
import type { NodeId, StoryLevel } from './timelineTypes.js';

export type ContextEvaluationId = string;
export type ContextInfluenceId = string;

export interface ContextStackProjection {
  target_node_id: NodeId;
  layers: ContextStackLayer[];
}

export interface ContextStackLayer {
  node_id: NodeId;
  level: StoryLevel;
  label: string;
  role: ContextLayerRole;
  distilled_context?: string | null;
  sort_order: number;
}

export type ContextLayerRole = 'target' | 'inherited' | 'sibling';

export interface ContextInfluenceProjection {
  target_node_id: NodeId;
  evaluation_id: ContextEvaluationId;
  records: ContextInfluenceRecord[];
}

export interface ContextInfluenceRecord {
  id: ContextInfluenceId;
  evaluation_id: ContextEvaluationId;
  timeline_node_id: NodeId;
  source_layer: StoryLevel;
  influence_kind: ContextInfluenceKind;
  confidence: number;
  reason: string;
  provenance: ContextInfluenceProvenance;
  bible_node_id?: BibleGraphNodeId | null;
  bible_edge_id?: BibleGraphEdgeId | null;
  introduced_by_node_id?: NodeId | null;
  sort_order: number;
}

export type ContextInfluenceKind = 'direct' | 'inherited' | 'candidate' | 'ignored' | 'rejected';

export type ContextInfluenceProvenance =
  | 'user_selected'
  | 'ai_selected'
  | 'parent_context'
  | 'graph_traversal'
  | 'proposal';
