<script lang="ts">
	import type { RelationshipType } from '$lib/types.js';
	import { TIMELINE } from '$lib/types.js';
	import { timelineState, timeToX, totalWidth, findNode } from '$lib/stores/timeline.svelte.js';
	import { connectionDrag } from '$lib/stores/timeline.svelte.js';
	import RelationshipArc from './RelationshipArc.svelte';

	let { offsetX }: { offsetX: number } = $props();

	function relationshipColor(type: RelationshipType): string {
		if (type === 'Causal') return 'var(--color-rel-causal)';
		if (type === 'Thematic') return 'var(--color-rel-thematic)';
		if (typeof type === 'object' && 'Convergence' in type) return 'var(--color-rel-convergence)';
		if (typeof type === 'object' && 'EntityDrives' in type) return 'var(--color-rel-character)';
		return 'var(--color-rel-default)';
	}

	function trackYOffset(trackIdx: number): number {
		return trackIdx * TIMELINE.TRACK_HEIGHT_PX;
	}

	function nodeCenter(nodeId: string): { x: number; y: number } | null {
		if (!timelineState.timeline) return null;
		const node = findNode(nodeId);
		if (!node) return null;
		const trackIdx = timelineState.timeline.tracks.findIndex(t => t.level === node.level);
		if (trackIdx === -1) return null;
		const midMs = (node.time_range.start_ms + node.time_range.end_ms) / 2;
		return {
			x: timeToX(midMs),
			y: trackYOffset(trackIdx) + TIMELINE.TRACK_HEIGHT_PX / 2,
		};
	}
</script>

<svg
	class="relationship-layer"
	style="width: {totalWidth()}px; transform: translateX(-{offsetX}px)"
>
	{#if timelineState.timeline}
		{#each timelineState.timeline.relationships as rel (rel.id)}
			{@const from = nodeCenter(rel.from_node)}
			{@const to = nodeCenter(rel.to_node)}
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
