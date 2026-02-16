import type { StoryArc, Entity, EntityCategory } from '../types.js';

/** Reactive story entity state. Arcs and bible entities from the server. */
export const storyState = $state<{
	arcs: StoryArc[];
	entities: Entity[];
}>({
	arcs: [],
	entities: [],
});

/** Get entities filtered by category. */
export function entitiesByCategory(category: EntityCategory): Entity[] {
	return storyState.entities.filter(e => e.category === category);
}

/** Get all entities that reference a specific node. */
export function entitiesForNode(nodeId: string): Entity[] {
	return storyState.entities.filter(e => e.node_refs.includes(nodeId));
}
