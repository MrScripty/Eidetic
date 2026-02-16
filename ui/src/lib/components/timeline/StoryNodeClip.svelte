<script lang="ts">
	import type { StoryNode, ContentStatus } from '$lib/types.js';
	import { TIMELINE, colorToHex } from '$lib/types.js';
	import { timeToX, xToTime, timelineState } from '$lib/stores/timeline.svelte.js';
	import { entitiesForNode } from '$lib/stores/story.svelte.js';

	let { node, color, selected, compact = false, leftBoundMs = 0, rightBoundMs = TIMELINE.DURATION_MS, onselect, onmove, onresize, ondelete, onsplit, onconnectstart }: {
		node: StoryNode;
		color: string;
		selected: boolean;
		compact?: boolean;
		/** Earliest allowed start_ms (end of previous node, or 0). */
		leftBoundMs?: number;
		/** Latest allowed end_ms (start of next node, or DURATION_MS). */
		rightBoundMs?: number;
		onselect: () => void;
		onmove: (startMs: number, endMs: number) => void;
		onresize: (startMs: number, endMs: number) => void;
		ondelete: () => void;
		onsplit: (atMs: number) => void;
		onconnectstart: (nodeId: string, x: number, y: number) => void;
	} = $props();

	let dragging = $state(false);
	let fitting = $state(false);
	let resizingSide: 'left' | 'right' | null = $state(null);
	let previewStartMs = $state(0);
	let previewEndMs = $state(0);
	let contextMenu: { x: number; y: number } | null = $state(null);

	// Blade cut preview: tracks cursor position as a ratio [0..1] within the node.
	let bladeHovering = $state(false);
	let bladeRatio = $state(0);

	// Use preview values during drag, actual values otherwise.
	let displayStart = $derived(dragging || resizingSide ? previewStartMs : node.time_range.start_ms);
	let displayEnd = $derived(dragging || resizingSide ? previewEndMs : node.time_range.end_ms);

	function statusColor(status: ContentStatus): string {
		switch (status) {
			case 'Empty': return 'var(--color-text-muted)';
			case 'NotesOnly': return 'var(--color-status-notes)';
			case 'Generating': return 'var(--color-status-generating)';
			case 'HasContent': return 'var(--color-status-generated)';
			default: return 'var(--color-text-muted)';
		}
	}

	function handleBladeClick(e: PointerEvent) {
		e.preventDefault();
		e.stopPropagation();
		const rect = (e.currentTarget as HTMLElement).closest('.node-clip')!.getBoundingClientRect();
		const localX = e.clientX - rect.left;
		const ratio = Math.max(0, Math.min(1, localX / rect.width));
		const duration = node.time_range.end_ms - node.time_range.start_ms;
		const splitMs = Math.round(node.time_range.start_ms + ratio * duration);
		if (splitMs - node.time_range.start_ms >= 5000 && node.time_range.end_ms - splitMs >= 5000) {
			onsplit(splitMs);
		}
	}

	function handlePointerDown(e: PointerEvent) {
		if (timelineState.activeTool === 'blade') {
			handleBladeClick(e);
			return;
		}
		if (resizingSide) return;
		e.preventDefault();
		e.stopPropagation();
		onselect();

		const target = e.currentTarget as HTMLElement;
		target.setPointerCapture(e.pointerId);
		dragging = true;
		previewStartMs = node.time_range.start_ms;
		previewEndMs = node.time_range.end_ms;
		const startClientX = e.clientX;
		const origStart = node.time_range.start_ms;
		const duration = node.time_range.end_ms - node.time_range.start_ms;

		function onPointerMove(ev: PointerEvent) {
			const deltaPx = ev.clientX - startClientX;
			const deltaMs = xToTime(deltaPx);

			if (ev.shiftKey) {
				fitting = true;
				let newStart = Math.max(leftBoundMs, Math.round(origStart + deltaMs));
				let newEnd = newStart + duration;
				if (newEnd > rightBoundMs) newEnd = rightBoundMs;
				if (newStart < leftBoundMs) newStart = leftBoundMs;
				if (newEnd - newStart >= 5000) {
					previewStartMs = newStart;
					previewEndMs = newEnd;
				}
			} else {
				fitting = false;
				let newStart = Math.max(leftBoundMs, Math.round(origStart + deltaMs));
				let newEnd = newStart + duration;
				if (newEnd > rightBoundMs) {
					newEnd = rightBoundMs;
					newStart = newEnd - duration;
				}
				if (newStart < leftBoundMs) {
					newStart = leftBoundMs;
					newEnd = newStart + duration;
				}
				previewStartMs = newStart;
				previewEndMs = newEnd;
			}
		}

		function onPointerUp() {
			const wasFitting = fitting;
			dragging = false;
			fitting = false;
			target.removeEventListener('pointermove', onPointerMove);
			target.removeEventListener('pointerup', onPointerUp);
			if (previewStartMs !== node.time_range.start_ms || previewEndMs !== node.time_range.end_ms) {
				const durationChanged = (previewEndMs - previewStartMs) !== duration;
				if (wasFitting && durationChanged) {
					onresize(previewStartMs, previewEndMs);
				} else {
					onmove(previewStartMs, previewEndMs);
				}
			}
		}

		target.addEventListener('pointermove', onPointerMove);
		target.addEventListener('pointerup', onPointerUp);
	}

	function handleResizeStart(e: PointerEvent, side: 'left' | 'right') {
		if (timelineState.activeTool === 'blade') {
			handleBladeClick(e);
			return;
		}
		e.preventDefault();
		e.stopPropagation();

		const target = e.currentTarget as HTMLElement;
		target.setPointerCapture(e.pointerId);
		resizingSide = side;
		previewStartMs = node.time_range.start_ms;
		previewEndMs = node.time_range.end_ms;
		const startClientX = e.clientX;
		const origStart = node.time_range.start_ms;
		const origEnd = node.time_range.end_ms;

		function onPointerMove(ev: PointerEvent) {
			const deltaPx = ev.clientX - startClientX;
			const deltaMs = xToTime(deltaPx);
			if (side === 'left') {
				const newStart = Math.max(leftBoundMs, Math.round(origStart + deltaMs));
				if (origEnd - newStart >= 5000) {
					previewStartMs = newStart;
				}
			} else {
				let newEnd = Math.round(origEnd + deltaMs);
				newEnd = Math.min(newEnd, rightBoundMs);
				if (newEnd - origStart >= 5000) {
					previewEndMs = newEnd;
				}
			}
		}

		function onPointerUp() {
			resizingSide = null;
			target.removeEventListener('pointermove', onPointerMove);
			target.removeEventListener('pointerup', onPointerUp);
			if (previewStartMs !== node.time_range.start_ms || previewEndMs !== node.time_range.end_ms) {
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
		setTimeout(() => document.addEventListener('click', dismissMenu), 0);
	}

	function handleDelete() {
		contextMenu = null;
		ondelete();
	}

	function handleSplit(e: MouseEvent) {
		const mid = Math.round((node.time_range.start_ms + node.time_range.end_ms) / 2);
		contextMenu = null;
		onsplit(mid);
	}

	function handleConnectStart(e: PointerEvent) {
		if (timelineState.activeTool === 'blade') {
			handleBladeClick(e);
			return;
		}
		e.preventDefault();
		e.stopPropagation();
		const rect = (e.currentTarget as HTMLElement).getBoundingClientRect();
		onconnectstart(node.id, rect.left + rect.width / 2, rect.top + rect.height / 2);
	}

	function handleBladeMove(e: PointerEvent) {
		if (timelineState.activeTool !== 'blade') {
			bladeHovering = false;
			return;
		}
		const el = (e.currentTarget as HTMLElement).closest('.node-clip');
		if (!el) return;
		const rect = el.getBoundingClientRect();
		bladeRatio = Math.max(0, Math.min(1, (e.clientX - rect.left) / rect.width));
		bladeHovering = true;
	}

	function handleBladeLeave() {
		bladeHovering = false;
	}

	const nodeEntities = $derived(entitiesForNode(node.id));
	const entityDots = $derived(nodeEntities.slice(0, 4));
	const entityOverflow = $derived(Math.max(0, nodeEntities.length - 4));
</script>

<!-- svelte-ignore a11y_no_static_element_interactions -->
<div
	class="node-clip"
	class:selected
	class:locked={node.locked}
	class:dragging
	class:fitting
	class:compact
	class:blade-mode={timelineState.activeTool === 'blade'}
	style="
		left: {timeToX(displayStart)}px;
		width: {timeToX(displayEnd) - timeToX(displayStart)}px;
		background: {color};
	"
	onpointerdown={handlePointerDown}
	onpointermove={handleBladeMove}
	onpointerleave={handleBladeLeave}
	oncontextmenu={handleContextMenu}
>
	<!-- Left resize handle -->
	<!-- svelte-ignore a11y_no_static_element_interactions -->
	<div
		class="resize-handle left"
		onpointerdown={(e) => handleResizeStart(e, 'left')}
	></div>

	<!-- Node content -->
	<span class="status-dot" style="background: {statusColor(node.content.status)}"></span>
	<span class="clip-name">{node.name}</span>

	{#if !compact}
		<!-- Connection handle -->
		<!-- svelte-ignore a11y_no_static_element_interactions -->
		<div
			class="connect-handle"
			onpointerdown={handleConnectStart}
		></div>
	{/if}

	<!-- Right resize handle -->
	<!-- svelte-ignore a11y_no_static_element_interactions -->
	<div
		class="resize-handle right"
		onpointerdown={(e) => handleResizeStart(e, 'right')}
	></div>

	<!-- Entity indicator dots -->
	{#if !compact && entityDots.length > 0}
		<div class="entity-dots">
			{#each entityDots as entity (entity.id)}
				<span class="entity-dot" style="background: {colorToHex(entity.color)}" title={entity.name}></span>
			{/each}
			{#if entityOverflow > 0}
				<span class="entity-dot-overflow">+{entityOverflow}</span>
			{/if}
		</div>
	{/if}

	<!-- Blade cut preview line -->
	{#if bladeHovering && timelineState.activeTool === 'blade'}
		<div class="blade-preview" style="left: {bladeRatio * 100}%"></div>
	{/if}
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
	.node-clip {
		position: absolute;
		top: 4px;
		height: calc(100% - 8px);
		border: 1px solid var(--color-shadow-medium);
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
		box-shadow: inset 1px 0 0 var(--color-overlay-medium), inset -1px 0 0 var(--color-overlay-medium);
	}

	.node-clip:hover {
		opacity: 1;
	}

	.node-clip.selected {
		outline: 2px solid var(--color-accent);
		outline-offset: 1px;
		opacity: 1;
	}

	.node-clip.locked {
		border: 1px dashed var(--color-overlay-bright);
	}

	.node-clip.dragging {
		cursor: grabbing;
		opacity: 0.7;
	}

	.node-clip.fitting {
		outline: 2px dashed var(--color-fitting-outline);
		outline-offset: 1px;
	}

	.node-clip.blade-mode {
		cursor: crosshair;
	}

	.resize-handle {
		position: absolute;
		top: 0;
		width: 5px;
		height: 100%;
		cursor: ew-resize;
		z-index: 1;
		background: var(--color-overlay-subtle);
	}

	.resize-handle.left {
		left: 0;
		border-right: 1px solid var(--color-overlay-border);
	}

	.resize-handle.right {
		right: 0;
		border-left: 1px solid var(--color-overlay-border);
	}

	.resize-handle:hover {
		background: var(--color-overlay-strong);
	}

	.status-dot {
		width: 6px;
		height: 6px;
		border-radius: 50%;
		flex-shrink: 0;
	}

	.clip-name {
		font-size: 0.7rem;
		color: var(--color-shadow-heavy);
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
		background: var(--color-overlay-bright);
		border: 1px solid var(--color-overlay-vivid);
		flex-shrink: 0;
		cursor: crosshair;
	}

	.connect-handle:hover {
		background: var(--color-overlay-intense);
	}

	.entity-dots {
		position: absolute;
		bottom: 2px;
		left: 8px;
		display: flex;
		gap: 2px;
		align-items: center;
		pointer-events: none;
	}

	.entity-dot {
		width: 5px;
		height: 5px;
		border-radius: 50%;
		flex-shrink: 0;
	}

	.entity-dot-overflow {
		font-size: 0.5rem;
		color: var(--color-overlay-bright);
		line-height: 1;
	}

	.blade-preview {
		position: absolute;
		top: 0;
		bottom: 0;
		width: 2px;
		background: var(--color-blade);
		pointer-events: none;
		z-index: 2;
		box-shadow: 0 0 4px var(--color-blade-glow);
	}

	.node-clip.compact {
		padding: 0 4px;
		gap: 2px;
	}

	.node-clip.compact .clip-name {
		font-size: 0.6rem;
	}

	.node-clip.compact .status-dot {
		width: 4px;
		height: 4px;
	}

	.node-clip.compact .resize-handle {
		width: 3px;
	}

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

	.context-menu button.danger {
		color: var(--color-danger);
	}
</style>
