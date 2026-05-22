import type { BibleGraphEdgeId, BibleGraphNodeId } from './bibleGraphTypes.js';

export interface GraphRendererStatus {
  running: boolean;
  node_count: number;
  edge_count: number;
  influence_count: number;
  last_error: string | null;
}

export type GraphRendererCommand =
  | { type: 'select_node'; node_id: BibleGraphNodeId }
  | { type: 'select_edge'; edge_id: BibleGraphEdgeId }
  | { type: 'select_influence'; influence_id: string }
  | { type: 'inspect_node'; node_id: BibleGraphNodeId };
