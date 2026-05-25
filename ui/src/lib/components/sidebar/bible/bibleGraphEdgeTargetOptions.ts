import type {
  BibleGraphNode,
  BibleGraphNodeId,
  BibleRenderGraphNode,
} from '$lib/bibleGraphTypes.js';

export interface BibleGraphEdgeTargetOption {
  nodeId: BibleGraphNodeId;
  label: string;
  schemaKey: string;
}

export function bibleGraphEdgeTargetOptions(
  nodes: Array<BibleGraphNode | BibleRenderGraphNode>,
  sourceNodeId: BibleGraphNodeId,
): BibleGraphEdgeTargetOption[] {
  return nodes
    .filter((node) => nodeId(node) !== sourceNodeId)
    .map((node) => ({
      nodeId: nodeId(node),
      label: nodeLabel(node),
      schemaKey: node.schema_key,
    }))
    .sort((left, right) => {
      const labelOrder = left.label.localeCompare(right.label);
      return labelOrder === 0 ? left.nodeId.localeCompare(right.nodeId) : labelOrder;
    });
}

function nodeId(node: BibleGraphNode | BibleRenderGraphNode): BibleGraphNodeId {
  return 'node_id' in node ? node.node_id : node.id;
}

function nodeLabel(node: BibleGraphNode | BibleRenderGraphNode): string {
  return 'label' in node ? node.label : node.name;
}
