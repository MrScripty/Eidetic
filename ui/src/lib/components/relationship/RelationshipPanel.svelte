<script lang="ts">
	import RelationshipGraph from './RelationshipGraph.svelte';
	import type { GraphNode, GraphEdge } from './RelationshipGraph.svelte';
	import { storyState, entitiesByCategory } from '$lib/stores/story.svelte.js';
	import { colorToHex } from '$lib/types.js';
	import type { EntityCategory } from '$lib/types.js';

	let { width, onclose, compact = false }: { width: number; onclose: () => void; compact?: boolean } = $props();

	let showAllEntities = $state(false);
	let selectedNodeId: string | null = $state(null);
	let hoveredNodeId: string | null = $state(null);

	const CATEGORY_COLORS: Record<EntityCategory, string> = {
		Character: 'var(--color-entity-character)',
		Location: 'var(--color-entity-location)',
		Prop: 'var(--color-entity-prop)',
		Theme: 'var(--color-entity-theme)',
		Event: 'var(--color-entity-event)',
	};

	const entities = $derived(
		showAllEntities ? storyState.entities : entitiesByCategory('Character')
	);

	const entityMap = $derived(
		new Map(storyState.entities.map(e => [e.id, e]))
	);

	/** Circular layout of nodes. */
	const nodes: GraphNode[] = $derived.by(() => {
		const count = entities.length;
		if (count === 0) return [];

		const cx = width / 2;
		const cy = width / 2;
		const radius = Math.min(width, 400) / 2 - 60;

		if (count === 1) {
			const e = entities[0]!;
			return [{
				id: e.id,
				name: e.name,
				color: colorToHex(e.color),
				x: cx,
				y: cy,
			}];
		}

		return entities.map((entity, i) => {
			const angle = (2 * Math.PI * i) / count - Math.PI / 2;
			return {
				id: entity.id,
				name: entity.name,
				color: colorToHex(entity.color),
				x: cx + radius * Math.cos(angle),
				y: cy + radius * Math.sin(angle),
			};
		});
	});

	/** Collect and deduplicate edges from both generic relations and character_relations. */
	const edges: GraphEdge[] = $derived.by(() => {
		const result: GraphEdge[] = [];
		const entityIds = new Set(entities.map(e => e.id));
		const seen = new Set<string>();

		for (const entity of entities) {
			// Generic entity relations
			for (const rel of entity.relations) {
				if (!entityIds.has(rel.target_entity_id)) continue;
				const key = [entity.id, rel.target_entity_id].sort().join('|') + '|' + rel.label;
				if (seen.has(key)) continue;
				seen.add(key);
				result.push({
					sourceId: entity.id,
					targetId: rel.target_entity_id,
					label: rel.label,
					color: CATEGORY_COLORS[entity.category] ?? 'var(--color-text-muted)',
				});
			}

			// Character-specific relations
			if (entity.details.type === 'Character') {
				for (const [targetId, label] of entity.details.character_relations) {
					if (!entityIds.has(targetId)) continue;
					const key = [entity.id, targetId].sort().join('|') + '|' + label;
					if (seen.has(key)) continue;
					seen.add(key);
					result.push({
						sourceId: entity.id,
						targetId,
						label,
						color: 'var(--color-entity-character)',
					});
				}
			}
		}

		return result;
	});

	/** Relations for the selected entity (for the detail footer). */
	const selectedEntity = $derived(
		selectedNodeId ? entityMap.get(selectedNodeId) ?? null : null
	);

	const selectedRelations = $derived.by(() => {
		if (!selectedEntity) return [];
		const rels: { label: string; targetName: string }[] = [];

		for (const rel of selectedEntity.relations) {
			const target = entityMap.get(rel.target_entity_id);
			if (target) rels.push({ label: rel.label, targetName: target.name });
		}

		if (selectedEntity.details.type === 'Character') {
			for (const [targetId, label] of selectedEntity.details.character_relations) {
				const target = entityMap.get(targetId);
				if (target) rels.push({ label, targetName: target.name });
			}
		}

		return rels;
	});

	function handleSelect(id: string) {
		selectedNodeId = selectedNodeId === id ? null : id;
	}
</script>

<div class="relationship-panel" class:compact>
	<div class="panel-header">
		<span class="panel-title">Relationships</span>
		<div class="header-controls">
			<button
				class="filter-btn"
				class:active={showAllEntities}
				onclick={() => showAllEntities = !showAllEntities}
				title={showAllEntities ? 'Showing all entities' : 'Showing characters only'}
			>
				{showAllEntities ? 'All' : 'Chr'}
			</button>
			<button class="close-btn" onclick={onclose}>&times;</button>
		</div>
	</div>

	<div class="graph-container">
		{#if entities.length === 0}
			<div class="empty-state">
				<p>No {showAllEntities ? 'entities' : 'characters'} yet</p>
				<p class="empty-hint">Add them via the Bible tab in the sidebar</p>
			</div>
		{:else}
			<RelationshipGraph
				{nodes}
				{edges}
				{selectedNodeId}
				{hoveredNodeId}
				onselect={handleSelect}
				onhover={(id) => hoveredNodeId = id}
			/>
		{/if}
	</div>

	{#if selectedEntity}
		<div class="detail-footer">
			<div class="detail-name">{selectedEntity.name}</div>
			{#if selectedEntity.tagline}
				<div class="detail-tagline">{selectedEntity.tagline}</div>
			{/if}
			{#if selectedRelations.length > 0}
				<div class="detail-relations">
					{#each selectedRelations as rel}
						<div class="detail-relation">
							<span class="rel-label">{rel.label}</span>
							<span class="rel-target">{rel.targetName}</span>
						</div>
					{/each}
				</div>
			{/if}
		</div>
	{/if}
</div>

<style>
	.relationship-panel {
		display: flex;
		flex-direction: column;
		overflow: hidden;
		flex: 1;
	}

	.relationship-panel.compact {
		flex: 0 0 auto;
		max-height: 45%;
	}

	.panel-header {
		display: flex;
		align-items: center;
		justify-content: space-between;
		padding: 8px 12px;
		border-bottom: 1px solid var(--color-border-subtle);
		flex-shrink: 0;
	}

	.panel-title {
		font-size: 0.85rem;
		color: var(--color-text-primary);
		font-weight: 500;
	}

	.header-controls {
		display: flex;
		align-items: center;
		gap: 4px;
	}

	.filter-btn {
		background: none;
		border: 1px solid var(--color-border-default);
		color: var(--color-text-secondary);
		padding: 2px 8px;
		border-radius: 4px;
		cursor: pointer;
		font-size: 0.75rem;
	}

	.filter-btn:hover {
		background: var(--color-bg-hover);
	}

	.filter-btn.active {
		background: var(--color-bg-hover);
		color: var(--color-accent);
		border-color: var(--color-accent);
	}

	.close-btn {
		background: none;
		border: none;
		color: var(--color-text-secondary);
		cursor: pointer;
		font-size: 1.1rem;
		padding: 0 4px;
		line-height: 1;
	}

	.close-btn:hover {
		color: var(--color-text-primary);
	}

	.graph-container {
		flex: 1;
		overflow: auto;
		position: relative;
	}

	.empty-state {
		display: flex;
		flex-direction: column;
		align-items: center;
		justify-content: center;
		height: 100%;
		color: var(--color-text-muted);
		font-size: 0.85rem;
		gap: 4px;
	}

	.empty-state p {
		margin: 0;
	}

	.empty-hint {
		font-size: 0.75rem;
	}

	.detail-footer {
		border-top: 1px solid var(--color-border-subtle);
		padding: 8px 12px;
		max-height: 150px;
		overflow-y: auto;
		flex-shrink: 0;
	}

	.detail-name {
		font-size: 0.85rem;
		font-weight: 500;
		color: var(--color-text-primary);
		margin-bottom: 2px;
	}

	.detail-tagline {
		font-size: 0.75rem;
		color: var(--color-text-secondary);
		margin-bottom: 6px;
	}

	.detail-relations {
		display: flex;
		flex-direction: column;
		gap: 2px;
	}

	.detail-relation {
		display: flex;
		align-items: center;
		gap: 6px;
		font-size: 0.75rem;
	}

	.rel-label {
		color: var(--color-text-muted);
		font-style: italic;
	}

	.rel-target {
		color: var(--color-text-secondary);
	}
</style>
