<script lang="ts">
  import { selectBibleGraphNode, selectedBibleGraphNodeId } from '$lib/stores/bible.svelte.js';
  import { getCachedBibleRenderGraphProjection } from '$lib/stores/bibleRenderGraphProjection.svelte.js';
  import BibleRenderGraphOutline from '../sidebar/bible/BibleRenderGraphOutline.svelte';

  const renderGraphProjection = $derived(getCachedBibleRenderGraphProjection());
  const graph = $derived(renderGraphProjection?.payload ?? null);
  const selectedGraphNodeId = $derived(selectedBibleGraphNodeId());

  function handleSelect(id: string) {
    selectBibleGraphNode(selectedGraphNodeId === id ? null : id);
  }
</script>

<section class="graph-workspace" aria-label="Bible graph workspace">
  {#if graph}
    <div class="graph-summary" aria-label="Bible graph projection summary">
      <span>{graph.nodes.length} nodes</span>
      <span>{graph.edges.length} edges</span>
      <span>{graph.influences.length} influences</span>
    </div>

    <div class="graph-surface">
      <BibleRenderGraphOutline
        projection={graph}
        selectedNodeId={selectedGraphNodeId}
        onselect={handleSelect}
      />
    </div>
  {:else}
    <div class="empty-state">No graph projection</div>
  {/if}
</section>

<style>
  .graph-workspace {
    display: flex;
    flex-direction: column;
    min-height: 0;
    height: 100%;
    background: var(--color-bg-primary);
  }

  .graph-summary {
    display: flex;
    align-items: center;
    gap: 10px;
    min-height: 30px;
    padding: 4px 10px;
    border-bottom: 1px solid var(--color-border-subtle);
    color: var(--color-text-muted);
    font-size: 0.76rem;
    font-variant-numeric: tabular-nums;
  }

  .graph-surface {
    min-height: 0;
    flex: 1;
    overflow: hidden;
  }

  .graph-surface :global(.graph-outline) {
    max-height: none;
    height: 100%;
    border: 0;
  }

  .empty-state {
    display: flex;
    align-items: center;
    justify-content: center;
    flex: 1;
    color: var(--color-text-muted);
    font-size: 0.85rem;
  }
</style>
