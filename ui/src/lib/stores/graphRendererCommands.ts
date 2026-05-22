import type { BibleGraphEdgeId, BibleGraphNodeId } from '$lib/bibleGraphTypes.js';
import type { GraphRendererCommand } from '$lib/graphRendererTypes.js';
import {
  selectBibleGraphEdge,
  selectBibleGraphInfluence,
  selectBibleGraphNode,
} from './bible.svelte.js';

export interface GraphRendererSelectionTarget {
  selectNode(nodeId: BibleGraphNodeId): void;
  selectEdge(edgeId: BibleGraphEdgeId): void;
  selectInfluence(influenceId: string): void;
  inspectNode(nodeId: BibleGraphNodeId): void;
}

export const bibleGraphRendererSelectionTarget: GraphRendererSelectionTarget = {
  selectNode: selectBibleGraphNode,
  selectEdge: selectBibleGraphEdge,
  selectInfluence: selectBibleGraphInfluence,
  inspectNode: selectBibleGraphNode,
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
