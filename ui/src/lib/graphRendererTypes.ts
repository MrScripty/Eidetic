import type {
  BibleGraphEdgeId,
  BibleGraphNodeId,
  BibleRenderGraphPosition,
} from './bibleGraphTypes.js';

export interface GraphRendererStatus {
  running: boolean;
  native_panel_ready: boolean;
  node_count: number;
  edge_count: number;
  native_visual_node_count: number;
  native_visual_edge_count: number;
  influence_count: number;
  last_error: string | null;
}

export type GraphRendererCommand =
  | { type: 'select_node'; node_id: BibleGraphNodeId }
  | { type: 'select_edge'; edge_id: BibleGraphEdgeId }
  | { type: 'select_influence'; influence_id: string }
  | { type: 'inspect_node'; node_id: BibleGraphNodeId };

export interface GraphRendererVisualSnapshot {
  nodes: GraphRendererVisualNode[];
  edges: GraphRendererVisualEdge[];
}

export interface GraphRendererVisualNode {
  node_id: BibleGraphNodeId;
  label: string;
  position: BibleRenderGraphPosition;
  radius: number;
  fill_color: string;
  outline_color: string;
  highlighted: boolean;
}

export interface GraphRendererVisualEdge {
  edge_id: BibleGraphEdgeId;
  from_node_id: BibleGraphNodeId;
  to_node_id: BibleGraphNodeId;
  from_position: BibleRenderGraphPosition;
  to_position: BibleRenderGraphPosition;
  width: number;
  stroke_color: string;
  highlighted: boolean;
}
