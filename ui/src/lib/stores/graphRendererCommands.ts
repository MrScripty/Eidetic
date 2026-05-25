import type { BibleGraphEdgeId, BibleGraphNodeId } from '$lib/bibleGraphTypes.js';
import type { GraphRendererCommand } from '$lib/graphRendererTypes.js';
import {
  clearBibleGraphSelection,
  selectBibleGraphEdge,
  selectBibleGraphInfluence,
  selectBibleGraphNeighborhood,
  selectBibleGraphNode,
} from './bible.svelte.js';

export interface GraphRendererSelectionTarget {
  selectNode(nodeId: BibleGraphNodeId): void;
  selectEdge(edgeId: BibleGraphEdgeId): void;
  selectInfluence(influenceId: string): void;
  inspectNode(nodeId: BibleGraphNodeId): void;
  focusNode(nodeId: BibleGraphNodeId): void;
  navigateToNode(nodeId: BibleGraphNodeId): void;
  clearSelection(): void;
}

export const bibleGraphRendererSelectionTarget: GraphRendererSelectionTarget = {
  selectNode: selectBibleGraphNode,
  selectEdge: selectBibleGraphEdge,
  selectInfluence: selectBibleGraphInfluence,
  inspectNode: selectBibleGraphNode,
  focusNode: selectBibleGraphNeighborhood,
  navigateToNode: selectBibleGraphNode,
  clearSelection: clearBibleGraphSelection,
};

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
