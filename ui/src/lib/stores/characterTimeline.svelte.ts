import type { EntityId } from '../types.js';

/** Reactive UI state for the character progression timeline lane. */
export const characterTimelineState = $state<{
	/** Currently selected character entity ID to display progression for. */
	selectedCharacterId: EntityId | null;
	/** Whether the character timeline lane is visible. */
	visible: boolean;
	/** Marker type visibility filters. */
	showSnapshots: boolean;
	showEvents: boolean;
	showMentions: boolean;
	/** ID of the marker being hovered (for tooltip display). */
	hoveredMarkerId: string | null;
}>({
	selectedCharacterId: null,
	visible: false,
	showSnapshots: true,
	showEvents: true,
	showMentions: true,
	hoveredMarkerId: null,
});
