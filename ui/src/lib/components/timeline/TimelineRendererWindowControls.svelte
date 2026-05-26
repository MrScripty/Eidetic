<script lang="ts">
  import { onMount } from 'svelte';
  import {
    closeTimelineRenderer,
    getTimelineRendererStatus,
    openTimelineRenderer,
  } from '$lib/timelineRendererApi.js';
  import type { TimelineRendererStatus } from '$lib/timelineRendererTypes.js';
  import { setTimelineRendererWindowStatus } from '$lib/stores/timelineRendererWindow.svelte.js';

  let status = $state<TimelineRendererStatus | null>(null);
  let pending = $state(false);
  let error = $state<string | null>(null);

  const running = $derived(status?.running ?? false);
  const statusLabel = $derived(
    status?.last_error ??
      (status
        ? status.renderer_window_ready
          ? `${status.clip_count} clips`
          : status.renderer_window_message
        : 'not checked'),
  );

  async function run(action: () => Promise<TimelineRendererStatus>): Promise<void> {
    pending = true;
    error = null;
    try {
      status = await action();
      setTimelineRendererWindowStatus(status);
    } catch (caught) {
      error = caught instanceof Error ? caught.message : 'Timeline renderer command failed';
    } finally {
      pending = false;
    }
  }

  onMount(() => {
    void run(getTimelineRendererStatus);
  });
</script>

<div class="timeline-renderer-controls" aria-label="Timeline renderer window">
  <span class="timeline-renderer-status" class:active={running}>{statusLabel}</span>
  {#if error}
    <span class="timeline-renderer-error">{error}</span>
  {/if}
  <button
    type="button"
    class="tl-btn"
    title="Open timeline renderer"
    disabled={pending}
    onclick={() => run(openTimelineRenderer)}
  >
    <svg width="14" height="14" viewBox="0 0 14 14" fill="none" stroke="currentColor">
      <rect x="2" y="3" width="10" height="8" rx="1" stroke-width="1.4" />
      <path d="M4 6h6M4 8h4" stroke-width="1.4" stroke-linecap="round" />
    </svg>
  </button>
  <button
    type="button"
    class="tl-btn"
    title="Refresh timeline renderer status"
    disabled={pending}
    onclick={() => run(getTimelineRendererStatus)}
  >
    <svg width="14" height="14" viewBox="0 0 14 14" fill="none" stroke="currentColor">
      <path d="M11 4.5A4.5 4.5 0 103.5 10" stroke-width="1.4" stroke-linecap="round" />
      <path d="M11 2v2.5H8.5" stroke-width="1.4" stroke-linecap="round" stroke-linejoin="round" />
    </svg>
  </button>
  <button
    type="button"
    class="tl-btn"
    title="Close timeline renderer"
    disabled={pending || !running}
    onclick={() => run(closeTimelineRenderer)}
  >
    <svg width="14" height="14" viewBox="0 0 14 14" fill="none" stroke="currentColor">
      <path d="M4 4l6 6M10 4l-6 6" stroke-width="1.5" stroke-linecap="round" />
    </svg>
  </button>
</div>

<style>
  .timeline-renderer-controls {
    display: flex;
    align-items: center;
    gap: 4px;
    min-width: 0;
    margin-left: auto;
  }

  .timeline-renderer-status,
  .timeline-renderer-error {
    max-width: 150px;
    overflow: hidden;
    text-overflow: ellipsis;
    white-space: nowrap;
    font-size: 0.72rem;
    color: var(--color-text-muted);
  }

  .timeline-renderer-status.active {
    color: var(--color-accent);
  }

  .timeline-renderer-error {
    color: var(--color-danger);
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

  .tl-btn:hover:not(:disabled) {
    background: var(--color-bg-hover);
    border-color: var(--color-border-subtle);
  }

  .tl-btn:focus-visible {
    outline: 2px solid var(--color-accent);
    outline-offset: 2px;
  }

  .tl-btn:disabled {
    opacity: 0.45;
    cursor: default;
  }
</style>
