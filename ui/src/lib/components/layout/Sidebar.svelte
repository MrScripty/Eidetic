<script lang="ts">
	import { storyState } from '$lib/stores/story.svelte.js';
	import { PANEL } from '$lib/types.js';
	import ArcList from '../sidebar/ArcList.svelte';
	import ArcDetail from '../sidebar/ArcDetail.svelte';
	import StoryBibleTab from '../sidebar/bible/StoryBibleTab.svelte';
	import AiConfigPanel from '../sidebar/AiConfigPanel.svelte';
	import ReferencePanel from '../sidebar/ReferencePanel.svelte';
	import ProgressionPanel from '../sidebar/ProgressionPanel.svelte';

	let { onclose }: { onclose: () => void } = $props();
	let activeTab: 'arcs' | 'bible' | 'ai' | 'refs' = $state('arcs');
	let selectedArcId: string | null = $state(null);
	let showProgression = $state(false);

	function switchTab(tab: 'arcs' | 'bible' | 'ai' | 'refs') {
		activeTab = tab;
		selectedArcId = null;
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
				class:active={activeTab === 'bible'}
				onclick={() => switchTab('bible')}
			>
				Bible
			</button>
			<button
				class="tab"
				class:active={activeTab === 'ai'}
				onclick={() => switchTab('ai')}
			>
				AI
			</button>
			<button
				class="tab"
				class:active={activeTab === 'refs'}
				onclick={() => switchTab('refs')}
			>
				Refs
			</button>
		</div>
		<button class="close-btn" onclick={onclose}>&times;</button>
	</div>

	<div class="sidebar-content">
		{#if activeTab === 'arcs'}
			{#if showProgression}
				<ProgressionPanel />
				<button class="toggle-view" onclick={() => showProgression = false}>Back to Arcs</button>
			{:else if selectedArcId}
				{@const arc = storyState.arcs.find(a => a.id === selectedArcId)}
				{#if arc}
					<ArcDetail {arc} onback={() => selectedArcId = null} />
				{:else}
					<ArcList onselect={(id) => selectedArcId = id} />
				{/if}
			{:else}
				<ArcList onselect={(id) => selectedArcId = id} />
				<button class="toggle-view" onclick={() => showProgression = true}>Analysis</button>
			{/if}
		{:else if activeTab === 'bible'}
			<StoryBibleTab />
		{:else if activeTab === 'ai'}
			<AiConfigPanel />
		{:else}
			<ReferencePanel />
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

	.toggle-view {
		margin: 4px 8px;
		padding: 4px 10px;
		font-size: 0.7rem;
		background: var(--color-bg-surface);
		border: 1px solid var(--color-border-default);
		border-radius: 4px;
		color: var(--color-text-secondary);
		cursor: pointer;
		align-self: flex-start;
	}

	.toggle-view:hover {
		background: var(--color-bg-hover);
	}
</style>
