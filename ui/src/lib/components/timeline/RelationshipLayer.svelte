<script lang="ts">
	import type { RelationshipType } from '$lib/types.js';
	import { TIMELINE } from '$lib/types.js';
	import { timelineState, timeToX, totalWidth } from '$lib/stores/timeline.svelte.js';
	import { connectionDrag } from '$lib/stores/timeline.svelte.js';
	import RelationshipArc from './RelationshipArc.svelte';

	let { offsetX }: { offsetX: number } = $props();

	function relationshipColor(type: RelationshipType): string {
		if (type === 'Causal') return '#6495ed';
		if (type === 'Thematic') return '#a855f7';
		if (typeof type === 'object' && 'Convergence' in type) return '#ffb347';
		if (typeof type === 'object' && 'CharacterDrives' in type) return '#77dd77';
		return '#888';
	}

	function clipCenter(clipId: string): { x: number; y: number } | null {
		if (!timelineState.timeline) return null;
		for (let trackIdx = 0; trackIdx < timelineState.timeline.tracks.length; trackIdx++) {
			const track = timelineState.timeline.tracks[trackIdx]!;
			const clip = track.clips.find(c => c.id === clipId);
			if (clip) {
				const midMs = (clip.time_range.start_ms + clip.time_range.end_ms) / 2;
				return {
					x: timeToX(midMs),
					y: trackIdx * TIMELINE.TRACK_HEIGHT_PX + TIMELINE.TRACK_HEIGHT_PX / 2,
				};
			}
		}
		return null;
	}
</script>

<svg
	class="relationship-layer"
	style="width: {totalWidth()}px; transform: translateX(-{offsetX}px)"
>
	{#if timelineState.timeline}
		{#each timelineState.timeline.relationships as rel (rel.id)}
			{@const from = clipCenter(rel.from_clip)}
			{@const to = clipCenter(rel.to_clip)}
			{#if from && to}
				<RelationshipArc
					fromX={from.x}
					fromY={from.y}
					toX={to.x}
					toY={to.y}
					color={relationshipColor(rel.relationship_type)}
				/>
			{/if}
		{/each}
	{/if}

	<!-- Temporary line during drag-to-connect -->
	{#if connectionDrag.active}
		<line
			x1={connectionDrag.fromX}
			y1={connectionDrag.fromY}
			x2={connectionDrag.currentX}
			y2={connectionDrag.currentY}
			stroke="var(--color-accent)"
			stroke-width="2"
			stroke-dasharray="6 3"
			opacity="0.6"
		/>
	{/if}
</svg>

<style>
	.relationship-layer {
		position: relative;
		height: 100%;
		overflow: visible;
	}
</style>
