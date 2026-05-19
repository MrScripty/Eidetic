import type {
  BibleGraphNodeId,
  BibleRenderGraphNeighborhood,
  BibleRenderGraphProjection,
} from '$lib/bibleGraphTypes.js';

export interface BibleRenderGraphOutlineItem {
  node_id: BibleGraphNodeId;
  label: string;
  depth: number;
  connected_node_count: number;
  edge_count: number;
  selected: boolean;
}

export function bibleRenderGraphOutlineItems(
  projection: BibleRenderGraphProjection,
  selectedNodeId: BibleGraphNodeId | null,
  query = '',
): BibleRenderGraphOutlineItem[] {
  const normalizedQuery = query.trim().toLowerCase();
  const neighborhoods = new Map<BibleGraphNodeId, BibleRenderGraphNeighborhood>(
    projection.neighborhoods.map((neighborhood) => [neighborhood.node_id, neighborhood]),
  );

  return projection.nodes
    .filter((node) => !normalizedQuery || node.label.toLowerCase().includes(normalizedQuery))
    .map((node) => {
      const neighborhood = neighborhoods.get(node.node_id);
      return {
        node_id: node.node_id,
        label: node.label,
        depth: node.depth,
        connected_node_count: neighborhood?.connected_node_ids.length ?? 0,
        edge_count: neighborhood?.edge_ids.length ?? 0,
        selected: node.node_id === selectedNodeId,
      };
    });
}
