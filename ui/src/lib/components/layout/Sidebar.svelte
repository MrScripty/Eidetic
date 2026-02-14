<script lang="ts">
	import { storyState } from '$lib/stores/story.svelte.js';
	import { colorToHex } from '$lib/types.js';
	import { PANEL } from '$lib/types.js';

	let { onclose }: { onclose: () => void } = $props();
	let activeTab: 'arcs' | 'characters' = $state('arcs');
</script>

<aside class="sidebar" style="width: {PANEL.SIDEBAR_WIDTH_PX}px">
	<div class="sidebar-header">
		<div class="tabs">
			<button
				class="tab"
				class:active={activeTab === 'arcs'}
				onclick={() => activeTab = 'arcs'}
			>
				Arcs
			</button>
			<button
				class="tab"
				class:active={activeTab === 'characters'}
				onclick={() => activeTab = 'characters'}
			>
				Characters
			</button>
		</div>
		<button class="close-btn" onclick={onclose}>&times;</button>
	</div>

	<div class="sidebar-content">
		{#if activeTab === 'arcs'}
			<ul class="entity-list">
				{#each storyState.arcs as arc}
					<li class="entity-item">
						<span class="color-dot" style="background: {colorToHex(arc.color)}"></span>
						<span class="entity-name">{arc.name}</span>
						<span class="entity-type">{typeof arc.arc_type === 'string' ? arc.arc_type : arc.arc_type.Custom}</span>
					</li>
				{/each}
				{#if storyState.arcs.length === 0}
					<li class="empty-state">No arcs yet</li>
				{/if}
			</ul>
		{:else}
			<ul class="entity-list">
				{#each storyState.characters as character}
					<li class="entity-item">
						<span class="color-dot" style="background: {colorToHex(character.color)}"></span>
						<span class="entity-name">{character.name}</span>
					</li>
				{/each}
				{#if storyState.characters.length === 0}
					<li class="empty-state">No characters yet</li>
				{/if}
			</ul>
		{/if}
	</div>
</aside>

<style>
	.sidebar {
		display: flex;
		flex-direction: column;
		background: var(--color-bg-secondary);
		border-right: 1px solid var(--color-border-default);
		flex-shrink: 0;
		overflow: hidden;
	}

	.sidebar-header {
		display: flex;
		align-items: center;
		justify-content: space-between;
		padding: 8px 12px;
		border-bottom: 1px solid var(--color-border-subtle);
	}

	.tabs {
		display: flex;
		gap: 4px;
	}

	.tab {
		padding: 4px 12px;
		background: none;
		border: none;
		color: var(--color-text-secondary);
		cursor: pointer;
		font-size: 0.85rem;
		border-radius: 4px;
	}

	.tab.active {
		background: var(--color-bg-surface);
		color: var(--color-text-primary);
	}

	.close-btn {
		background: none;
		border: none;
		color: var(--color-text-muted);
		cursor: pointer;
		font-size: 1.2rem;
		padding: 0 4px;
	}

	.sidebar-content {
		flex: 1;
		overflow-y: auto;
		padding: 8px 0;
	}

	.entity-list {
		list-style: none;
		margin: 0;
		padding: 0;
	}

	.entity-item {
		display: flex;
		align-items: center;
		gap: 8px;
		padding: 8px 12px;
		cursor: pointer;
		transition: background 0.1s;
	}

	.entity-item:hover {
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
</style>
