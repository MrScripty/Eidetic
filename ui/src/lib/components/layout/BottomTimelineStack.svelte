<script lang="ts">
  import Timeline from '../timeline/Timeline.svelte';
  import { mainTimelinePanelHeightPx } from '$lib/timelineTypes.js';

  let { timelineHeight = mainTimelinePanelHeightPx() } = $props();
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
</div>

<style>
  .timeline-stack {
    display: flex;
    flex-direction: column;
    flex: 0 0 auto;
    width: 100%;
    min-width: 0;
    max-width: 100%;
    background: var(--color-bg-primary);
    overflow: hidden;
  }

  .timeline-panel {
    box-sizing: border-box;
    width: 100%;
    min-width: 0;
    max-width: 100%;
    overflow: hidden;
    background: var(--color-bg-primary);
  }
</style>
