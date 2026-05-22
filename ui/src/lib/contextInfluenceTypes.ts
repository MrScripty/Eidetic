import type { BibleGraphEdgeId, BibleGraphNodeId } from './bibleGraphTypes.js';
import type { NodeId, StoryLevel } from './timelineTypes.js';

export type ContextEvaluationId = string;
export type ContextInfluenceId = string;

export interface RecordContextEvaluationCommand {
  evaluation: ContextEvaluation;
  influences: ContextInfluenceRecord[];
}

export interface ContextEvaluation {
  id: ContextEvaluationId;
  target_node_id: NodeId;
  task_kind: ContextEvaluationTaskKind;
  summary: string;
  distilled_context?: string | null;
  created_at_ms: number;
}

export type ContextEvaluationTaskKind =
  | 'generate_timeline_context'
  | 'generate_script'
  | 'inspect_context';

export interface ContextStackProjection {
  target_node_id: NodeId;
  layers: ContextStackLayer[];
}

export interface ContextStackProjectionRequest {
  target_node_id: NodeId;
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
  task_kind: ContextEvaluationTaskKind;
  distilled_context?: string | null;
  records: ContextInfluenceRecord[];
}

export interface ContextInfluenceProjectionRequest {
  target_node_id: NodeId;
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
