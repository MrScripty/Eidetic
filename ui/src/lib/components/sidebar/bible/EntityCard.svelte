<script lang="ts">
	import type { Entity } from '$lib/types.js';
	import { colorToHex } from '$lib/types.js';

	let { entity, selected = false, onselect }: {
		entity: Entity;
		selected?: boolean;
		onselect: (id: string) => void;
	} = $props();

	const categoryLabels: Record<string, string> = {
		Character: 'CHR',
		Location: 'LOC',
		Prop: 'PRP',
		Theme: 'THM',
		Event: 'EVT',
	};

	function categoryColor(cat: string): string {
		switch (cat) {
			case 'Character': return 'var(--color-entity-character)';
			case 'Location': return 'var(--color-entity-location)';
			case 'Prop': return 'var(--color-entity-prop)';
			case 'Theme': return 'var(--color-entity-theme)';
			case 'Event': return 'var(--color-entity-event)';
			default: return 'var(--color-text-muted)';
		}
	}
</script>

<button class="entity-card" class:selected onclick={() => onselect(entity.id)}>
	<span class="color-dot" style="background: {colorToHex(entity.color)}"></span>
	<span class="entity-name">{entity.name}</span>
	<span class="category-badge" style="color: {categoryColor(entity.category)}">
		{categoryLabels[entity.category] ?? entity.category}
	</span>
	{#if entity.clip_refs.length > 0}
		<span class="ref-count" title="{entity.clip_refs.length} linked clips">
			{entity.clip_refs.length}
		</span>
	{/if}
</button>

<style>
	.entity-card {
		display: flex;
		align-items: center;
		gap: 8px;
		width: 100%;
		padding: 8px 12px;
		background: none;
		border: none;
		cursor: pointer;
		transition: background 0.1s;
		text-align: left;
	}

	.entity-card:hover {
		background: var(--color-bg-hover);
	}

	.entity-card.selected {
		background: var(--color-bg-surface);
		border-left: 2px solid var(--color-accent);
		padding-left: 10px;
	}

	.color-dot {
		width: 10px;
		height: 10px;
		border-radius: 50%;
		flex-shrink: 0;
	}

	.entity-name {
		flex: 1;
		color: var(--color-text-primary);
		font-size: 0.9rem;
		white-space: nowrap;
		overflow: hidden;
		text-overflow: ellipsis;
	}

	.category-badge {
		font-size: 0.65rem;
		font-weight: 600;
		letter-spacing: 0.05em;
		flex-shrink: 0;
	}

	.ref-count {
		font-size: 0.7rem;
		color: var(--color-text-muted);
		background: var(--color-bg-elevated);
		padding: 1px 5px;
		border-radius: 8px;
		flex-shrink: 0;
	}
</style>
