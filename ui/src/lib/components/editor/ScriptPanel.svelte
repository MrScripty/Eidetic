<script lang="ts">
	import ScriptView from './ScriptView.svelte';
	import { timelineState } from '$lib/stores/timeline.svelte.js';
	import { storyState } from '$lib/stores/story.svelte.js';
	import { editorState } from '$lib/stores/editor.svelte.js';
	import { colorToHex } from '$lib/types.js';
	import type { BeatClip, StoryArc } from '$lib/types.js';

	/** All clips sorted by start time across all tracks. */
	let sortedClips = $derived.by(() => {
		const tl = timelineState.timeline;
		if (!tl) return [];
		const entries: { clip: BeatClip; arcId: string }[] = [];
		for (const track of tl.tracks) {
			for (const clip of track.clips) {
				entries.push({ clip, arcId: track.arc_id });
			}
		}
		entries.sort((a, b) => a.clip.time_range.start_ms - b.clip.time_range.start_ms);
		return entries;
	});

	function arcFor(arcId: string): StoryArc | undefined {
		return storyState.arcs.find(a => a.id === arcId);
	}

	function scriptText(clip: BeatClip): string | null {
		return clip.content.user_refined_script || clip.content.generated_script || null;
	}

	let scrollContainer: HTMLDivElement | undefined = $state();

	// Auto-scroll to selected clip's section.
	$effect(() => {
		const selId = editorState.selectedClipId;
		if (selId && scrollContainer) {
			const el = scrollContainer.querySelector(`[data-clip-id="${selId}"]`);
			if (el) {
				el.scrollIntoView({ behavior: 'smooth', block: 'nearest' });
			}
		}
	});
</script>

<div class="script-panel" bind:this={scrollContainer}>
	<div class="script-panel-header">
		<span class="script-panel-title">Script</span>
		<span class="script-panel-count">{sortedClips.filter(e => scriptText(e.clip)).length} / {sortedClips.length} beats</span>
	</div>

	<div class="script-panel-body">
		{#if sortedClips.length === 0}
			<p class="script-empty">No clips on the timeline yet.</p>
		{:else}
			{#each sortedClips as entry (entry.clip.id)}
				{@const arc = arcFor(entry.arcId)}
				{@const text = scriptText(entry.clip)}
				<div
					class="script-beat"
					class:selected={editorState.selectedClipId === entry.clip.id}
					class:dimmed={!text}
					data-clip-id={entry.clip.id}
				>
					<div class="beat-header">
						<span
							class="arc-dot"
							style="background: {arc ? colorToHex(arc.color) : 'var(--color-rel-default)'}"
						></span>
						<span class="beat-name">{entry.clip.name}</span>
						<span class="beat-type">{entry.clip.beat_type}</span>
					</div>
					{#if text}
						<ScriptView {text} entities={storyState.entities} />
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
		overflow-y: auto;
		padding: 8px 12px;
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

	.no-script {
		font-size: 0.75rem;
		color: var(--color-text-muted);
		font-style: italic;
		margin: 4px 0;
		padding-left: 14px;
	}
</style>
