<script lang="ts">
	import ArcTrack from './ArcTrack.svelte';
	import StructureBar from './StructureBar.svelte';
	import TimeRuler from './TimeRuler.svelte';
	import { timelineState, totalWidth } from '$lib/stores/timeline.svelte.js';
	import { storyState } from '$lib/stores/story.svelte.js';
	import { TIMELINE, colorToHex } from '$lib/types.js';
	import type { StoryArc } from '$lib/types.js';

	function arcForTrack(arcId: string): StoryArc | undefined {
		return storyState.arcs.find(a => a.id === arcId);
	}

	function onwheel(e: WheelEvent) {
		if (e.ctrlKey) {
			e.preventDefault();
			const factor = e.deltaY > 0 ? 0.9 : 1.1;
			timelineState.zoom = Math.max(0.1, Math.min(10, timelineState.zoom * factor));
		} else {
			timelineState.scrollX = Math.max(0, timelineState.scrollX + e.deltaX);
		}
	}
</script>

<div class="timeline-container" {onwheel}>
	<!-- Relationship layer placeholder (Sprint 2) -->
	<div class="relationship-layer" style="height: 40px"></div>

	<!-- Arc tracks -->
	<div class="tracks" style="width: {totalWidth()}px; transform: translateX(-{timelineState.scrollX}px)">
		{#if timelineState.timeline}
			{#each timelineState.timeline.tracks as track}
				{@const arc = arcForTrack(track.arc_id)}
				<ArcTrack
					{track}
					color={arc ? colorToHex(arc.color) : '#888'}
					label={arc?.name ?? 'Unknown'}
				/>
			{/each}
		{/if}
	</div>

	<!-- Structure bar -->
	{#if timelineState.timeline}
		<StructureBar
			structure={timelineState.timeline.structure}
			width={totalWidth()}
			offsetX={timelineState.scrollX}
		/>
	{/if}

	<!-- Time ruler -->
	<TimeRuler
		durationMs={TIMELINE.DURATION_MS}
		width={totalWidth()}
		offsetX={timelineState.scrollX}
	/>
</div>

<style>
	.timeline-container {
		display: flex;
		flex-direction: column;
		height: 100%;
		overflow: hidden;
		user-select: none;
	}

	.relationship-layer {
		flex-shrink: 0;
		border-bottom: 1px solid var(--color-border-subtle);
		background: var(--color-bg-primary);
	}

	.tracks {
		flex: 1;
		overflow: hidden;
		position: relative;
	}
</style>
