<script lang="ts">
	import type { BeatClip as BeatClipType, ContentStatus } from '$lib/types.js';
	import { TIMELINE } from '$lib/types.js';
	import { timeToX, xToTime } from '$lib/stores/timeline.svelte.js';

	let { clip, color, selected, onselect, onmove, onresize, ondelete, onsplit, onconnectstart }: {
		clip: BeatClipType;
		color: string;
		selected: boolean;
		onselect: () => void;
		onmove: (startMs: number, endMs: number) => void;
		onresize: (startMs: number, endMs: number) => void;
		ondelete: () => void;
		onsplit: (atMs: number) => void;
		onconnectstart: (clipId: string, x: number, y: number) => void;
	} = $props();

	let dragging = $state(false);
	let resizingSide: 'left' | 'right' | null = $state(null);
	let previewStartMs = $state(0);
	let previewEndMs = $state(0);
	let contextMenu: { x: number; y: number } | null = $state(null);

	// Use preview values during drag, actual values otherwise.
	let displayStart = $derived(dragging || resizingSide ? previewStartMs : clip.time_range.start_ms);
	let displayEnd = $derived(dragging || resizingSide ? previewEndMs : clip.time_range.end_ms);

	function statusColor(status: ContentStatus): string {
		switch (status) {
			case 'Empty': return 'var(--color-text-muted)';
			case 'NotesOnly': return 'var(--color-status-notes)';
			case 'Generating': return 'var(--color-status-generating)';
			case 'Generated': return 'var(--color-status-generated)';
			case 'UserRefined': return 'var(--color-status-written)';
			case 'UserWritten': return 'var(--color-status-written)';
			default: return 'var(--color-text-muted)';
		}
	}

	function handlePointerDown(e: PointerEvent) {
		if (resizingSide) return;
		e.preventDefault();
		e.stopPropagation();
		onselect();

		const target = e.currentTarget as HTMLElement;
		target.setPointerCapture(e.pointerId);
		dragging = true;
		previewStartMs = clip.time_range.start_ms;
		previewEndMs = clip.time_range.end_ms;
		const startClientX = e.clientX;
		const origStart = clip.time_range.start_ms;
		const duration = clip.time_range.end_ms - clip.time_range.start_ms;

		function onPointerMove(ev: PointerEvent) {
			const deltaPx = ev.clientX - startClientX;
			const deltaMs = xToTime(deltaPx);
			let newStart = Math.max(0, Math.round(origStart + deltaMs));
			let newEnd = newStart + duration;
			// Clamp to timeline bounds.
			if (newEnd > TIMELINE.DURATION_MS) {
				newEnd = TIMELINE.DURATION_MS;
				newStart = newEnd - duration;
			}
			previewStartMs = newStart;
			previewEndMs = newEnd;
		}

		function onPointerUp() {
			dragging = false;
			target.removeEventListener('pointermove', onPointerMove);
			target.removeEventListener('pointerup', onPointerUp);
			if (previewStartMs !== clip.time_range.start_ms || previewEndMs !== clip.time_range.end_ms) {
				onmove(previewStartMs, previewEndMs);
			}
		}

		target.addEventListener('pointermove', onPointerMove);
		target.addEventListener('pointerup', onPointerUp);
	}

	function handleResizeStart(e: PointerEvent, side: 'left' | 'right') {
		e.preventDefault();
		e.stopPropagation();

		const target = e.currentTarget as HTMLElement;
		target.setPointerCapture(e.pointerId);
		resizingSide = side;
		previewStartMs = clip.time_range.start_ms;
		previewEndMs = clip.time_range.end_ms;
		const startClientX = e.clientX;
		const origStart = clip.time_range.start_ms;
		const origEnd = clip.time_range.end_ms;

		function onPointerMove(ev: PointerEvent) {
			const deltaPx = ev.clientX - startClientX;
			const deltaMs = xToTime(deltaPx);
			if (side === 'left') {
				const newStart = Math.max(0, Math.round(origStart + deltaMs));
				if (origEnd - newStart >= 5000) { // min 5 seconds
					previewStartMs = newStart;
				}
			} else {
				let newEnd = Math.round(origEnd + deltaMs);
				newEnd = Math.min(newEnd, TIMELINE.DURATION_MS);
				if (newEnd - origStart >= 5000) {
					previewEndMs = newEnd;
				}
			}
		}

		function onPointerUp() {
			resizingSide = null;
			target.removeEventListener('pointermove', onPointerMove);
			target.removeEventListener('pointerup', onPointerUp);
			if (previewStartMs !== clip.time_range.start_ms || previewEndMs !== clip.time_range.end_ms) {
				onresize(previewStartMs, previewEndMs);
			}
		}

		target.addEventListener('pointermove', onPointerMove);
		target.addEventListener('pointerup', onPointerUp);
	}

	function handleContextMenu(e: MouseEvent) {
		e.preventDefault();
		e.stopPropagation();
		contextMenu = { x: e.clientX, y: e.clientY };

		function dismissMenu(ev: MouseEvent) {
			contextMenu = null;
			document.removeEventListener('click', dismissMenu);
		}
		// Dismiss on next click anywhere.
		setTimeout(() => document.addEventListener('click', dismissMenu), 0);
	}

	function handleDelete() {
		contextMenu = null;
		ondelete();
	}

	function handleSplit(e: MouseEvent) {
		// Split at the midpoint of the clip.
		const mid = Math.round((clip.time_range.start_ms + clip.time_range.end_ms) / 2);
		contextMenu = null;
		onsplit(mid);
	}

	function handleConnectStart(e: PointerEvent) {
		e.preventDefault();
		e.stopPropagation();
		const rect = (e.currentTarget as HTMLElement).getBoundingClientRect();
		onconnectstart(clip.id, rect.left + rect.width / 2, rect.top + rect.height / 2);
	}
</script>

<!-- svelte-ignore a11y_no_static_element_interactions -->
<div
	class="beat-clip"
	class:selected
	class:locked={clip.locked}
	class:dragging
	style="
		left: {timeToX(displayStart)}px;
		width: {timeToX(displayEnd) - timeToX(displayStart)}px;
		background: {color};
	"
	onpointerdown={handlePointerDown}
	oncontextmenu={handleContextMenu}
>
	<!-- Left resize handle -->
	<!-- svelte-ignore a11y_no_static_element_interactions -->
	<div
		class="resize-handle left"
		onpointerdown={(e) => handleResizeStart(e, 'left')}
	></div>

	<!-- Clip content -->
	<span class="status-dot" style="background: {statusColor(clip.content.status)}"></span>
	<span class="clip-name">{clip.name}</span>

	<!-- Connection handle -->
	<!-- svelte-ignore a11y_no_static_element_interactions -->
	<div
		class="connect-handle"
		onpointerdown={handleConnectStart}
	></div>

	<!-- Right resize handle -->
	<!-- svelte-ignore a11y_no_static_element_interactions -->
	<div
		class="resize-handle right"
		onpointerdown={(e) => handleResizeStart(e, 'right')}
	></div>
</div>

{#if contextMenu}
	<div
		class="context-menu"
		style="left: {contextMenu.x}px; top: {contextMenu.y}px"
	>
		<button onclick={handleSplit}>Split</button>
		<button class="danger" onclick={handleDelete}>Delete</button>
	</div>
{/if}

<style>
	.beat-clip {
		position: absolute;
		top: 4px;
		height: calc(100% - 8px);
		border: none;
		border-radius: 4px;
		cursor: grab;
		display: flex;
		align-items: center;
		gap: 4px;
		overflow: hidden;
		opacity: 0.85;
		transition: opacity 0.1s, outline 0.1s;
		padding: 0 8px;
		min-width: 0;
		user-select: none;
	}

	.beat-clip:hover {
		opacity: 1;
	}

	.beat-clip.selected {
		outline: 2px solid var(--color-accent);
		outline-offset: 1px;
		opacity: 1;
	}

	.beat-clip.locked {
		border: 1px dashed rgba(255, 255, 255, 0.4);
	}

	.beat-clip.dragging {
		cursor: grabbing;
		opacity: 0.7;
	}

	.resize-handle {
		position: absolute;
		top: 0;
		width: 5px;
		height: 100%;
		cursor: ew-resize;
		z-index: 1;
	}

	.resize-handle.left {
		left: 0;
	}

	.resize-handle.right {
		right: 0;
	}

	.resize-handle:hover {
		background: rgba(255, 255, 255, 0.2);
	}

	.status-dot {
		width: 6px;
		height: 6px;
		border-radius: 50%;
		flex-shrink: 0;
	}

	.clip-name {
		font-size: 0.7rem;
		color: rgba(0, 0, 0, 0.8);
		font-weight: 600;
		text-overflow: ellipsis;
		overflow: hidden;
		white-space: nowrap;
		flex: 1;
	}

	.connect-handle {
		width: 8px;
		height: 8px;
		border-radius: 50%;
		background: rgba(255, 255, 255, 0.4);
		border: 1px solid rgba(255, 255, 255, 0.6);
		flex-shrink: 0;
		cursor: crosshair;
	}

	.connect-handle:hover {
		background: rgba(255, 255, 255, 0.8);
	}

	.context-menu {
		position: fixed;
		z-index: 100;
		background: var(--color-bg-surface);
		border: 1px solid var(--color-border-default);
		border-radius: 4px;
		box-shadow: 0 4px 12px rgba(0, 0, 0, 0.3);
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

	.context-menu button.danger {
		color: #e55;
	}
</style>
