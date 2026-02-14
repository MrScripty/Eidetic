<script lang="ts">
	import { storyState } from '$lib/stores/story.svelte.js';
	import { PANEL } from '$lib/types.js';
	import ArcList from '../sidebar/ArcList.svelte';
	import ArcDetail from '../sidebar/ArcDetail.svelte';
	import CharacterList from '../sidebar/CharacterList.svelte';
	import CharacterDetail from '../sidebar/CharacterDetail.svelte';
	import AiConfigPanel from '../sidebar/AiConfigPanel.svelte';

	let { onclose }: { onclose: () => void } = $props();
	let activeTab: 'arcs' | 'characters' | 'ai' = $state('arcs');
	let selectedArcId: string | null = $state(null);
	let selectedCharacterId: string | null = $state(null);

	function switchTab(tab: 'arcs' | 'characters' | 'ai') {
		activeTab = tab;
		selectedArcId = null;
		selectedCharacterId = null;
	}
</script>

<aside class="sidebar" style="width: {PANEL.SIDEBAR_WIDTH_PX}px">
	<div class="sidebar-header">
		<div class="tabs">
			<button
				class="tab"
				class:active={activeTab === 'arcs'}
				onclick={() => switchTab('arcs')}
			>
				Arcs
			</button>
			<button
				class="tab"
				class:active={activeTab === 'characters'}
				onclick={() => switchTab('characters')}
			>
				Characters
			</button>
			<button
				class="tab"
				class:active={activeTab === 'ai'}
				onclick={() => switchTab('ai')}
			>
				AI
			</button>
		</div>
		<button class="close-btn" onclick={onclose}>&times;</button>
	</div>

	<div class="sidebar-content">
		{#if activeTab === 'arcs'}
			{#if selectedArcId}
				{@const arc = storyState.arcs.find(a => a.id === selectedArcId)}
				{#if arc}
					<ArcDetail {arc} onback={() => selectedArcId = null} />
				{:else}
					<ArcList onselect={(id) => selectedArcId = id} />
				{/if}
			{:else}
				<ArcList onselect={(id) => selectedArcId = id} />
			{/if}
		{:else if activeTab === 'characters'}
			{#if selectedCharacterId}
				{@const character = storyState.characters.find(c => c.id === selectedCharacterId)}
				{#if character}
					<CharacterDetail {character} onback={() => selectedCharacterId = null} />
				{:else}
					<CharacterList onselect={(id) => selectedCharacterId = id} />
				{/if}
			{:else}
				<CharacterList onselect={(id) => selectedCharacterId = id} />
			{/if}
		{:else}
			<AiConfigPanel />
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
		overflow: hidden;
		display: flex;
		flex-direction: column;
	}
</style>
