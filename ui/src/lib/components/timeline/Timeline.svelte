<script lang="ts">
	import ArcTrack from './ArcTrack.svelte';
	import StructureBar from './StructureBar.svelte';
	import TimeRuler from './TimeRuler.svelte';
	import RelationshipLayer from './RelationshipLayer.svelte';
	import { timelineState, totalWidth, connectionDrag, timeToX } from '$lib/stores/timeline.svelte.js';
	import { storyState } from '$lib/stores/story.svelte.js';
	import { TIMELINE, colorToHex } from '$lib/types.js';
	import type { StoryArc } from '$lib/types.js';
	import { createRelationship } from '$lib/api.js';

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

	function handleConnectStart(clipId: string, x: number, y: number) {
		connectionDrag.active = true;
		connectionDrag.fromClipId = clipId;
		connectionDrag.fromX = x;
		connectionDrag.fromY = y;
		connectionDrag.currentX = x;
		connectionDrag.currentY = y;

		function onPointerMove(e: PointerEvent) {
			connectionDrag.currentX = e.clientX;
			connectionDrag.currentY = e.clientY;
		}

		async function onPointerUp(e: PointerEvent) {
			document.removeEventListener('pointermove', onPointerMove);
			document.removeEventListener('pointerup', onPointerUp);
			connectionDrag.active = false;

			// Find clip under cursor via DOM hit-testing.
			const target = document.elementFromPoint(e.clientX, e.clientY);
			const clipEl = target?.closest('.beat-clip');
			if (clipEl && timelineState.timeline && connectionDrag.fromClipId) {
				const bounds = clipEl.getBoundingClientRect();
				// Find which clip this element belongs to by matching position.
				for (const track of timelineState.timeline.tracks) {
					for (const clip of track.clips) {
						if (clip.id === connectionDrag.fromClipId) continue;
						const clipLeft = timeToX(clip.time_range.start_ms) - timelineState.scrollX;
						const clipRight = timeToX(clip.time_range.end_ms) - timelineState.scrollX;
						// Check rough position match.
						if (Math.abs(bounds.width - (clipRight - clipLeft)) < 10) {
							await createRelationship(connectionDrag.fromClipId, clip.id, 'causal');
							connectionDrag.fromClipId = null;
							return;
						}
					}
				}
			}
			connectionDrag.fromClipId = null;
		}

		document.addEventListener('pointermove', onPointerMove);
		document.addEventListener('pointerup', onPointerUp);
	}
</script>

<div class="timeline-container" {onwheel}>
	<!-- Relationship curves above tracks -->
	<div class="relationship-wrapper">
		<RelationshipLayer offsetX={timelineState.scrollX} />
	</div>

	<!-- Arc tracks -->
	<div class="tracks" style="width: {totalWidth()}px; transform: translateX(-{timelineState.scrollX}px)">
		{#if timelineState.timeline}
			{#each timelineState.timeline.tracks as track}
				{@const arc = arcForTrack(track.arc_id)}
				<ArcTrack
					{track}
					color={arc ? colorToHex(arc.color) : '#888'}
					label={arc?.name ?? 'Unknown'}
					onconnectstart={handleConnectStart}
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

	.relationship-wrapper {
		flex-shrink: 0;
		height: 40px;
		border-bottom: 1px solid var(--color-border-subtle);
		background: var(--color-bg-primary);
		overflow: visible;
	}

	.tracks {
		flex: 1;
		overflow: hidden;
		position: relative;
	}
</style>
