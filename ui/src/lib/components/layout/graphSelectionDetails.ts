import type {
  BibleGraphEdgeId,
  BibleGraphNodeId,
  BibleRenderGraphEdge,
  BibleRenderGraphInfluence,
  BibleRenderGraphNeighborhood,
  BibleRenderGraphNode,
  BibleRenderGraphProjection,
} from '$lib/bibleGraphTypes.js';
import type { BibleGraphSelection } from '$lib/stores/bible.svelte.js';

export type GraphSelectionDetail =
  | { kind: 'edge'; edge: BibleRenderGraphEdge; fromLabel: string; toLabel: string }
  | {
      kind: 'influence';
      influence: BibleRenderGraphInfluence;
      nodeLabel?: string;
      edgeLabel?: string;
    }
  | { kind: 'context_layer'; timelineNodeId: string; influenceCount: number }
  | {
      kind: 'neighborhood';
      node: BibleRenderGraphNode;
      neighborhood?: BibleRenderGraphNeighborhood;
      connectedLabels: string[];
    };

export function graphSelectionDetail(
  projection: BibleRenderGraphProjection | null,
  selection: BibleGraphSelection,
): GraphSelectionDetail | null {
  if (!projection) return null;

  const nodesById = new Map<BibleGraphNodeId, BibleRenderGraphNode>(
    projection.nodes.map((node) => [node.node_id, node]),
  );
  const edgesById = new Map<BibleGraphEdgeId, BibleRenderGraphEdge>(
    projection.edges.map((edge) => [edge.edge_id, edge]),
  );

  switch (selection.kind) {
    case 'edge': {
      const edge = edgesById.get(selection.edgeId);
      if (!edge) return null;

      return {
        kind: 'edge',
        edge,
        fromLabel: nodesById.get(edge.from_node_id)?.label ?? edge.from_node_id,
        toLabel: nodesById.get(edge.to_node_id)?.label ?? edge.to_node_id,
      };
    }
    case 'influence': {
      const influence = projection.influences.find(
        (candidate) => candidate.influence_id === selection.influenceId,
      );
      if (!influence) return null;

      return {
        kind: 'influence',
        influence,
        nodeLabel: influence.bible_node_id
          ? (nodesById.get(influence.bible_node_id)?.label ?? influence.bible_node_id)
          : undefined,
        edgeLabel: influence.bible_edge_id
          ? (edgesById.get(influence.bible_edge_id)?.label ?? influence.bible_edge_id)
          : undefined,
      };
    }
    case 'context_layer': {
      return {
        kind: 'context_layer',
        timelineNodeId: selection.timelineNodeId,
        influenceCount: projection.influences.filter(
          (influence) => influence.timeline_node_id === selection.timelineNodeId,
        ).length,
      };
    }
    case 'neighborhood': {
      const node = nodesById.get(selection.nodeId);
      if (!node) return null;

      const neighborhood = projection.neighborhoods.find(
        (candidate) => candidate.node_id === selection.nodeId,
      );

      return {
        kind: 'neighborhood',
        node,
        neighborhood,
        connectedLabels:
          neighborhood?.connected_node_ids.map(
            (nodeId) => nodesById.get(nodeId)?.label ?? nodeId,
          ) ?? [],
      };
    }
    case 'none':
    case 'node':
      return null;
  }
}
