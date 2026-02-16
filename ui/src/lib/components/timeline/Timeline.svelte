<script lang="ts">
	import LevelTrack from './LevelTrack.svelte';
	import StructureBar from './StructureBar.svelte';
	import TimeRuler from './TimeRuler.svelte';
	import RelationshipLayer from './RelationshipLayer.svelte';
	import Playhead from './Playhead.svelte';
	import TimelineToolbar from './TimelineToolbar.svelte';
	import { timelineState, totalWidth, connectionDrag, timeToX, zoomToFit, zoomTo } from '$lib/stores/timeline.svelte.js';
	import { registerShortcut } from '$lib/stores/shortcuts.svelte.js';
	import { TIMELINE } from '$lib/types.js';
	import type { TimelineGap, StoryLevel } from '$lib/types.js';
	import { createRelationship, getGaps, removeTrack } from '$lib/api.js';
	import { characterTimelineState } from '$lib/stores/characterTimeline.svelte.js';

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

	function gapsForLevel(level: StoryLevel): TimelineGap[] {
		return gaps.filter(g => g.level === level);
	}

	function onwheel(e: WheelEvent) {
		e.preventDefault();
		if (e.ctrlKey) {
			const factor = e.deltaY > 0 ? 0.9 : 1.1;
			zoomTo(timelineState.zoom * factor);
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
			registerShortcut({
				key: 'c', description: 'Toggle character timeline', skipInInput: true,
				action: () => { characterTimelineState.visible = !characterTimelineState.visible; },
			}),
		];
		return () => unsubs.forEach(fn => fn());
	});

	function handleConnectStart(nodeId: string, x: number, y: number) {
		connectionDrag.active = true;
		connectionDrag.fromNodeId = nodeId;
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

			const target = document.elementFromPoint(e.clientX, e.clientY);
			const clipEl = target?.closest('.node-clip');
			if (clipEl && timelineState.timeline && connectionDrag.fromNodeId) {
				// Find which node was dropped on by checking bounds
				for (const node of timelineState.timeline.nodes) {
					if (node.id === connectionDrag.fromNodeId) continue;
					const nodeLeft = timeToX(node.time_range.start_ms) - timelineState.scrollX;
					const nodeRight = timeToX(node.time_range.end_ms) - timelineState.scrollX;
					const bounds = clipEl.getBoundingClientRect();
					if (Math.abs(bounds.width - (nodeRight - nodeLeft)) < 10) {
						await createRelationship(connectionDrag.fromNodeId, node.id, 'Causal');
						connectionDrag.fromNodeId = null;
						return;
					}
				}
			}
			connectionDrag.fromNodeId = null;
		}

		document.addEventListener('pointermove', onPointerMove);
		document.addEventListener('pointerup', onPointerUp);
	}

	// Track label context menu state.
	let trackContextMenu: { x: number; y: number; trackId: string } | null = $state(null);

	function handleTrackContextMenu(e: MouseEvent, trackId: string) {
		e.preventDefault();
		e.stopPropagation();
		trackContextMenu = { x: e.clientX, y: e.clientY, trackId };
		function dismiss() {
			trackContextMenu = null;
			document.removeEventListener('click', dismiss);
		}
		setTimeout(() => document.addEventListener('click', dismiss), 0);
	}

	async function handleDeleteTrack(trackId: string) {
		try {
			await removeTrack(trackId);
		} catch (e) {
			// Timeline refreshes via WS event on success.
		}
	}
</script>

<div class="timeline-container">
	<TimelineToolbar />

	<div class="timeline-body" {onwheel}>
		<!-- Fixed label column (always visible, not affected by scroll/zoom) -->
		<div class="label-column">
			<!-- Spacer matching time ruler height -->
			<div class="label-spacer ruler-spacer"></div>

			<!-- Labels matching timeline-content rows -->
			<div class="label-content">
				<!-- Spacer matching relationship wrapper -->
				<div class="label-spacer rel-spacer"></div>

				<!-- Track labels -->
				<div class="labels-tracks">
					{#if timelineState.timeline}
						{#each timelineState.timeline.tracks as track}
							<!-- svelte-ignore a11y_no_static_element_interactions -->
							<div
								class="track-label"
								style="height: {track.collapsed ? 0 : TIMELINE.TRACK_HEIGHT_PX}px"
								class:collapsed={track.collapsed}
								oncontextmenu={(e) => handleTrackContextMenu(e, track.id)}
							>
								<span class="track-label-text">{track.label}</span>
							</div>
						{/each}
					{/if}
				</div>

				<!-- Spacer matching structure bar -->
				<div class="label-spacer structure-spacer"></div>
			</div>

			<!-- Spacer matching scrollbar -->
			<div class="label-spacer scrollbar-spacer"></div>
		</div>

		<!-- Scrollable time content column -->
		<div class="time-column" bind:clientWidth={timelineState.viewportWidth}>
			<!-- Time ruler at top -->
			<TimeRuler
				durationMs={TIMELINE.DURATION_MS}
				width={totalWidth()}
				offsetX={timelineState.scrollX}
			/>

			<!-- Time-aligned content (playhead, relationships, tracks, structure) -->
			<div class="timeline-content">
				<Playhead />

				<!-- Relationship curves above tracks -->
				<div class="relationship-wrapper">
					<RelationshipLayer offsetX={timelineState.scrollX} />
				</div>

				<!-- Level tracks -->
				<div class="tracks">
					<div class="tracks-content" style="width: {totalWidth()}px; transform: translateX(-{timelineState.scrollX}px)">
						{#if timelineState.timeline}
							{#each timelineState.timeline.tracks as track}
								<LevelTrack
									{track}
									gaps={gapsForLevel(track.level)}
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
	</div>
</div>

<!-- Track context menu (fixed position overlay) -->
{#if trackContextMenu}
	<div class="context-menu" style="left: {trackContextMenu.x}px; top: {trackContextMenu.y}px">
		<button class="delete-track-btn" onclick={() => { const id = trackContextMenu!.trackId; trackContextMenu = null; handleDeleteTrack(id); }}>Delete Track</button>
	</div>
{/if}

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

	.timeline-body {
		display: flex;
		flex: 1;
		overflow: hidden;
	}

	/* -- Label column (fixed, always visible) -- */

	.label-column {
		width: 80px;
		flex-shrink: 0;
		display: flex;
		flex-direction: column;
		background: var(--color-bg-primary);
		border-right: 1px solid var(--color-border-default);
		z-index: 5;
	}

	.label-spacer {
		flex-shrink: 0;
	}

	.ruler-spacer {
		height: 28px; /* TIMELINE.TIME_RULER_HEIGHT_PX */
		border-bottom: 1px solid var(--color-border-default);
	}

	.rel-spacer {
		height: 40px;
		border-bottom: 1px solid var(--color-border-subtle);
	}

	.structure-spacer {
		height: 33px; /* STRUCTURE_BAR_HEIGHT_PX + 1px border */
		border-top: 1px solid var(--color-border-default);
	}

	.scrollbar-spacer {
		height: 12px;
		border-top: 1px solid var(--color-border-subtle);
	}

	.label-content {
		flex: 1;
		display: flex;
		flex-direction: column;
		overflow: hidden;
	}

	.labels-tracks {
		flex: 1;
		overflow: hidden;
	}

	.track-label {
		display: flex;
		align-items: center;
		justify-content: flex-end;
		gap: 2px;
		padding: 0 4px 0 8px;
		font-size: 0.75rem;
		color: var(--color-text-secondary);
		overflow: hidden;
		border-bottom: 1px solid var(--color-border-subtle);
	}

	.track-label.collapsed {
		display: none;
	}

	.track-label-text {
		overflow: hidden;
		text-overflow: ellipsis;
		white-space: nowrap;
	}

	/* -- Time content column -- */

	.time-column {
		flex: 1;
		display: flex;
		flex-direction: column;
		overflow: hidden;
		min-width: 0;
	}

	.timeline-content {
		position: relative;
		flex: 1;
		display: flex;
		flex-direction: column;
		overflow: hidden;
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

	/* -- Context menu -- */

	.context-menu {
		position: fixed;
		z-index: 100;
		background: var(--color-bg-surface);
		border: 1px solid var(--color-border-default);
		border-radius: 4px;
		box-shadow: 0 4px 12px var(--color-shadow);
		padding: 4px 0;
		min-width: 120px;
	}

	.context-menu button {
		display: block;
		width: 100%;
		padding: 6px 12px;
		background: none;
		border: none;
		color: var(--color-text-primary);
		font-size: 0.85rem;
		cursor: pointer;
		text-align: left;
	}

	.context-menu button:hover {
		background: var(--color-bg-hover);
	}

	.delete-track-btn:hover {
		color: var(--color-danger) !important;
	}
</style>
