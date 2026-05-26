<script lang="ts">
  import { onMount } from 'svelte';
  import {
    closeTimelineRenderer,
    focusTimelineRenderer,
    getTimelineRendererStatus,
    openTimelineRenderer,
  } from '$lib/timelineRendererApi.js';
  import type { TimelineRendererStatus } from '$lib/timelineRendererTypes.js';
  import { setTimelineRendererWindowStatus } from '$lib/stores/timelineRendererWindow.svelte.js';
  import {
    timelineRendererStatusLabel,
    timelineRendererWindowControlState,
  } from './timelineRendererWindowControls.js';

  let status = $state<TimelineRendererStatus | null>(null);
  let pending = $state(false);
  let error = $state<string | null>(null);

  const running = $derived(status?.running ?? false);
  const statusLabel = $derived(timelineRendererStatusLabel(status));
  const openControl = $derived(timelineRendererWindowControlState('open', status, pending));
  const refreshControl = $derived(timelineRendererWindowControlState('refresh', status, pending));
  const focusControl = $derived(timelineRendererWindowControlState('focus', status, pending));
  const closeControl = $derived(timelineRendererWindowControlState('close', status, pending));

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
    title={openControl.label}
    aria-label={openControl.label}
    disabled={openControl.disabled}
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
    title={refreshControl.label}
    aria-label={refreshControl.label}
    disabled={refreshControl.disabled}
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
    title={focusControl.label}
    aria-label={focusControl.label}
    disabled={focusControl.disabled}
    onclick={() => run(focusTimelineRenderer)}
  >
    <svg width="14" height="14" viewBox="0 0 14 14" fill="none" stroke="currentColor">
      <path d="M7 2v10M2 7h10" stroke-width="1.4" stroke-linecap="round" />
      <circle cx="7" cy="7" r="3" stroke-width="1.3" />
    </svg>
  </button>
  <button
    type="button"
    class="tl-btn"
    title={closeControl.label}
    aria-label={closeControl.label}
    disabled={closeControl.disabled}
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
    flex: 0 0 auto;
    gap: 4px;
    min-width: max-content;
  }

  .timeline-renderer-status,
  .timeline-renderer-error {
    max-width: clamp(60px, 14vw, 150px);
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
    flex: 0 0 auto;
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
