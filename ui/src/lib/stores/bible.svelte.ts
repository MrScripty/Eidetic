import type { EntityId } from '../types.js';

/** Shared bible selection state — connects sidebar list to the detail panel. */
export const bibleState = $state<{
	selectedEntityId: EntityId | null;
}>({
	selectedEntityId: null,
});

export function selectEntity(id: EntityId | null) {
	bibleState.selectedEntityId = id;
}
