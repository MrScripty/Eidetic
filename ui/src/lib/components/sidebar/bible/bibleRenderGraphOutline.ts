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
  influence_count: number;
  active: boolean;
  selected: boolean;
}

export function bibleRenderGraphOutlineItems(
  projection: BibleRenderGraphProjection,
  selectedNodeId: BibleGraphNodeId | null = projection.selected_node_id ?? null,
  query = '',
): BibleRenderGraphOutlineItem[] {
  const projectedSelectedNodeId = selectedNodeId ?? projection.selected_node_id ?? null;
  const normalizedQuery = query.trim().toLowerCase();
  const neighborhoods = new Map<BibleGraphNodeId, BibleRenderGraphNeighborhood>(
    projection.neighborhoods.map((neighborhood) => [neighborhood.node_id, neighborhood]),
  );
  const influenceCounts = new Map<BibleGraphNodeId, number>();
  for (const influence of projection.influences) {
    if (!influence.bible_node_id) continue;
    influenceCounts.set(
      influence.bible_node_id,
      (influenceCounts.get(influence.bible_node_id) ?? 0) + 1,
    );
  }

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
        influence_count: influenceCounts.get(node.node_id) ?? 0,
        active: influenceCounts.has(node.node_id),
        selected: node.node_id === projectedSelectedNodeId,
      };
    });
}
