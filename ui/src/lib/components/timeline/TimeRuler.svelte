<script lang="ts">
	import { TIMELINE, formatTime } from '$lib/types.js';
	import { timeToX, xToTime, timelineState } from '$lib/stores/timeline.svelte.js';

	let { durationMs, width, offsetX }: {
		durationMs: number;
		width: number;
		offsetX: number;
	} = $props();

	function handleClick(e: MouseEvent) {
		const target = e.currentTarget as HTMLElement;
		const rect = target.getBoundingClientRect();
		const localX = e.clientX - rect.left;
		const timeMs = xToTime(localX + offsetX);
		timelineState.playheadMs = Math.max(0, Math.min(timeMs, TIMELINE.DURATION_MS));
	}

	/** Generate tick marks at sensible intervals based on zoom level. */
	function ticks(): { ms: number; label: string; major: boolean }[] {
		const result: { ms: number; label: string; major: boolean }[] = [];
		// Tick every 30 seconds, major every 2.5 minutes.
		const minor = 30_000;
		const major = 150_000;
		for (let ms = 0; ms <= durationMs; ms += minor) {
			result.push({
				ms,
				label: formatTime(ms),
				major: ms % major === 0,
			});
		}
		return result;
	}
</script>

<!-- svelte-ignore a11y_click_events_have_key_events -->
<!-- svelte-ignore a11y_no_static_element_interactions -->
<div class="time-ruler" style="height: {TIMELINE.TIME_RULER_HEIGHT_PX}px" onclick={handleClick}>
	<div class="ruler-track" style="width: {width}px; transform: translateX(-{offsetX}px)">
		{#each ticks() as tick}
			<div
				class="tick"
				class:major={tick.major}
				style="left: {timeToX(tick.ms)}px"
			>
				<div class="tick-line"></div>
				{#if tick.major}
					<span class="tick-label">{tick.label}</span>
				{/if}
			</div>
		{/each}
	</div>
</div>

<style>
	.time-ruler {
		flex-shrink: 0;
		border-bottom: 1px solid var(--color-border-default);
		overflow: hidden;
		position: relative;
		background: var(--color-bg-secondary);
		cursor: pointer;
	}

	.ruler-track {
		position: relative;
		height: 100%;
	}

	.tick {
		position: absolute;
		top: 0;
		height: 100%;
		display: flex;
		flex-direction: column;
		align-items: center;
	}

	.tick-line {
		width: 1px;
		height: 6px;
		background: var(--color-text-muted);
	}

	.tick.major .tick-line {
		height: 10px;
		background: var(--color-text-secondary);
	}

	.tick-label {
		font-size: 0.65rem;
		color: var(--color-text-muted);
		margin-top: 2px;
		white-space: nowrap;
	}
</style>
