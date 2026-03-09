<script lang="ts">
	let {
		min = 100,
		max = Infinity,
		position = $bindable(300),
		orientation = 'horizontal' as 'horizontal' | 'vertical',
		reverse = false,
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
		const rawDelta = orientation === 'vertical'
			? -(e.clientX - startX)
			: (e.clientY - startY);
		const delta = reverse ? -rawDelta : rawDelta;
		position = Math.max(min, Math.min(max, startPos + delta));
	}

	function onpointerup() {
		dragging = false;
	}

	function onkeydown(event: KeyboardEvent) {
		const decreaseKey = orientation === 'vertical' ? 'ArrowLeft' : 'ArrowUp';
		const increaseKey = orientation === 'vertical' ? 'ArrowRight' : 'ArrowDown';
		const step = 24;

		if (event.key !== decreaseKey && event.key !== increaseKey) {
			return;
		}

		event.preventDefault();
		const rawDelta = event.key === increaseKey ? step : -step;
		const delta = reverse ? -rawDelta : rawDelta;
		position = Math.max(min, Math.min(max, position + delta));
	}
</script>

<button
	type="button"
	class="resizer"
	class:active={dragging}
	class:vertical={orientation === 'vertical'}
	aria-label="Resize panels"
	{onpointerdown}
	{onpointermove}
	{onpointerup}
	{onkeydown}
></button>

<style>
	.resizer {
		width: 100%;
		height: 6px;
		cursor: row-resize;
		padding: 0;
		border: none;
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
