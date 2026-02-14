<script lang="ts">
	import ArcTrack from './ArcTrack.svelte';
	import StructureBar from './StructureBar.svelte';
	import TimeRuler from './TimeRuler.svelte';
	import RelationshipLayer from './RelationshipLayer.svelte';
	import Playhead from './Playhead.svelte';
	import TimelineToolbar from './TimelineToolbar.svelte';
	import { timelineState, totalWidth, connectionDrag, timeToX, zoomToFit } from '$lib/stores/timeline.svelte.js';
	import { registerShortcut } from '$lib/stores/shortcuts.svelte.js';
	import { storyState } from '$lib/stores/story.svelte.js';
	import { TIMELINE, colorToHex } from '$lib/types.js';
	import type { StoryArc, TimelineGap } from '$lib/types.js';
	import { createRelationship, getGaps } from '$lib/api.js';

	let gaps = $state<TimelineGap[]>([]);
	let scrollbarEl: HTMLDivElement | undefined = $state();
	let scrollbarSyncing = false;
	let hasAutoFit = false;

	// Auto zoom-to-fit when timeline first loads and viewport is measured.
	$effect(() => {
		if (!hasAutoFit && timelineState.timeline && timelineState.viewportWidth > 0) {
			hasAutoFit = true;
			zoomToFit();
		}
	});

	// Refetch gaps when timeline changes.
	$effect(() => {
		if (timelineState.timeline) {
			getGaps().then(g => gaps = g).catch(() => {});
		}
	});

	function gapsForTrack(trackId: string): TimelineGap[] {
		return gaps.filter(g => g.track_id === trackId);
	}

	function arcForTrack(arcId: string): StoryArc | undefined {
		return storyState.arcs.find(a => a.id === arcId);
	}

	function onwheel(e: WheelEvent) {
		e.preventDefault();
		if (e.ctrlKey) {
			const factor = e.deltaY > 0 ? 0.9 : 1.1;
			timelineState.zoom = Math.max(0.005, Math.min(10, timelineState.zoom * factor));
		} else {
			const delta = Math.abs(e.deltaX) > Math.abs(e.deltaY) ? e.deltaX : e.deltaY;
			const maxScroll = Math.max(0, totalWidth() - timelineState.viewportWidth);
			timelineState.scrollX = Math.max(0, Math.min(maxScroll, timelineState.scrollX + delta));
		}
	}

	function onScrollbar(e: Event) {
		const el = e.currentTarget as HTMLDivElement;
		if (!scrollbarSyncing) {
			timelineState.scrollX = el.scrollLeft;
		}
	}

	// Keep scrollbar in sync when scrollX changes from wheel/zoom.
	$effect(() => {
		const sx = timelineState.scrollX;
		if (scrollbarEl && Math.abs(scrollbarEl.scrollLeft - sx) > 1) {
			scrollbarSyncing = true;
			scrollbarEl.scrollLeft = sx;
			scrollbarSyncing = false;
		}
	});

	// Register timeline tool shortcuts.
	$effect(() => {
		const unsubs = [
			registerShortcut({
				key: 'a', description: 'Selection tool', skipInInput: true,
				action: () => { timelineState.activeTool = 'select'; },
			}),
			registerShortcut({
				key: 'b', description: 'Blade tool', skipInInput: true,
				action: () => { timelineState.activeTool = 'blade'; },
			}),
			registerShortcut({
				key: 'n', description: 'Toggle snapping', skipInInput: true,
				action: () => { timelineState.snapping = !timelineState.snapping; },
			}),
		];
		return () => unsubs.forEach(fn => fn());
	});

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
							await createRelationship(connectionDrag.fromClipId, clip.id, 'Causal');
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

<div class="timeline-container" {onwheel} bind:clientWidth={timelineState.viewportWidth}>
	<TimelineToolbar />

	<!-- Time ruler at top -->
	<TimeRuler
		durationMs={TIMELINE.DURATION_MS}
		width={totalWidth()}
		offsetX={timelineState.scrollX}
	/>

	<!-- Playhead overlay (covers tracks area) -->
	<div class="timeline-content" style="position: relative; flex: 1; display: flex; flex-direction: column; overflow: hidden;">
	<Playhead />

	<!-- Relationship curves above tracks -->
	<div class="relationship-wrapper">
		<RelationshipLayer offsetX={timelineState.scrollX} />
	</div>

	<!-- Arc tracks -->
	<div class="tracks">
		<div class="tracks-content" style="width: {totalWidth()}px; transform: translateX(-{timelineState.scrollX}px)">
			{#if timelineState.timeline}
				{#each timelineState.timeline.tracks as track}
					{@const arc = arcForTrack(track.arc_id)}
					<ArcTrack
						{track}
						color={arc ? colorToHex(arc.color) : '#888'}
						label={arc?.name ?? 'Unknown'}
						gaps={gapsForTrack(track.id)}
						onconnectstart={handleConnectStart}
					/>
				{/each}
			{/if}
		</div>
	</div>

	<!-- Structure bar -->
	{#if timelineState.timeline}
		<StructureBar
			structure={timelineState.timeline.structure}
			width={totalWidth()}
			offsetX={timelineState.scrollX}
		/>
	{/if}

	</div>

	<!-- Horizontal scrollbar -->
	<div class="timeline-scrollbar" bind:this={scrollbarEl} onscroll={onScrollbar}>
		<div style="width: {totalWidth()}px; height: 1px;"></div>
	</div>
</div>

<style>
	.timeline-container {
		display: flex;
		flex-direction: column;
		width: 100%;
		height: 100%;
		overflow: hidden;
		user-select: none;
		position: relative;
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
		min-width: 0;
	}

	.tracks-content {
		position: relative;
		height: 100%;
	}

	.timeline-scrollbar {
		flex-shrink: 0;
		overflow-x: auto;
		overflow-y: hidden;
		height: 12px;
		background: var(--color-bg-secondary);
		border-top: 1px solid var(--color-border-subtle);
	}
</style>
