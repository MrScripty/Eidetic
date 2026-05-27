import type { BibleGraphEdgeId, BibleGraphNodeId } from '$lib/bibleGraphTypes.js';
import type { GraphRendererCommand } from '$lib/graphRendererTypes.js';
import { categorySchema, newNodeName } from '$lib/components/sidebar/bible/bibleGraphCategories.js';
import {
  clearBibleGraphSelection,
  focusBibleGraphNeighborhood,
  selectBibleGraphEdge,
  selectBibleGraphInfluence,
  selectBibleGraphNode,
} from './bible.svelte.js';
import {
  createBibleGraphNodeProjection,
  deleteBibleGraphNodeProjection,
} from './bibleGraphNodeProjection.svelte.js';
import {
  getCachedBibleGraphSchemaListProjection,
  refreshBibleGraphSchemaListProjection,
} from './bibleGraphSchemaProjection.svelte.js';
import {
  getActiveBibleRenderGraphProjectionRequest,
  getCachedBibleRenderGraphProjection,
  refreshBibleRenderGraphProjection,
} from './bibleRenderGraphProjection.svelte.js';

export interface GraphRendererSelectionTarget {
  selectNode(nodeId: BibleGraphNodeId): void;
  selectEdge(edgeId: BibleGraphEdgeId): void;
  selectInfluence(influenceId: string): void;
  inspectNode(nodeId: BibleGraphNodeId): void;
  focusNode(nodeId: BibleGraphNodeId): void;
  navigateToNode(nodeId: BibleGraphNodeId): void;
  deleteNode(nodeId: BibleGraphNodeId): void | Promise<void>;
  createConnectedNode(parentId: BibleGraphNodeId): void | Promise<void>;
  clearSelection(): void;
}

export const bibleGraphRendererSelectionTarget: GraphRendererSelectionTarget = {
  selectNode: selectBibleGraphNode,
  selectEdge: selectBibleGraphEdge,
  selectInfluence: selectBibleGraphInfluence,
  inspectNode: selectBibleGraphNode,
  focusNode: focusBibleGraphNeighborhood,
  navigateToNode: selectBibleGraphNode,
  deleteNode: deleteBibleGraphNodeFromRenderer,
  createConnectedNode: createConnectedBibleGraphNodeFromRenderer,
  clearSelection: clearBibleGraphSelection,
};

async function deleteBibleGraphNodeFromRenderer(nodeId: BibleGraphNodeId): Promise<void> {
  await deleteBibleGraphNodeProjection(nodeId);
  clearBibleGraphSelection();
  await refreshBibleRenderGraphProjection(getActiveBibleRenderGraphProjectionRequest());
}

async function createConnectedBibleGraphNodeFromRenderer(
  parentId: BibleGraphNodeId,
): Promise<void> {
  const renderProjection = getCachedBibleRenderGraphProjection()?.payload;
  const parent = renderProjection?.nodes.find((node) => node.node_id === parentId);
  const category = parent && parent.category !== 'canonical' ? parent.category : 'other';
  const schemaProjection =
    getCachedBibleGraphSchemaListProjection()?.payload ??
    (await refreshBibleGraphSchemaListProjection()).payload;
  const schema = categorySchema(category, schemaProjection);
  if (!schema) {
    throw new Error(`Schema unavailable for ${category}`);
  }

  await createBibleGraphNodeProjection({
    parent_id: parentId,
    schema_key: schema.schema_key,
    name: newNodeName(category, schemaProjection),
    sort_order: renderProjection?.nodes.filter((node) => node.parent_id === parentId).length ?? 0,
  });
  await refreshBibleRenderGraphProjection(getActiveBibleRenderGraphProjectionRequest());
}

export function applyGraphRendererCommand(
  command: GraphRendererCommand,
  target: GraphRendererSelectionTarget = bibleGraphRendererSelectionTarget,
): void {
  switch (command.type) {
    case 'select_node':
      target.selectNode(command.node_id);
      return;
    case 'select_edge':
      target.selectEdge(command.edge_id);
      return;
    case 'select_influence':
      target.selectInfluence(command.influence_id);
      return;
    case 'inspect_node':
      target.inspectNode(command.node_id);
      return;
    case 'focus_node':
      target.focusNode(command.node_id);
      return;
    case 'navigate_to_node':
      target.navigateToNode(command.node_id);
      return;
    case 'delete_node':
      void Promise.resolve(target.deleteNode(command.node_id)).catch(() => {});
      return;
    case 'create_connected_node':
      void Promise.resolve(target.createConnectedNode(command.parent_id)).catch(() => {});
      return;
    case 'clear_selection':
      target.clearSelection();
      return;
  }
}

export function applyGraphRendererCommands(
  commands: GraphRendererCommand[],
  target: GraphRendererSelectionTarget = bibleGraphRendererSelectionTarget,
): number {
  for (const command of commands) {
    applyGraphRendererCommand(command, target);
  }
  return commands.length;
}
