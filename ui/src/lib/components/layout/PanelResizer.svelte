<script lang="ts">
	let {
		min = 100,
		max = Infinity,
		position = $bindable(300),
		orientation = 'horizontal' as 'horizontal' | 'vertical',
	} = $props();

	let dragging = $state(false);
	let startX = 0;
	let startY = 0;
	let startPos = 0;

	function onpointerdown(e: PointerEvent) {
		dragging = true;
		startX = e.clientX;
		startY = e.clientY;
		startPos = position;
		(e.target as HTMLElement).setPointerCapture(e.pointerId);
	}

	function onpointermove(e: PointerEvent) {
		if (!dragging) return;
		const delta = orientation === 'vertical'
			? -(e.clientX - startX)
			: (e.clientY - startY);
		position = Math.max(min, Math.min(max, startPos + delta));
	}

	function onpointerup() {
		dragging = false;
	}
</script>

<div
	class="resizer"
	class:active={dragging}
	class:vertical={orientation === 'vertical'}
	role="separator"
	aria-orientation={orientation}
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

	.resizer.vertical {
		width: 6px;
		height: 100%;
		cursor: col-resize;
	}

	.resizer:hover,
	.resizer.active {
		background: var(--color-accent);
	}
</style>
