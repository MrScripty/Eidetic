<script lang="ts">
  import { onMount } from 'svelte';
  import type { BibleRenderGraphProjectionRequest } from '$lib/bibleGraphTypes.js';
  import {
    closeGraphRenderer,
    focusGraphRenderer,
    getGraphRendererStatus,
    openGraphRenderer,
    updateGraphRendererProjectionRequest,
  } from '$lib/graphRendererApi.js';
  import type { GraphRendererStatus } from '$lib/graphRendererTypes.js';
  import { setGraphRendererWindowStatus } from '$lib/stores/graphRendererWindow.svelte.js';
  import { graphRendererWindowStatusDisplay } from './graphRendererWindowStatus.js';

  let {
    graphProjectionRequest,
  }: {
    graphProjectionRequest: BibleRenderGraphProjectionRequest;
  } = $props();

  let status = $state<GraphRendererStatus | null>(null);
  let pending = $state(false);
  let error = $state<string | null>(null);
  let lastSyncedRequestKey = $state<string | null>(null);

  const isOpen = $derived(status?.renderer_window_open ?? false);
  const canFocus = $derived(status?.renderer_window_focus_supported ?? false);
  const statusDisplay = $derived(graphRendererWindowStatusDisplay(status));
  const primaryDisabled = $derived(
    pending || (isOpen && !statusDisplay.nativeWindowAvailable) || (!isOpen && pending),
  );

  async function run(action: () => Promise<GraphRendererStatus>): Promise<boolean> {
    pending = true;
    error = null;
    try {
      status = await action();
      setGraphRendererWindowStatus(status);
      return true;
    } catch (caught) {
      error = caught instanceof Error ? caught.message : 'Graph renderer command failed';
      return false;
    } finally {
      pending = false;
    }
  }

  function openWindow(): void {
    const requestKey = JSON.stringify(graphProjectionRequest);
    void run(() =>
      openGraphRenderer({
        graph_projection_request: graphProjectionRequest,
      }),
    ).then((succeeded) => {
      if (succeeded) {
        lastSyncedRequestKey = requestKey;
      }
    });
  }

  function focusWindow(): void {
    void run(focusGraphRenderer);
  }

  function closeWindow(): void {
    void run(closeGraphRenderer);
  }

  onMount(() => {
    void run(getGraphRendererStatus);
  });

  $effect(() => {
    if (!status?.renderer_window_open) {
      lastSyncedRequestKey = null;
      return;
    }

    const requestKey = JSON.stringify(graphProjectionRequest);
    if (requestKey === lastSyncedRequestKey) {
      return;
    }
    void run(() => updateGraphRendererProjectionRequest(graphProjectionRequest)).then(
      (succeeded) => {
        if (succeeded) {
          lastSyncedRequestKey = requestKey;
        }
      },
    );
  });
</script>

<section class="renderer-window-controls" aria-label="Bible graph renderer window">
  <div class="renderer-window-main">
    <div class="renderer-window-status" aria-live="polite">
      <span class:active={statusDisplay.active}>{statusDisplay.label}</span>
      <span>{statusDisplay.message}</span>
      {#if error}
        <span class="error">{error}</span>
      {/if}

      <span class="renderer-window-actions">
        <button
          type="button"
          class="primary"
          onclick={statusDisplay.nativeWindowAvailable && isOpen ? focusWindow : openWindow}
          disabled={primaryDisabled}
        >
          {statusDisplay.primaryActionLabel}
        </button>
        <button type="button" onclick={focusWindow} disabled={pending || !canFocus}> Focus </button>
        <button type="button" onclick={closeWindow} disabled={pending || !isOpen}> Close </button>
      </span>
    </div>
  </div>
</section>

<style>
  .renderer-window-controls {
    display: flex;
    align-items: center;
    gap: 14px;
    padding: 12px;
    border-bottom: 1px solid var(--color-border-subtle);
    background: var(--color-bg-secondary);
  }

  .renderer-window-main {
    display: flex;
    flex-direction: column;
    gap: 6px;
    min-width: 0;
    width: 100%;
  }

  .renderer-window-status {
    display: flex;
    align-items: center;
    flex-wrap: wrap;
    gap: 8px;
    min-width: 0;
    color: var(--color-text-muted);
    font-size: 0.76rem;
  }

  .renderer-window-status span {
    min-width: 0;
    max-width: 100%;
    overflow: visible;
  }

  .renderer-window-status > span:not(.renderer-window-actions) {
    overflow: hidden;
    text-overflow: ellipsis;
    white-space: nowrap;
  }

  .renderer-window-status .active {
    color: var(--color-accent);
  }

  .renderer-window-status .error {
    color: var(--color-danger);
  }

  .renderer-window-actions {
    display: flex;
    align-items: center;
    gap: 4px;
    justify-content: flex-start;
    min-width: 0;
    flex-wrap: wrap;
  }

  button {
    border: 1px solid var(--color-border-subtle);
    border-radius: 4px;
    padding: 6px 9px;
    background: var(--color-bg-surface);
    color: var(--color-text-secondary);
    font-size: 0.78rem;
    cursor: pointer;
  }

  button.primary {
    border-color: color-mix(in srgb, var(--color-accent) 72%, var(--color-border-subtle));
    background: color-mix(in srgb, var(--color-accent) 16%, var(--color-bg-surface));
    color: var(--color-text-primary);
    font-weight: 600;
  }

  button:hover:not(:disabled) {
    background: var(--color-bg-hover);
    color: var(--color-text-primary);
  }

  button:focus-visible {
    outline: 2px solid var(--color-accent);
    outline-offset: 2px;
  }

  button:disabled {
    opacity: 0.45;
    cursor: default;
  }
</style>
