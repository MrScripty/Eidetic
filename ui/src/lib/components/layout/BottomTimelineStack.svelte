<script lang="ts">
  import Timeline from '../timeline/Timeline.svelte';
  import { mainTimelinePanelHeightPx } from '$lib/types.js';

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
    background: var(--color-bg-primary);
  }

  .timeline-panel {
    box-sizing: border-box;
    overflow: hidden;
    background: var(--color-bg-primary);
  }
</style>
