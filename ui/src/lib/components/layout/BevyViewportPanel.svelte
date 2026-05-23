<script lang="ts">
  import { onMount } from 'svelte';
  import {
    mountEmbeddedViewport,
    setEmbeddedViewportFocus,
    unmountEmbeddedViewport,
    updateEmbeddedViewportBounds,
  } from '$lib/embeddedViewportApi.js';
  import {
    embeddedViewportBoundsChanged,
    embeddedViewportBoundsFromRect,
  } from '$lib/embeddedViewportBounds.js';
  import type { BibleRenderGraphProjectionRequest } from '$lib/bibleGraphTypes.js';
  import type { EmbeddedViewportBounds, EmbeddedViewportKind } from '$lib/embeddedViewportTypes.js';

  let {
    viewportId,
    kind,
    ariaLabel,
    initialGraphProjectionRequest,
  }: {
    viewportId: string;
    kind: EmbeddedViewportKind;
    ariaLabel: string;
    initialGraphProjectionRequest?: BibleRenderGraphProjectionRequest;
  } = $props();

  let panel: HTMLButtonElement;
  let mounted = false;
  let lastBounds: EmbeddedViewportBounds | null = null;

  function readBounds(): EmbeddedViewportBounds | null {
    if (!panel) {
      return null;
    }
    return embeddedViewportBoundsFromRect(
      panel.getBoundingClientRect(),
      window.devicePixelRatio || 1,
    );
  }

  async function syncBounds(): Promise<void> {
    const bounds = readBounds();
    if (!bounds || !embeddedViewportBoundsChanged(lastBounds, bounds)) {
      return;
    }

    if (!mounted) {
      await mountEmbeddedViewport({
        viewport_id: viewportId,
        kind,
        bounds,
        ...(initialGraphProjectionRequest
          ? { graph_projection_request: initialGraphProjectionRequest }
          : {}),
      });
      mounted = true;
    } else {
      await updateEmbeddedViewportBounds({
        viewport_id: viewportId,
        bounds,
      });
    }
    lastBounds = bounds;
  }

  function setFocus(focused: boolean): void {
    if (mounted) {
      void setEmbeddedViewportFocus({
        viewport_id: viewportId,
        focused,
      }).catch(() => {});
    }
  }

  onMount(() => {
    const observer = new ResizeObserver(() => {
      void syncBounds().catch(() => {});
    });
    observer.observe(panel);
    window.addEventListener('resize', syncBounds);
    void syncBounds().catch(() => {});

    return () => {
      observer.disconnect();
      window.removeEventListener('resize', syncBounds);
      if (mounted) {
        void unmountEmbeddedViewport(viewportId).catch(() => {});
      }
    };
  });
</script>

<button
  type="button"
  bind:this={panel}
  class="bevy-viewport-panel"
  aria-label={ariaLabel}
  onfocus={() => setFocus(true)}
  onblur={() => setFocus(false)}
></button>

<style>
  .bevy-viewport-panel {
    min-width: 0;
    min-height: 0;
    width: 100%;
    height: 100%;
    padding: 0;
    border: 0;
    outline: none;
    background: transparent;
    appearance: none;
    cursor: default;
  }

  .bevy-viewport-panel:focus-visible {
    box-shadow: inset 0 0 0 1px var(--color-accent);
  }
</style>
