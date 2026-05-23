<script lang="ts">
  import { onMount } from 'svelte';
  import type { BibleRenderGraphProjectionRequest } from '$lib/bibleGraphTypes.js';
  import {
    closeGraphRenderer,
    focusGraphRenderer,
    getGraphRendererStatus,
    openGraphRenderer,
  } from '$lib/graphRendererApi.js';
  import type { GraphRendererStatus } from '$lib/graphRendererTypes.js';

  let {
    graphProjectionRequest,
  }: {
    graphProjectionRequest: BibleRenderGraphProjectionRequest;
  } = $props();

  let status = $state<GraphRendererStatus | null>(null);
  let pending = $state(false);
  let error = $state<string | null>(null);

  const isOpen = $derived(status?.renderer_window_open ?? false);
  const isVisible = $derived(status?.renderer_window_visible ?? false);
  const statusText = $derived(status?.renderer_window_message ?? 'Graph renderer window is closed');

  async function run(action: () => Promise<GraphRendererStatus>): Promise<void> {
    pending = true;
    error = null;
    try {
      status = await action();
    } catch (caught) {
      error = caught instanceof Error ? caught.message : 'Graph renderer command failed';
    } finally {
      pending = false;
    }
  }

  function openWindow(): void {
    void run(() =>
      openGraphRenderer({
        graph_projection_request: graphProjectionRequest,
      }),
    );
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
</script>

<section class="renderer-window-controls" aria-label="Bible graph renderer window">
  <div class="renderer-window-status" aria-live="polite">
    <span class:active={isVisible}>
      {#if isVisible}
        Renderer visible
      {:else if isOpen}
        Renderer preparing
      {:else}
        Renderer closed
      {/if}
    </span>
    <span>{statusText}</span>
    {#if error}
      <span class="error">{error}</span>
    {/if}
  </div>

  <div class="renderer-window-actions">
    <button type="button" onclick={openWindow} disabled={pending}> Open Graph </button>
    <button type="button" onclick={focusWindow} disabled={pending || !isOpen}> Focus </button>
    <button type="button" onclick={closeWindow} disabled={pending || !isOpen}> Close </button>
  </div>
</section>

<style>
  .renderer-window-controls {
    display: flex;
    align-items: center;
    justify-content: space-between;
    gap: 10px;
    min-height: 34px;
    padding: 4px 8px;
    border-bottom: 1px solid var(--color-border-subtle);
    background: var(--color-bg-secondary);
  }

  .renderer-window-status {
    display: flex;
    align-items: center;
    gap: 8px;
    min-width: 0;
    color: var(--color-text-muted);
    font-size: 0.76rem;
  }

  .renderer-window-status span {
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
    flex-shrink: 0;
  }

  button {
    border: 1px solid var(--color-border-subtle);
    border-radius: 4px;
    padding: 3px 7px;
    background: var(--color-bg-surface);
    color: var(--color-text-secondary);
    font-size: 0.76rem;
    cursor: pointer;
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
