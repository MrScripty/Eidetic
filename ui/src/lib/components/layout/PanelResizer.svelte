<script lang="ts">
	let { min = 100, position = $bindable(300) } = $props();

	let dragging = $state(false);
	let startY = 0;
	let startPos = 0;

	function onpointerdown(e: PointerEvent) {
		dragging = true;
		startY = e.clientY;
		startPos = position;
		(e.target as HTMLElement).setPointerCapture(e.pointerId);
	}

	function onpointermove(e: PointerEvent) {
		if (!dragging) return;
		const delta = e.clientY - startY;
		position = Math.max(min, startPos + delta);
	}

	function onpointerup() {
		dragging = false;
	}
</script>

<div
	class="resizer"
	class:active={dragging}
	role="separator"
	aria-orientation="horizontal"
	tabindex="0"
	{onpointerdown}
	{onpointermove}
	{onpointerup}
></div>

<style>
	.resizer {
		height: 6px;
		cursor: row-resize;
		background: var(--color-border-subtle);
		flex-shrink: 0;
		transition: background 0.15s;
	}

	.resizer:hover,
	.resizer.active {
		background: var(--color-accent);
	}
</style>
