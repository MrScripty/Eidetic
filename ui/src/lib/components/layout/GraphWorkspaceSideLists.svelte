<script lang="ts">
  import type { BibleRenderGraphInfluence } from '$lib/bibleGraphTypes.js';
  import type { BibleGraphSelection } from '$lib/stores/bible.svelte.js';
  import type {
    GraphWorkspaceEdgeItem,
    GraphWorkspaceNeighborhoodItem,
  } from './graphWorkspaceItems.js';

  let {
    selection,
    influences,
    edgeItems,
    neighborhoodItems,
    onselectinfluence,
    onselectedge,
    onselectneighborhood,
  }: {
    selection: BibleGraphSelection;
    influences: BibleRenderGraphInfluence[];
    edgeItems: GraphWorkspaceEdgeItem[];
    neighborhoodItems: GraphWorkspaceNeighborhoodItem[];
    onselectinfluence: (id: string) => void;
    onselectedge: (id: string) => void;
    onselectneighborhood: (id: string) => void;
  } = $props();
</script>

<div class="graph-side-lists">
  {#if influences.length > 0}
    <section class="graph-list" aria-label="Active graph influences">
      <h2>Influences</h2>
      {#each influences as influence (influence.influence_id)}
        <button
          type="button"
          class:selected={selection.kind === 'influence' &&
            selection.influenceId === influence.influence_id}
          aria-pressed={selection.kind === 'influence' &&
            selection.influenceId === influence.influence_id}
          onclick={() => onselectinfluence(influence.influence_id)}
        >
          <span class="item-kind">{influence.influence_kind}</span>
          <span class="item-reason">{influence.reason}</span>
          <span class="item-metric">{Math.round(influence.confidence * 100)}%</span>
        </button>
      {/each}
    </section>
  {/if}

  {#if edgeItems.length > 0}
    <section class="graph-list" aria-label="Bible graph edges">
      <h2>Edges</h2>
      {#each edgeItems as edge (edge.edgeId)}
        <button
          type="button"
          class:selected={selection.kind === 'edge' && selection.edgeId === edge.edgeId}
          aria-pressed={selection.kind === 'edge' && selection.edgeId === edge.edgeId}
          onclick={() => onselectedge(edge.edgeId)}
        >
          <span class="item-kind">{edge.label}</span>
          <span class="item-reason"
            >{edge.fromLabel} {edge.directed ? '->' : '-'} {edge.toLabel}</span
          >
        </button>
      {/each}
    </section>
  {/if}

  {#if neighborhoodItems.length > 0}
    <section class="graph-list" aria-label="Bible graph neighborhoods">
      <h2>Neighborhoods</h2>
      {#each neighborhoodItems as neighborhood (neighborhood.nodeId)}
        <button
          type="button"
          class:selected={selection.kind === 'neighborhood' &&
            selection.nodeId === neighborhood.nodeId}
          aria-pressed={selection.kind === 'neighborhood' &&
            selection.nodeId === neighborhood.nodeId}
          onclick={() => onselectneighborhood(neighborhood.nodeId)}
        >
          <span class="item-kind">{neighborhood.label}</span>
          <span class="item-reason"
            >{neighborhood.connectedNodeCount} connected, {neighborhood.edgeCount} edges</span
          >
        </button>
      {/each}
    </section>
  {/if}
</div>

<style>
  .graph-side-lists {
    min-width: 0;
    overflow-y: auto;
    border-left: 1px solid var(--color-border-subtle);
    background: var(--color-bg-secondary);
  }

  .graph-list {
    border-bottom: 1px solid var(--color-border-default);
  }

  h2 {
    margin: 0;
    padding: 8px 10px 4px;
    color: var(--color-text-muted);
    font-size: 0.68rem;
    font-weight: 600;
    text-transform: uppercase;
  }

  button {
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

  button:hover,
  button:focus-visible {
    background: var(--color-bg-hover);
    color: var(--color-text-primary);
  }

  button:focus-visible {
    outline: 2px solid var(--color-accent);
    outline-offset: -2px;
  }

  button.selected {
    background: var(--color-bg-surface);
    color: var(--color-text-primary);
    box-shadow: inset 2px 0 0 var(--color-warning);
  }

  .item-kind,
  .item-metric {
    color: var(--color-warning);
    font-size: 0.7rem;
    text-transform: uppercase;
  }

  .item-reason {
    grid-column: 1 / -1;
    min-width: 0;
    overflow: hidden;
    color: var(--color-text-secondary);
    font-size: 0.76rem;
    line-height: 1.3;
    text-overflow: ellipsis;
    white-space: nowrap;
  }
</style>
