import type { BibleGraphEdgeId, BibleGraphNodeId } from '../bibleGraphTypes.js';

export type BibleGraphSelection =
  | { kind: 'none' }
  | { kind: 'node'; nodeId: BibleGraphNodeId }
  | { kind: 'edge'; edgeId: BibleGraphEdgeId }
  | { kind: 'influence'; influenceId: string }
  | { kind: 'context_layer'; timelineNodeId: string }
  | { kind: 'neighborhood'; nodeId: BibleGraphNodeId };

/** Shared bible selection state — connects sidebar list to the detail panel. */
export const bibleState = $state<{
  graphSelection: BibleGraphSelection;
  graphFocusedNeighborhoodNodeId: BibleGraphNodeId | null;
}>({
  graphSelection: { kind: 'none' },
  graphFocusedNeighborhoodNodeId: null,
});

export function selectBibleGraphNode(id: BibleGraphNodeId | null): void {
  bibleState.graphSelection = id ? { kind: 'node', nodeId: id } : { kind: 'none' };
}

export function selectBibleGraphEdge(id: BibleGraphEdgeId | null): void {
  bibleState.graphSelection = id ? { kind: 'edge', edgeId: id } : { kind: 'none' };
}

export function selectBibleGraphInfluence(id: string | null): void {
  bibleState.graphSelection = id ? { kind: 'influence', influenceId: id } : { kind: 'none' };
}

export function selectBibleGraphContextLayer(timelineNodeId: string | null): void {
  bibleState.graphSelection = timelineNodeId
    ? { kind: 'context_layer', timelineNodeId }
    : { kind: 'none' };
}

export function selectBibleGraphNeighborhood(id: BibleGraphNodeId | null): void {
  bibleState.graphSelection = id ? { kind: 'neighborhood', nodeId: id } : { kind: 'none' };
}

export function focusBibleGraphNeighborhood(id: BibleGraphNodeId | null): void {
  bibleState.graphFocusedNeighborhoodNodeId = id;
  if (id) {
    bibleState.graphSelection = { kind: 'neighborhood', nodeId: id };
  }
}

export function clearBibleGraphFocusedNeighborhood(): void {
  bibleState.graphFocusedNeighborhoodNodeId = null;
}

export function clearBibleGraphSelection(): void {
  bibleState.graphSelection = { kind: 'none' };
}

export function selectedBibleGraphNodeId(): BibleGraphNodeId | null {
  return bibleState.graphSelection.kind === 'node' ? bibleState.graphSelection.nodeId : null;
}
