<script lang="ts">
  import { aiStatusState } from '$lib/stores/aiStatus.svelte.js';

  let {
    rightOffsetPx,
  }: {
    rightOffsetPx: number;
  } = $props();
</script>

<div
  class="ai-indicator"
  style="right: {rightOffsetPx}px"
  title={aiStatusState.status?.connected
    ? `AI: ${aiStatusState.status.model ?? 'connected'}`
    : 'AI: disconnected'}
>
  <span
    class="ai-dot"
    class:connected={aiStatusState.status?.connected}
    class:disconnected={aiStatusState.status && !aiStatusState.status.connected}
  ></span>
</div>

<style>
  .ai-indicator {
    position: fixed;
    bottom: 8px;
    z-index: 10;
    display: flex;
    align-items: center;
    padding: 4px 8px;
    background: var(--color-bg-surface);
    border: 1px solid var(--color-border-subtle);
    border-radius: 10px;
    cursor: default;
  }

  .ai-dot {
    width: 8px;
    height: 8px;
    border-radius: 50%;
    background: var(--color-text-muted);
  }

  .ai-dot.connected {
    background: var(--color-success);
  }

  .ai-dot.disconnected {
    background: var(--color-danger);
  }
</style>
