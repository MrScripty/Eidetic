<script lang="ts">
	import { timelineState, zoomToFit } from '$lib/stores/timeline.svelte.js';
	import { formatTime } from '$lib/types.js';
</script>

<div class="tl-toolbar">
	<!-- Tool modes -->
	<button
		class="tl-btn"
		class:active={timelineState.activeTool === 'select'}
		title="Selection Tool (A)"
		onclick={() => timelineState.activeTool = 'select'}
	>
		<svg width="14" height="14" viewBox="0 0 14 14" fill="currentColor">
			<path d="M2 1l8 5.5-3.5.5 2 4.5-1.5.7-2-4.5L2 10z"/>
		</svg>
	</button>
	<button
		class="tl-btn"
		class:active={timelineState.activeTool === 'blade'}
		title="Blade Tool (B)"
		onclick={() => timelineState.activeTool = 'blade'}
	>
		<svg width="14" height="14" viewBox="0 0 14 14" fill="currentColor">
			<path d="M11 1L5.5 8.5 4 7l-3 5h4l2-3 1.5 1.5L13 3z"/>
		</svg>
	</button>

	<div class="tl-sep"></div>

	<!-- Snapping -->
	<button
		class="tl-btn"
		class:active={timelineState.snapping}
		title="Snapping (N)"
		onclick={() => timelineState.snapping = !timelineState.snapping}
	>
		<svg width="14" height="14" viewBox="0 0 14 14" fill="currentColor">
			<path d="M7 1C4.5 1 2.5 3 2.5 5.5c0 1.5.7 2.8 1.8 3.7L7 13l2.7-3.8c1.1-.9 1.8-2.2 1.8-3.7C11.5 3 9.5 1 7 1zm0 6.5a2 2 0 110-4 2 2 0 010 4z"/>
		</svg>
	</button>

	<div class="tl-sep"></div>

	<!-- Zoom controls -->
	<button
		class="tl-btn"
		title="Zoom Out (Ctrl+-)"
		onclick={() => timelineState.zoom = Math.max(0.005, timelineState.zoom / 1.25)}
	>&#8722;</button>

	<button
		class="tl-btn"
		title="Zoom to Fit (Ctrl+0)"
		onclick={zoomToFit}
	>
		<svg width="14" height="14" viewBox="0 0 14 14" fill="none" stroke="currentColor" stroke-width="1.5">
			<rect x="2" y="2" width="10" height="10" rx="1"/>
			<path d="M5 4H4v2M9 4h1v2M5 10H4V8M9 10h1V8"/>
		</svg>
	</button>

	<button
		class="tl-btn"
		title="Zoom In (Ctrl+=)"
		onclick={() => timelineState.zoom = Math.min(10, timelineState.zoom * 1.25)}
	>+</button>

	<div class="tl-sep"></div>

	<!-- Playhead time display -->
	<span class="tl-time" title="Playhead position">{formatTime(timelineState.playheadMs)}</span>
</div>

<style>
	.tl-toolbar {
		display: flex;
		align-items: center;
		gap: 2px;
		padding: 2px 8px;
		background: var(--color-bg-secondary);
		border-bottom: 1px solid var(--color-border-subtle);
		flex-shrink: 0;
	}

	.tl-btn {
		background: none;
		border: 1px solid transparent;
		color: var(--color-text-secondary);
		padding: 3px 6px;
		border-radius: 4px;
		cursor: pointer;
		font-size: 0.85rem;
		line-height: 1;
		display: flex;
		align-items: center;
		justify-content: center;
		min-width: 24px;
		height: 24px;
	}

	.tl-btn:hover {
		background: var(--color-bg-hover);
		border-color: var(--color-border-subtle);
	}

	.tl-btn.active {
		background: var(--color-bg-hover);
		border-color: var(--color-accent);
		color: var(--color-accent);
	}

	.tl-sep {
		width: 1px;
		height: 16px;
		background: var(--color-border-subtle);
		margin: 0 4px;
	}

	.tl-time {
		font-size: 0.75rem;
		font-family: 'JetBrains Mono', 'Fira Code', monospace;
		color: var(--color-text-secondary);
		padding: 0 4px;
		min-width: 48px;
		text-align: center;
	}
</style>
