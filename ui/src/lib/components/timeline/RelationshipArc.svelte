<script lang="ts">
	let { fromX, fromY, toX, toY, color }: {
		fromX: number;
		fromY: number;
		toX: number;
		toY: number;
		color: string;
	} = $props();

	// Control point is above the midpoint for a nice arc.
	let cpX = $derived((fromX + toX) / 2);
	let cpY = $derived(Math.min(fromY, toY) - 30);
</script>

<path
	d="M {fromX} {fromY} Q {cpX} {cpY} {toX} {toY}"
	stroke={color}
	stroke-width="2"
	fill="none"
	opacity="0.7"
/>
