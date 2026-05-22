<script lang="ts">
  import {
    bibleState,
    selectBibleGraphInfluence,
    selectBibleGraphNode,
    selectedBibleGraphNodeId,
  } from '$lib/stores/bible.svelte.js';
  import { getCachedBibleRenderGraphProjection } from '$lib/stores/bibleRenderGraphProjection.svelte.js';
  import BibleRenderGraphOutline from '../sidebar/bible/BibleRenderGraphOutline.svelte';

  const renderGraphProjection = $derived(getCachedBibleRenderGraphProjection());
  const graph = $derived(renderGraphProjection?.payload ?? null);
  const selectedGraphNodeId = $derived(selectedBibleGraphNodeId());
  const graphSelection = $derived(bibleState.graphSelection);

  function handleSelect(id: string) {
    selectBibleGraphNode(selectedGraphNodeId === id ? null : id);
  }

  function handleSelectInfluence(id: string) {
    selectBibleGraphInfluence(
      graphSelection.kind === 'influence' && graphSelection.influenceId === id ? null : id,
    );
  }
</script>

<section class="graph-workspace" aria-label="Bible graph workspace">
  {#if graph}
    <div class="graph-summary" aria-label="Bible graph projection summary">
      <span>{graph.nodes.length} nodes</span>
      <span>{graph.edges.length} edges</span>
      <span>{graph.influences.length} influences</span>
    </div>

    <div class="graph-surface" class:has-influences={graph.influences.length > 0}>
      <BibleRenderGraphOutline
        projection={graph}
        selectedNodeId={selectedGraphNodeId}
        onselect={handleSelect}
      />

      {#if graph.influences.length > 0}
        <section class="influence-list" aria-label="Active graph influences">
          {#each graph.influences as influence (influence.influence_id)}
            <button
              type="button"
              class:selected={graphSelection.kind === 'influence' &&
                graphSelection.influenceId === influence.influence_id}
              aria-pressed={graphSelection.kind === 'influence' &&
                graphSelection.influenceId === influence.influence_id}
              onclick={() => handleSelectInfluence(influence.influence_id)}
            >
              <span class="influence-kind">{influence.influence_kind}</span>
              <span class="influence-reason">{influence.reason}</span>
              <span class="influence-confidence">{Math.round(influence.confidence * 100)}%</span>
            </button>
          {/each}
        </section>
      {/if}
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
    display: grid;
    grid-template-columns: minmax(0, 1fr);
    min-height: 0;
    flex: 1;
    overflow: hidden;
  }

  .graph-surface.has-influences {
    grid-template-columns: minmax(0, 1fr) minmax(220px, 0.34fr);
  }

  .graph-surface :global(.graph-outline) {
    max-height: none;
    height: 100%;
    border: 0;
  }

  .influence-list {
    min-width: 0;
    overflow-y: auto;
    border-left: 1px solid var(--color-border-subtle);
    background: var(--color-bg-secondary);
  }

  .influence-list button {
    display: grid;
    grid-template-columns: minmax(0, 1fr) auto;
    gap: 4px 8px;
    width: 100%;
    min-height: 42px;
    padding: 8px 10px;
    border: 0;
    border-bottom: 1px solid var(--color-border-subtle);
    background: transparent;
    color: var(--color-text-secondary);
    cursor: pointer;
    text-align: left;
  }

  .influence-list button:hover,
  .influence-list button:focus-visible {
    background: var(--color-bg-hover);
    color: var(--color-text-primary);
  }

  .influence-list button:focus-visible {
    outline: 2px solid var(--color-accent);
    outline-offset: -2px;
  }

  .influence-list button.selected {
    background: var(--color-bg-surface);
    color: var(--color-text-primary);
    box-shadow: inset 2px 0 0 var(--color-warning);
  }

  .influence-kind,
  .influence-confidence {
    color: var(--color-warning);
    font-size: 0.7rem;
    text-transform: uppercase;
  }

  .influence-reason {
    grid-column: 1 / -1;
    min-width: 0;
    overflow: hidden;
    color: var(--color-text-secondary);
    font-size: 0.76rem;
    line-height: 1.3;
    text-overflow: ellipsis;
    white-space: nowrap;
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
