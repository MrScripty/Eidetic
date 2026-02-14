<script lang="ts">
	import { timelineState, timeToX, xToTime } from '$lib/stores/timeline.svelte.js';
	import { TIMELINE } from '$lib/types.js';

	let containerEl: HTMLDivElement | undefined = $state();
	let dragging = $state(false);

	let xPos = $derived(timeToX(timelineState.playheadMs) - timelineState.scrollX);
	let visible = $derived(xPos >= -12 && xPos <= timelineState.viewportWidth + 12);

	function clampTime(ms: number): number {
		return Math.max(0, Math.min(ms, TIMELINE.DURATION_MS));
	}

	function handlePointerDown(e: PointerEvent) {
		e.preventDefault();
		e.stopPropagation();
		dragging = true;
		(e.currentTarget as HTMLElement).setPointerCapture(e.pointerId);
	}

	function handlePointerMove(e: PointerEvent) {
		if (!dragging) return;
		// Get the timeline container (our parent) to compute position
		const parent = containerEl?.parentElement;
		if (!parent) return;
		const rect = parent.getBoundingClientRect();
		const localX = e.clientX - rect.left;
		timelineState.playheadMs = clampTime(xToTime(localX + timelineState.scrollX));
	}

	function handlePointerUp() {
		dragging = false;
	}
</script>

{#if visible}
	<!-- svelte-ignore a11y_no_static_element_interactions -->
	<div
		class="playhead"
		class:dragging
		style="left: {xPos}px"
		bind:this={containerEl}
		onpointerdown={handlePointerDown}
		onpointermove={handlePointerMove}
		onpointerup={handlePointerUp}
		onlostpointercapture={handlePointerUp}
	>
		<div class="playhead-head"></div>
		<div class="playhead-line"></div>
	</div>
{/if}

<style>
	.playhead {
		position: absolute;
		top: 0;
		bottom: 0;
		z-index: 20;
		display: flex;
		flex-direction: column;
		align-items: center;
		cursor: ew-resize;
		/* Widen the hit area with padding */
		padding: 0 6px;
		margin-left: -6px;
	}

	.playhead-head {
		width: 0;
		height: 0;
		border-left: 6px solid transparent;
		border-right: 6px solid transparent;
		border-top: 8px solid var(--color-playhead);
		flex-shrink: 0;
	}

	.playhead-line {
		width: 2px;
		flex: 1;
		background: var(--color-playhead);
	}

	.playhead.dragging .playhead-line {
		background: var(--color-playhead-active);
	}

	.playhead.dragging .playhead-head {
		border-top-color: var(--color-playhead-active);
	}
</style>
