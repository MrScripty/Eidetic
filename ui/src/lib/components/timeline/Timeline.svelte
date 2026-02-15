<script lang="ts">
	import ArcTrack from './ArcTrack.svelte';
	import StructureBar from './StructureBar.svelte';
	import TimeRuler from './TimeRuler.svelte';
	import RelationshipLayer from './RelationshipLayer.svelte';
	import Playhead from './Playhead.svelte';
	import TimelineToolbar from './TimelineToolbar.svelte';
	import { timelineState, totalWidth, connectionDrag, timeToX, zoomToFit, zoomTo } from '$lib/stores/timeline.svelte.js';
	import { registerShortcut } from '$lib/stores/shortcuts.svelte.js';
	import { storyState } from '$lib/stores/story.svelte.js';
	import { TIMELINE, colorToHex } from '$lib/types.js';
	import type { StoryArc, TimelineGap } from '$lib/types.js';
	import { createRelationship, getGaps, closeGap, closeAllGaps, createArc, addTrack, removeTrack, deleteArc, setSubBeatsVisible } from '$lib/api.js';
	import { editorState } from '$lib/stores/editor.svelte.js';
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
				key: 'g', description: 'Close gap before selected clip', skipInInput: true,
				action: () => { handleCloseGapBeforeSelected(); },
			}),
			registerShortcut({
				key: 'g', shift: true, description: 'Close all gaps on track', skipInInput: true,
				action: () => { handleCloseAllGapsOnTrack(); },
			}),
			registerShortcut({
				key: 'c', description: 'Toggle character timeline', skipInInput: true,
				action: () => { characterTimelineState.visible = !characterTimelineState.visible; },
			}),
		];
		return () => unsubs.forEach(fn => fn());
	});

	async function handleCloseGapBeforeSelected() {
		const clipId = editorState.selectedClipId;
		if (!clipId || !timelineState.timeline) return;
		for (const track of timelineState.timeline.tracks) {
			const clip = track.clips.find(c => c.id === clipId);
			if (clip) {
				await closeGap(track.id, clip.time_range.start_ms);
				return;
			}
		}
	}

	async function handleCloseAllGapsOnTrack() {
		const clipId = editorState.selectedClipId;
		if (!clipId || !timelineState.timeline) return;
		for (const track of timelineState.timeline.tracks) {
			if (track.clips.some(c => c.id === clipId)) {
				await closeAllGaps(track.id);
				return;
			}
		}
	}

	async function handleAddTrack() {
		try {
			const arc = await createArc('New Arc', 'APlot');
			await addTrack(arc.id);
		} catch (e) {
			// Timeline refreshes via WS event on success.
		}
	}

	async function handleDeleteTrack(trackId: string) {
		if (!timelineState.timeline) return;
		const track = timelineState.timeline.tracks.find(t => t.id === trackId);
		if (!track) return;
		try {
			await removeTrack(trackId);
			await deleteArc(track.arc_id).catch(() => {});
		} catch (e) {
			// Timeline refreshes via WS event on success.
		}
	}

	async function handleToggleSubBeats(trackId: string, visible: boolean) {
		await setSubBeatsVisible(trackId, visible);
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

			const target = document.elementFromPoint(e.clientX, e.clientY);
			const clipEl = target?.closest('.beat-clip');
			if (clipEl && timelineState.timeline && connectionDrag.fromClipId) {
				const bounds = clipEl.getBoundingClientRect();
				for (const track of timelineState.timeline.tracks) {
					for (const clip of track.clips) {
						if (clip.id === connectionDrag.fromClipId) continue;
						const clipLeft = timeToX(clip.time_range.start_ms) - timelineState.scrollX;
						const clipRight = timeToX(clip.time_range.end_ms) - timelineState.scrollX;
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
							{@const arc = arcForTrack(track.arc_id)}
							<!-- svelte-ignore a11y_no_static_element_interactions -->
							<div
								class="track-label"
								style="height: {TIMELINE.TRACK_HEIGHT_PX}px"
								oncontextmenu={(e) => handleTrackContextMenu(e, track.id)}
							>
								{#if track.sub_beats.length > 0}
									<button class="subtrack-toggle" title="Toggle beats" onclick={() => handleToggleSubBeats(track.id, !track.sub_beats_visible)}>
										{track.sub_beats_visible ? '▼' : '▶'}
									</button>
								{/if}
								<span class="track-label-text">{arc?.name ?? 'Unknown'}</span>
								<button
									class="track-delete-btn"
									title="Delete Track"
									onclick={() => handleDeleteTrack(track.id)}
								>&times;</button>
							</div>
							{#if track.sub_beats_visible && track.sub_beats.length > 0}
								<div class="subtrack-label" style="height: {TIMELINE.BEAT_SUBTRACK_HEIGHT_PX}px">
									<span class="subtrack-label-text">beats</span>
								</div>
							{/if}
						{/each}
					{/if}
					<button class="add-track-row" onclick={handleAddTrack} title="Add new track">
						<span class="add-track-plus">+</span> Add Track
					</button>
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

				<!-- Arc tracks -->
				<div class="tracks">
					<div class="tracks-content" style="width: {totalWidth()}px; transform: translateX(-{timelineState.scrollX}px)">
						{#if timelineState.timeline}
							{#each timelineState.timeline.tracks as track}
								{@const arc = arcForTrack(track.arc_id)}
								<ArcTrack
									{track}
									color={arc ? colorToHex(arc.color) : 'var(--color-rel-default)'}
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

	/* ── Label column (fixed, always visible) ── */

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

	.track-label-text {
		overflow: hidden;
		text-overflow: ellipsis;
		white-space: nowrap;
	}

	.track-delete-btn {
		display: none;
		background: none;
		border: none;
		color: var(--color-text-muted);
		font-size: 0.85rem;
		line-height: 1;
		cursor: pointer;
		padding: 0 2px;
		flex-shrink: 0;
		border-radius: 2px;
	}

	.track-label:hover .track-delete-btn {
		display: block;
	}

	.track-delete-btn:hover {
		color: var(--color-danger);
		background: var(--color-danger-bg);
	}

	.subtrack-toggle {
		background: none;
		border: none;
		color: var(--color-text-muted);
		font-size: 0.55rem;
		line-height: 1;
		cursor: pointer;
		padding: 0 1px;
		flex-shrink: 0;
	}

	.subtrack-toggle:hover {
		color: var(--color-text-secondary);
	}

	.subtrack-label {
		display: flex;
		align-items: center;
		justify-content: flex-end;
		padding: 0 4px 0 8px;
		font-size: 0.65rem;
		color: var(--color-text-muted);
		overflow: hidden;
		border-bottom: 1px solid var(--color-border-subtle);
		background: var(--color-bg-secondary);
	}

	.subtrack-label-text {
		overflow: hidden;
		text-overflow: ellipsis;
		white-space: nowrap;
		font-style: italic;
	}

	.add-track-row {
		display: flex;
		align-items: center;
		gap: 4px;
		width: 100%;
		padding: 4px 8px;
		background: none;
		border: 1px dashed var(--color-border-subtle);
		border-radius: 4px;
		color: var(--color-text-muted);
		font-size: 0.7rem;
		cursor: pointer;
		margin: 4px 0;
		white-space: nowrap;
	}

	.add-track-row:hover {
		background: var(--color-bg-hover);
		border-color: var(--color-border-default);
		color: var(--color-text-secondary);
	}

	.add-track-plus {
		font-size: 0.9rem;
		line-height: 1;
	}

	/* ── Time content column ── */

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

	/* ── Context menu ── */

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
