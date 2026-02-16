<script lang="ts">
	import ScriptView from './ScriptView.svelte';
	import { timelineState, scrollToNode, nodesAtLevel } from '$lib/stores/timeline.svelte.js';
	import { storyState } from '$lib/stores/story.svelte.js';
	import { editorState } from '$lib/stores/editor.svelte.js';
	import { colorToHex } from '$lib/types.js';
	import type { StoryNode } from '$lib/types.js';

	/** All Beat-level nodes sorted by start time — these contain the scripts. */
	let sortedNodes = $derived.by(() => {
		return nodesAtLevel('Beat');
	});

	/** Get the primary arc color for a node from node_arcs. */
	function arcColorForNode(nodeId: string): string {
		const nodeArcs = timelineState.timeline?.node_arcs.filter(na => na.node_id === nodeId) ?? [];
		if (nodeArcs.length === 0) return 'var(--color-rel-default)';
		const arc = storyState.arcs.find(a => a.id === nodeArcs[0]!.arc_id);
		return arc ? colorToHex(arc.color) : 'var(--color-rel-default)';
	}

	function scriptText(node: StoryNode): string | null {
		return node.content.content || null;
	}


	function handleNodeClick(node: StoryNode) {
		editorState.selectedNodeId = node.id;
		editorState.selectedNode = node;
		scrollToNode(node.time_range.start_ms, node.time_range.end_ms);
	}

	let scrollContainer: HTMLDivElement | undefined = $state();

	// Auto-scroll to selected node's section.
	$effect(() => {
		const selId = editorState.selectedNodeId;
		if (selId && scrollContainer) {
			const el = scrollContainer.querySelector(`[data-node-id="${selId}"]`);
			if (el) {
				el.scrollIntoView({ behavior: 'smooth', block: 'nearest' });
			}
		}
	});
</script>

<div class="script-panel" bind:this={scrollContainer}>
	<div class="script-panel-header">
		<span class="script-panel-title">Script</span>
		<span class="script-panel-count">{sortedNodes.filter(n => scriptText(n)).length} / {sortedNodes.length} beats</span>
	</div>

	<div class="script-panel-body">
		{#if sortedNodes.length === 0}
			<p class="script-empty">No beat nodes on the timeline yet.</p>
		{:else}
			{#each sortedNodes as node (node.id)}
				{@const text = scriptText(node)}
				{@const streamText = editorState.streamingNodeId === node.id && editorState.streamingText ? editorState.streamingText : null}
				<!-- svelte-ignore a11y_no_static_element_interactions -->
				<!-- svelte-ignore a11y_click_events_have_key_events -->
				<div
					class="script-beat"
					class:selected={editorState.selectedNodeId === node.id}
					class:dimmed={!text && !streamText}
					data-node-id={node.id}
					onclick={() => handleNodeClick(node)}
				>
					<div class="beat-header">
						<span
							class="arc-dot"
							style="background: {arcColorForNode(node.id)}"
						></span>
						<span class="beat-name">{node.name}</span>
						{#if node.beat_type}
							<span class="beat-type">{typeof node.beat_type === 'string' ? node.beat_type : node.beat_type.Custom}</span>
						{/if}
					</div>
					{#if text}
						<ScriptView {text} entities={storyState.entities} />
					{:else if streamText}
						<ScriptView text={streamText} streaming={true} entities={storyState.entities} />
					{:else if node.content.notes}
						<p class="beat-outline">{node.content.notes}</p>
					{:else}
						<p class="no-script">No script generated</p>
					{/if}
				</div>
			{/each}
		{/if}
	</div>
</div>

<style>
	.script-panel {
		display: flex;
		flex-direction: column;
		height: 100%;
		overflow: hidden;
		background: var(--color-bg-secondary);
	}

	.script-panel-header {
		display: flex;
		align-items: center;
		justify-content: space-between;
		padding: 4px 12px;
		border-bottom: 1px solid var(--color-border-subtle);
		flex-shrink: 0;
	}

	.script-panel-title {
		font-size: 0.8rem;
		font-weight: 600;
		color: var(--color-text-primary);
	}

	.script-panel-count {
		font-size: 0.7rem;
		color: var(--color-text-muted);
	}

	.script-panel-body {
		flex: 1;
		overflow-x: auto;
		overflow-y: hidden;
		padding: 8px 12px;
		column-width: 40ch;
		column-gap: 24px;
		column-rule: 1px solid var(--color-border-subtle);
		column-fill: auto;
	}

	.script-empty {
		color: var(--color-text-muted);
		font-size: 0.8rem;
		text-align: center;
		padding: 24px 0;
		margin: 0;
	}

	.script-beat {
		margin-bottom: 12px;
		border-radius: 4px;
		border: 1px solid transparent;
		padding: 4px;
		break-inside: avoid;
		cursor: pointer;
		transition: border-color 0.1s, background 0.1s;
	}

	.script-beat:hover:not(.selected) {
		border-color: var(--color-border-default);
		background: var(--color-bg-hover);
	}

	.script-beat.selected {
		border-color: var(--color-accent);
	}

	.script-beat.dimmed {
		opacity: 0.5;
	}

	.beat-header {
		display: flex;
		align-items: center;
		gap: 6px;
		margin-bottom: 4px;
	}

	.arc-dot {
		width: 8px;
		height: 8px;
		border-radius: 50%;
		flex-shrink: 0;
	}

	.beat-name {
		font-size: 0.75rem;
		font-weight: 600;
		color: var(--color-text-primary);
	}

	.beat-type {
		font-size: 0.65rem;
		color: var(--color-text-muted);
		text-transform: capitalize;
	}

	.beat-outline {
		font-size: 0.7rem;
		color: var(--color-text-muted);
		font-style: italic;
		margin: 4px 0;
		padding-left: 14px;
	}

	.no-script {
		font-size: 0.75rem;
		color: var(--color-text-muted);
		font-style: italic;
		margin: 4px 0;
		padding-left: 14px;
	}
</style>
