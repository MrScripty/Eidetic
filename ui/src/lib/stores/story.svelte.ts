import type { StoryArc, Character } from '../types.js';

/** Reactive story entity state. Arcs and characters from the server. */
export const storyState = $state<{
	arcs: StoryArc[];
	characters: Character[];
}>({
	arcs: [],
	characters: [],
});
