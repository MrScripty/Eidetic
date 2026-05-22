import type {
  BibleGraphEdgeId,
  BibleGraphNodeId,
  BibleRenderGraphProjection,
} from '$lib/bibleGraphTypes.js';

export interface GraphWorkspaceEdgeItem {
  edgeId: BibleGraphEdgeId;
  label: string;
  fromLabel: string;
  toLabel: string;
  directed: boolean;
}

export interface GraphWorkspaceNeighborhoodItem {
  nodeId: BibleGraphNodeId;
  label: string;
  connectedNodeCount: number;
  edgeCount: number;
}

export function graphWorkspaceEdgeItems(
  projection: BibleRenderGraphProjection,
): GraphWorkspaceEdgeItem[] {
  const nodeLabels = new Map(projection.nodes.map((node) => [node.node_id, node.label]));

  return [...projection.edges]
    .sort((a, b) => a.sort_order - b.sort_order || a.edge_id.localeCompare(b.edge_id))
    .map((edge) => ({
      edgeId: edge.edge_id,
      label: edge.label,
      fromLabel: nodeLabels.get(edge.from_node_id) ?? edge.from_node_id,
      toLabel: nodeLabels.get(edge.to_node_id) ?? edge.to_node_id,
      directed: edge.directed,
    }));
}

export function graphWorkspaceNeighborhoodItems(
  projection: BibleRenderGraphProjection,
): GraphWorkspaceNeighborhoodItem[] {
  const nodeLabels = new Map(projection.nodes.map((node) => [node.node_id, node.label]));

  return [...projection.neighborhoods]
    .sort((a, b) => {
      const aLabel = nodeLabels.get(a.node_id) ?? a.node_id;
      const bLabel = nodeLabels.get(b.node_id) ?? b.node_id;
      return aLabel.localeCompare(bLabel);
    })
    .map((neighborhood) => ({
      nodeId: neighborhood.node_id,
      label: nodeLabels.get(neighborhood.node_id) ?? neighborhood.node_id,
      connectedNodeCount: neighborhood.connected_node_ids.length,
      edgeCount: neighborhood.edge_ids.length,
    }));
}
