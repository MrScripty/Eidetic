<script lang="ts">
	import type { EpisodeStructure } from '$lib/types.js';
	import { TIMELINE } from '$lib/types.js';
	import { timeToX, rangeWidth } from '$lib/stores/timeline.svelte.js';

	let { structure, width, offsetX }: {
		structure: EpisodeStructure;
		width: number;
		offsetX: number;
	} = $props();

	const segmentColors: Record<string, string> = {
		ColdOpen: '#4a5568',
		MainTitles: '#2d3748',
		Act: '#374151',
		CommercialBreak: '#1f2937',
		Tag: '#4a5568',
	};
</script>

<div class="structure-bar" style="height: {TIMELINE.STRUCTURE_BAR_HEIGHT_PX}px">
	<div class="structure-track" style="width: {width}px; transform: translateX(-{offsetX}px)">
		{#each structure.segments as segment}
			{@const segWidth = rangeWidth(segment.time_range)}
			{#if segWidth > 0}
				<div
					class="segment"
					style="
						left: {timeToX(segment.time_range.start_ms)}px;
						width: {segWidth}px;
						background: {segmentColors[segment.segment_type] ?? '#374151'};
					"
					title={segment.label}
				>
					<span class="segment-label">{segment.label}</span>
				</div>
			{/if}
		{/each}
	</div>
</div>

<style>
	.structure-bar {
		flex-shrink: 0;
		border-top: 1px solid var(--color-border-default);
		overflow: hidden;
		position: relative;
	}

	.structure-track {
		position: relative;
		height: 100%;
	}

	.segment {
		position: absolute;
		top: 0;
		height: 100%;
		display: flex;
		align-items: center;
		justify-content: center;
		border-right: 1px solid var(--color-border-default);
		overflow: hidden;
	}

	.segment-label {
		font-size: 0.7rem;
		color: var(--color-text-muted);
		text-overflow: ellipsis;
		overflow: hidden;
		white-space: nowrap;
		padding: 0 4px;
	}
</style>
