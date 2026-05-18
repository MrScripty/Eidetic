import type { BibleGraphNodeId } from '../types.js';

/** Shared bible selection state — connects sidebar list to the detail panel. */
export const bibleState = $state<{
  selectedGraphNodeId: BibleGraphNodeId | null;
}>({
  selectedGraphNodeId: null,
});

export function selectBibleGraphNode(id: BibleGraphNodeId | null) {
  bibleState.selectedGraphNodeId = id;
}
