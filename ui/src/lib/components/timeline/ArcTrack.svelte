<script lang="ts">
	import type { ArcTrack as ArcTrackType, TimelineGap } from '$lib/types.js';
	import { TIMELINE } from '$lib/types.js';
	import { xToTime, timeToX, timelineState } from '$lib/stores/timeline.svelte.js';
	import { editorState } from '$lib/stores/editor.svelte.js';
	import { updateClip, createClip, deleteClip, splitClip, fillGap, closeGap, generateScript } from '$lib/api.js';
	import { startGeneration } from '$lib/stores/editor.svelte.js';
	import { notify } from '$lib/stores/notifications.svelte.js';
	import BeatClip from './BeatClip.svelte';
	import type { BeatClip as BeatClipType } from '$lib/types.js';

	let { track, color, label, gaps = [], onconnectstart, ondeletetrack }: {
		track: ArcTrackType;
		color: string;
		label: string;
		gaps?: TimelineGap[];
		onconnectstart: (clipId: string, x: number, y: number) => void;
		ondeletetrack: (trackId: string) => void;
	} = $props();

	// Viewport-aware clip filtering: only render clips whose pixel range
	// intersects the visible viewport [scrollX, scrollX + viewportWidth].
	let visibleClips = $derived.by(() => {
		const vw = timelineState.viewportWidth;
		if (vw <= 0) return track.clips; // Fallback: render all if not measured yet.
		const sx = timelineState.scrollX;
		return track.clips.filter((c: BeatClipType) => {
			const left = timeToX(c.time_range.start_ms);
			const right = timeToX(c.time_range.end_ms);
			return right >= sx && left <= sx + vw;
		});
	});

	let visibleGaps = $derived.by(() => {
		const vw = timelineState.viewportWidth;
		if (vw <= 0) return gaps;
		const sx = timelineState.scrollX;
		return gaps.filter((g) => {
			const left = timeToX(g.time_range.start_ms);
			const right = timeToX(g.time_range.end_ms);
			return right >= sx && left <= sx + vw;
		});
	});

	/** Sorted clips for boundary lookups. */
	let sortedClips = $derived(
		[...track.clips].sort((a, b) => a.time_range.start_ms - b.time_range.start_ms)
	);

	/** Get the boundary constraints for a clip (end of previous clip, start of next clip). */
	function clipBounds(clipId: string): { left: number; right: number } {
		const idx = sortedClips.findIndex(c => c.id === clipId);
		const left = idx > 0 ? sortedClips[idx - 1].time_range.end_ms : 0;
		const right = idx < sortedClips.length - 1 ? sortedClips[idx + 1].time_range.start_ms : TIMELINE.DURATION_MS;
		return { left, right };
	}

	function selectClip(clip: ArcTrackType['clips'][number]) {
		editorState.selectedClipId = clip.id;
		editorState.selectedClip = clip;
	}

	async function handleMove(clipId: string, startMs: number, endMs: number) {
		await updateClip(clipId, { start_ms: startMs, end_ms: endMs });
	}

	let regenPromptClipId: string | null = $state(null);

	async function handleResize(clipId: string, startMs: number, endMs: number) {
		await updateClip(clipId, { start_ms: startMs, end_ms: endMs });
		// Check if this clip has generated/refined script.
		const clip = track.clips.find(c => c.id === clipId);
		if (clip && (clip.content.generated_script || clip.content.user_refined_script)) {
			regenPromptClipId = clipId;
		}
	}

	async function handleRegenerate() {
		if (!regenPromptClipId) return;
		const clipId = regenPromptClipId;
		regenPromptClipId = null;
		startGeneration(clipId);
		await generateScript(clipId);
	}

	function dismissRegenPrompt() {
		regenPromptClipId = null;
	}

	async function handleDelete(clipId: string) {
		if (editorState.selectedClipId === clipId) {
			editorState.selectedClipId = null;
			editorState.selectedClip = null;
		}
		await deleteClip(clipId);
	}

	async function handleSplit(clipId: string, atMs: number) {
		if (editorState.selectedClipId === clipId) {
			editorState.selectedClipId = null;
			editorState.selectedClip = null;
		}
		try {
			await splitClip(clipId, atMs);
		} catch (e) {
			notify('error', `Split failed: ${e instanceof Error ? e.message : 'unknown error'}`);
		}
	}

	let gapContextMenu: { x: number; y: number; gap: TimelineGap } | null = $state(null);
	let trackContextMenu: { x: number; y: number } | null = $state(null);

	function handleTrackContextMenu(e: MouseEvent) {
		e.preventDefault();
		e.stopPropagation();
		trackContextMenu = { x: e.clientX, y: e.clientY };
		function dismiss() {
			trackContextMenu = null;
			document.removeEventListener('click', dismiss);
		}
		setTimeout(() => document.addEventListener('click', dismiss), 0);
	}

	async function handleFillGap(gap: TimelineGap) {
		await fillGap(gap.track_id, gap.time_range.start_ms, gap.time_range.end_ms);
	}

	async function handleCloseGap(gap: TimelineGap) {
		gapContextMenu = null;
		await closeGap(gap.track_id, gap.time_range.end_ms);
	}

	function handleGapContextMenu(e: MouseEvent, gap: TimelineGap) {
		e.preventDefault();
		e.stopPropagation();
		gapContextMenu = { x: e.clientX, y: e.clientY, gap };
		function dismiss() {
			gapContextMenu = null;
			document.removeEventListener('click', dismiss);
		}
		setTimeout(() => document.addEventListener('click', dismiss), 0);
	}

	async function handleDblClick(e: MouseEvent) {
		const target = e.currentTarget as HTMLElement;
		const rect = target.getBoundingClientRect();
		const localX = e.clientX - rect.left;
		const timeMs = xToTime(localX + timelineState.scrollX);
		const defaultDuration = 60_000;
		const startMs = Math.max(0, Math.round(timeMs - defaultDuration / 2));
		const endMs = Math.min(startMs + defaultDuration, TIMELINE.DURATION_MS);
		await createClip(track.id, 'New Beat', 'setup', startMs, endMs);
	}
</script>

<!-- svelte-ignore a11y_no_static_element_interactions -->
<div class="arc-track" style="height: {TIMELINE.TRACK_HEIGHT_PX}px">
	<!-- svelte-ignore a11y_no_static_element_interactions -->
	<div class="track-label" style="transform: translateX({timelineState.scrollX}px)" oncontextmenu={handleTrackContextMenu}>
		<span class="track-label-text">{label}</span>
		<button
			class="track-delete-btn"
			title="Delete Track"
			onclick={() => ondeletetrack(track.id)}
		>&times;</button>
	</div>
	<div class="track-lane" class:blade-mode={timelineState.activeTool === 'blade'} ondblclick={handleDblClick}>
		{#each visibleClips as clip (clip.id)}
			{@const bounds = clipBounds(clip.id)}
			<BeatClip
				{clip}
				{color}
				selected={editorState.selectedClipId === clip.id}
				leftBoundMs={bounds.left}
				rightBoundMs={bounds.right}
				onselect={() => selectClip(clip)}
				onmove={(s, e) => handleMove(clip.id, s, e)}
				onresize={(s, e) => handleResize(clip.id, s, e)}
				ondelete={() => handleDelete(clip.id)}
				onsplit={(atMs) => handleSplit(clip.id, atMs)}
				onconnectstart={onconnectstart}
			/>
		{/each}
		{#each visibleGaps as gap}
			<!-- svelte-ignore a11y_click_events_have_key_events -->
			<!-- svelte-ignore a11y_no_static_element_interactions -->
			<div
				class="gap-marker"
				style="left: {timeToX(gap.time_range.start_ms)}px; width: {timeToX(gap.time_range.end_ms) - timeToX(gap.time_range.start_ms)}px"
				title="Click to fill gap · Right-click for options"
				onclick={() => handleFillGap(gap)}
				oncontextmenu={(e) => handleGapContextMenu(e, gap)}
			>
				<span class="gap-label">+</span>
			</div>
		{/each}
	</div>
	{#if regenPromptClipId}
		<div class="regen-prompt">
			<span>Duration changed. Regenerate?</span>
			<button class="regen-btn" onclick={handleRegenerate}>Regenerate</button>
			<button class="keep-btn" onclick={dismissRegenPrompt}>Keep</button>
		</div>
	{/if}
</div>

{#if gapContextMenu}
	<div class="gap-context-menu" style="left: {gapContextMenu.x}px; top: {gapContextMenu.y}px">
		<button onclick={() => { handleFillGap(gapContextMenu!.gap); gapContextMenu = null; }}>Fill Gap</button>
		<button onclick={() => handleCloseGap(gapContextMenu!.gap)}>Close Gap</button>
	</div>
{/if}

{#if trackContextMenu}
	<div class="gap-context-menu" style="left: {trackContextMenu.x}px; top: {trackContextMenu.y}px">
		<button class="delete-track-btn" onclick={() => { trackContextMenu = null; ondeletetrack(track.id); }}>Delete Track</button>
	</div>
{/if}

<style>
	.arc-track {
		display: flex;
		align-items: center;
		border-bottom: 1px solid var(--color-border-subtle);
	}

	.track-label {
		width: 80px;
		flex-shrink: 0;
		padding: 0 4px 0 8px;
		font-size: 0.75rem;
		color: var(--color-text-secondary);
		display: flex;
		align-items: center;
		justify-content: flex-end;
		gap: 2px;
		overflow: hidden;
		position: relative;
		z-index: 2;
		background: var(--color-bg-primary);
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

	.track-lane {
		flex: 1;
		position: relative;
		height: 100%;
	}

	.track-lane.blade-mode {
		cursor: crosshair;
	}

	.gap-marker {
		position: absolute;
		top: 4px;
		bottom: 4px;
		background: var(--color-overlay-faint);
		border: 1px dashed var(--color-border-subtle);
		border-radius: 4px;
		cursor: pointer;
		display: flex;
		align-items: center;
		justify-content: center;
		transition: background 0.15s;
	}

	.gap-marker:hover {
		background: var(--color-overlay-light);
		border-color: var(--color-border-default);
	}

	.gap-label {
		font-size: 1rem;
		color: var(--color-text-muted);
		pointer-events: none;
	}

	.regen-prompt {
		position: absolute;
		right: 0;
		top: 50%;
		transform: translateY(-50%);
		display: flex;
		align-items: center;
		gap: 6px;
		padding: 2px 8px;
		background: var(--color-bg-surface);
		border: 1px solid var(--color-border-default);
		border-radius: 4px;
		font-size: 0.7rem;
		color: var(--color-text-secondary);
		z-index: 5;
		white-space: nowrap;
	}

	.regen-btn {
		font-size: 0.65rem;
		padding: 1px 8px;
		border-radius: 8px;
		border: 1px solid var(--color-accent);
		background: var(--color-bg-surface);
		color: var(--color-accent);
		cursor: pointer;
	}

	.keep-btn {
		font-size: 0.65rem;
		padding: 1px 8px;
		border-radius: 8px;
		border: 1px solid var(--color-border-default);
		background: var(--color-bg-surface);
		color: var(--color-text-secondary);
		cursor: pointer;
	}

	.gap-context-menu {
		position: fixed;
		z-index: 100;
		background: var(--color-bg-surface);
		border: 1px solid var(--color-border-default);
		border-radius: 4px;
		box-shadow: 0 4px 12px var(--color-shadow);
		padding: 4px 0;
		min-width: 120px;
	}

	.gap-context-menu button {
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

	.gap-context-menu button:hover {
		background: var(--color-bg-hover);
	}

	.delete-track-btn:hover {
		color: var(--color-danger) !important;
	}
</style>
