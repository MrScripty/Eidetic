<script lang="ts">
	import type { ArcTrack as ArcTrackType } from '$lib/types.js';
	import { TIMELINE } from '$lib/types.js';
	import { xToTime, timelineState } from '$lib/stores/timeline.svelte.js';
	import { editorState } from '$lib/stores/editor.svelte.js';
	import { updateClip, createClip, deleteClip, splitClip } from '$lib/api.js';
	import BeatClip from './BeatClip.svelte';

	let { track, color, label, onconnectstart }: {
		track: ArcTrackType;
		color: string;
		label: string;
		onconnectstart: (clipId: string, x: number, y: number) => void;
	} = $props();

	function selectClip(clip: ArcTrackType['clips'][number]) {
		editorState.selectedClipId = clip.id;
		editorState.selectedClip = clip;
	}

	async function handleMove(clipId: string, startMs: number, endMs: number) {
		await updateClip(clipId, { start_ms: startMs, end_ms: endMs });
	}

	async function handleResize(clipId: string, startMs: number, endMs: number) {
		await updateClip(clipId, { start_ms: startMs, end_ms: endMs });
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
		await splitClip(clipId, atMs);
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
	<div class="track-label">{label}</div>
	<div class="track-lane" ondblclick={handleDblClick}>
		{#each track.clips as clip (clip.id)}
			<BeatClip
				{clip}
				{color}
				selected={editorState.selectedClipId === clip.id}
				onselect={() => selectClip(clip)}
				onmove={(s, e) => handleMove(clip.id, s, e)}
				onresize={(s, e) => handleResize(clip.id, s, e)}
				ondelete={() => handleDelete(clip.id)}
				onsplit={(atMs) => handleSplit(clip.id, atMs)}
				onconnectstart={onconnectstart}
			/>
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
</style>
