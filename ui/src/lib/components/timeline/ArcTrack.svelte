<script lang="ts">
	import type { ArcTrack as ArcTrackType } from '$lib/types.js';
	import { TIMELINE } from '$lib/types.js';
	import { timeToX, rangeWidth } from '$lib/stores/timeline.svelte.js';
	import { editorState } from '$lib/stores/editor.svelte.js';

	let { track, color, label }: {
		track: ArcTrackType;
		color: string;
		label: string;
	} = $props();

	function selectClip(clip: ArcTrackType['clips'][number]) {
		editorState.selectedClipId = clip.id;
		editorState.selectedClip = clip;
	}
</script>

<div class="arc-track" style="height: {TIMELINE.TRACK_HEIGHT_PX}px">
	<div class="track-label">{label}</div>
	<div class="track-lane">
		{#each track.clips as clip}
			<button
				class="beat-clip"
				class:selected={editorState.selectedClipId === clip.id}
				class:locked={clip.locked}
				style="
					left: {timeToX(clip.time_range.start_ms)}px;
					width: {rangeWidth(clip.time_range)}px;
					background: {color};
				"
				onclick={() => selectClip(clip)}
				title="{clip.name} ({clip.content.status})"
			>
				<span class="clip-name">{clip.name}</span>
			</button>
		{/each}
	</div>
</div>

<style>
	.arc-track {
		display: flex;
		align-items: center;
		border-bottom: 1px solid var(--color-border-subtle);
	}

	.track-label {
		width: 80px;
		flex-shrink: 0;
		padding: 0 8px;
		font-size: 0.75rem;
		color: var(--color-text-secondary);
		text-align: right;
		overflow: hidden;
		text-overflow: ellipsis;
		white-space: nowrap;
	}

	.track-lane {
		flex: 1;
		position: relative;
		height: 100%;
	}

	.beat-clip {
		position: absolute;
		top: 4px;
		height: calc(100% - 8px);
		border: none;
		border-radius: 4px;
		cursor: pointer;
		display: flex;
		align-items: center;
		justify-content: center;
		overflow: hidden;
		opacity: 0.85;
		transition: opacity 0.1s, outline 0.1s;
		padding: 0 4px;
		min-width: 0;
	}

	.beat-clip:hover {
		opacity: 1;
	}

	.beat-clip.selected {
		outline: 2px solid var(--color-accent);
		outline-offset: 1px;
		opacity: 1;
	}

	.beat-clip.locked {
		border: 1px dashed rgba(255, 255, 255, 0.4);
	}

	.clip-name {
		font-size: 0.7rem;
		color: rgba(0, 0, 0, 0.8);
		font-weight: 600;
		text-overflow: ellipsis;
		overflow: hidden;
		white-space: nowrap;
	}
</style>
