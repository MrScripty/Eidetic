<script lang="ts">
	import { colorToHex } from '$lib/types.js';
	import { createArc } from '$lib/api.js';
	import { storyState } from '$lib/stores/story.svelte.js';

	let { onselect }: {
		onselect: (id: string) => void;
	} = $props();

	async function handleAdd() {
		await createArc('New Arc', 'APlot');
	}
</script>

<div class="arc-list">
	<ul class="entity-list">
		{#each storyState.arcs as arc (arc.id)}
			<li class="entity-item">
				<button class="entity-btn" onclick={() => onselect(arc.id)}>
					<span class="color-dot" style="background: {colorToHex(arc.color)}"></span>
					<span class="entity-name">{arc.name}</span>
					<span class="entity-type">
						{typeof arc.arc_type === 'string' ? arc.arc_type : arc.arc_type.Custom}
					</span>
				</button>
			</li>
		{/each}
		{#if storyState.arcs.length === 0}
			<li class="empty-state">No arcs yet</li>
		{/if}
	</ul>
	<button class="add-btn" onclick={handleAdd}>+ Add Arc</button>
</div>

<style>
	.arc-list {
		display: flex;
		flex-direction: column;
		height: 100%;
	}

	.entity-list {
		list-style: none;
		margin: 0;
		padding: 0;
		flex: 1;
		overflow-y: auto;
	}

	.entity-item {
		padding: 0;
	}

	.entity-btn {
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

	.entity-btn:hover {
		background: var(--color-bg-hover);
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
	}

	.entity-type {
		color: var(--color-text-muted);
		font-size: 0.75rem;
	}

	.empty-state {
		padding: 16px 12px;
		color: var(--color-text-muted);
		font-size: 0.85rem;
		text-align: center;
	}

	.add-btn {
		padding: 8px 12px;
		background: none;
		border: none;
		border-top: 1px solid var(--color-border-subtle);
		color: var(--color-accent);
		cursor: pointer;
		font-size: 0.85rem;
		text-align: center;
	}

	.add-btn:hover {
		background: var(--color-bg-hover);
	}
</style>
