import type { BibleGraphNodeId, EntityId } from '../types.js';

/** Shared bible selection state — connects sidebar list to the detail panel. */
export const bibleState = $state<{
  selectedEntityId: EntityId | null;
  selectedGraphNodeId: BibleGraphNodeId | null;
}>({
  selectedEntityId: null,
  selectedGraphNodeId: null,
});

export function selectEntity(id: EntityId | null) {
  bibleState.selectedEntityId = id;
  bibleState.selectedGraphNodeId = null;
}

export function selectBibleGraphNode(id: BibleGraphNodeId | null) {
  bibleState.selectedGraphNodeId = id;
  bibleState.selectedEntityId = null;
}
