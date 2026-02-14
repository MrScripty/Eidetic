<script lang="ts">
	import type { EntityCategory } from '$lib/types.js';
	import { createEntity } from '$lib/api.js';
	import { storyState } from '$lib/stores/story.svelte.js';
	import EntityCard from './EntityCard.svelte';
	import EntityDetail from './EntityDetail.svelte';

	let selectedEntityId: string | null = $state(null);
	let searchQuery = $state('');
	let activeFilter: EntityCategory | 'All' = $state('All');

	const categories: (EntityCategory | 'All')[] = ['All', 'Character', 'Location', 'Prop', 'Theme', 'Event'];

	const filteredEntities = $derived(() => {
		let list = storyState.entities;
		if (activeFilter !== 'All') {
			list = list.filter(e => e.category === activeFilter);
		}
		if (searchQuery.trim()) {
			const q = searchQuery.toLowerCase();
			list = list.filter(e =>
				e.name.toLowerCase().includes(q) ||
				e.tagline.toLowerCase().includes(q)
			);
		}
		return list;
	});

	const selectedEntity = $derived(
		selectedEntityId ? storyState.entities.find(e => e.id === selectedEntityId) : null
	);

	async function handleAdd(category: EntityCategory) {
		const defaults: Record<EntityCategory, string> = {
			Character: 'New Character',
			Location: 'New Location',
			Prop: 'New Prop',
			Theme: 'New Theme',
			Event: 'New Event',
		};
		const entity = await createEntity({ name: defaults[category], category });
		selectedEntityId = entity.id;
	}

	function filterLabel(cat: EntityCategory | 'All'): string {
		if (cat === 'All') return 'All';
		return cat.slice(0, 3);
	}

	function filterColor(cat: EntityCategory | 'All'): string {
		switch (cat) {
			case 'Character': return 'var(--color-entity-character)';
			case 'Location': return 'var(--color-entity-location)';
			case 'Prop': return 'var(--color-entity-prop)';
			case 'Theme': return 'var(--color-entity-theme)';
			case 'Event': return 'var(--color-entity-event)';
			default: return 'var(--color-text-secondary)';
		}
	}
</script>

{#if selectedEntity}
	<EntityDetail entity={selectedEntity} onback={() => selectedEntityId = null} />
{:else}
	<div class="bible-tab">
		<div class="search-bar">
			<input
				class="search-input"
				type="text"
				placeholder="Search entities..."
				bind:value={searchQuery}
			/>
		</div>

		<div class="filter-pills">
			{#each categories as cat}
				<button
					class="pill"
					class:active={activeFilter === cat}
					style={activeFilter === cat ? `color: ${filterColor(cat)}; border-color: ${filterColor(cat)}` : ''}
					onclick={() => activeFilter = cat}
				>
					{filterLabel(cat)}
				</button>
			{/each}
		</div>

		<ul class="entity-list">
			{#each filteredEntities() as entity (entity.id)}
				<li>
					<EntityCard {entity} onselect={(id) => selectedEntityId = id} />
				</li>
			{/each}
			{#if filteredEntities().length === 0}
				<li class="empty-state">
					{searchQuery ? 'No matching entities' : 'No entities yet'}
				</li>
			{/if}
		</ul>

		<div class="add-buttons">
			{#if activeFilter !== 'All'}
				<button class="add-btn" onclick={() => handleAdd(activeFilter as EntityCategory)}>
					+ Add {activeFilter}
				</button>
			{:else}
				<div class="add-menu">
					<button class="add-btn-small" style="color: var(--color-entity-character)" onclick={() => handleAdd('Character')}>+ Chr</button>
					<button class="add-btn-small" style="color: var(--color-entity-location)" onclick={() => handleAdd('Location')}>+ Loc</button>
					<button class="add-btn-small" style="color: var(--color-entity-prop)" onclick={() => handleAdd('Prop')}>+ Prp</button>
					<button class="add-btn-small" style="color: var(--color-entity-theme)" onclick={() => handleAdd('Theme')}>+ Thm</button>
					<button class="add-btn-small" style="color: var(--color-entity-event)" onclick={() => handleAdd('Event')}>+ Evt</button>
				</div>
			{/if}
		</div>
	</div>
{/if}

<style>
	.bible-tab {
		display: flex;
		flex-direction: column;
		height: 100%;
	}

	.search-bar {
		padding: 8px 12px 4px;
	}

	.search-input {
		width: 100%;
		padding: 6px 10px;
		background: var(--color-bg-surface);
		color: var(--color-text-primary);
		border: 1px solid var(--color-border-default);
		border-radius: 4px;
		font-size: 0.85rem;
		box-sizing: border-box;
	}

	.search-input:focus {
		outline: none;
		border-color: var(--color-accent);
	}

	.search-input::placeholder {
		color: var(--color-text-muted);
	}

	.filter-pills {
		display: flex;
		gap: 4px;
		padding: 6px 12px;
		flex-wrap: wrap;
	}

	.pill {
		padding: 2px 8px;
		font-size: 0.7rem;
		border: 1px solid var(--color-border-default);
		border-radius: 12px;
		background: none;
		color: var(--color-text-secondary);
		cursor: pointer;
		text-transform: uppercase;
		letter-spacing: 0.04em;
		font-weight: 500;
	}

	.pill.active {
		background: var(--color-bg-elevated);
	}

	.pill:hover {
		background: var(--color-bg-hover);
	}

	.entity-list {
		list-style: none;
		margin: 0;
		padding: 0;
		flex: 1;
		overflow-y: auto;
	}

	.empty-state {
		padding: 16px 12px;
		color: var(--color-text-muted);
		font-size: 0.85rem;
		text-align: center;
	}

	.add-buttons {
		border-top: 1px solid var(--color-border-subtle);
	}

	.add-btn {
		width: 100%;
		padding: 8px 12px;
		background: none;
		border: none;
		color: var(--color-accent);
		cursor: pointer;
		font-size: 0.85rem;
		text-align: center;
	}

	.add-btn:hover {
		background: var(--color-bg-hover);
	}

	.add-menu {
		display: flex;
		justify-content: center;
		gap: 2px;
		padding: 4px;
	}

	.add-btn-small {
		padding: 6px 8px;
		background: none;
		border: none;
		cursor: pointer;
		font-size: 0.75rem;
		font-weight: 500;
		border-radius: 4px;
	}

	.add-btn-small:hover {
		background: var(--color-bg-hover);
	}
</style>
