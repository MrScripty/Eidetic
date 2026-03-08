<script lang="ts">
	import Timeline from '../timeline/Timeline.svelte';
	import CharacterTimeline from '../timeline/CharacterTimeline.svelte';
	import { characterTimelineState } from '$lib/stores/characterTimeline.svelte.js';
	import { PANEL, mainTimelinePanelHeightPx } from '$lib/types.js';

	let {
		timelineHeight = mainTimelinePanelHeightPx(),
	} = $props();
</script>

<div class="timeline-stack">
	<div
		class="timeline-panel"
		style="
			height: {timelineHeight}px;
			min-height: {timelineHeight}px;
			max-height: {timelineHeight}px;
			flex: 0 0 {timelineHeight}px;
		"
	>
		<Timeline />
	</div>

	{#if characterTimelineState.visible}
		<div
			class="char-timeline-panel"
			style="
				height: {PANEL.CHARACTER_TIMELINE_HEIGHT_PX}px;
				min-height: {PANEL.CHARACTER_TIMELINE_HEIGHT_PX}px;
				max-height: {PANEL.CHARACTER_TIMELINE_HEIGHT_PX}px;
				flex: 0 0 {PANEL.CHARACTER_TIMELINE_HEIGHT_PX}px;
			"
		>
			<CharacterTimeline />
		</div>
	{/if}
</div>

<style>
	.timeline-stack {
		display: flex;
		flex-direction: column;
		flex: 0 0 auto;
		background: var(--color-bg-primary);
	}

	.timeline-panel,
	.char-timeline-panel {
		box-sizing: border-box;
		overflow: hidden;
		background: var(--color-bg-primary);
	}
</style>
