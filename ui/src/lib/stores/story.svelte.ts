import type { StoryArc } from '../types.js';

/** Reactive story state for legacy arc surfaces that have not moved to projections yet. */
export const storyState = $state<{
  arcs: StoryArc[];
}>({
  arcs: [],
});
