<script lang="ts">
  import { clearBibleGraphSelection, type BibleGraphSelection } from '$lib/stores/bible.svelte.js';
  import type { BibleRenderGraphProjection } from '$lib/bibleGraphTypes.js';
  import { graphSelectionDetail } from './graphSelectionDetails.js';

  let {
    projection,
    selection,
  }: {
    projection: BibleRenderGraphProjection | null;
    selection: BibleGraphSelection;
  } = $props();

  const detail = $derived(graphSelectionDetail(projection, selection));
</script>

<section class="graph-selection-detail" aria-label="Graph selection detail">
  <header>
    <h2>Graph Detail</h2>
    <button type="button" aria-label="Close graph detail" onclick={clearBibleGraphSelection}>
      x
    </button>
  </header>

  {#if detail?.kind === 'edge'}
    <dl>
      <div>
        <dt>Edge</dt>
        <dd>{detail.edge.label}</dd>
      </div>
      <div>
        <dt>From</dt>
        <dd>{detail.fromLabel}</dd>
      </div>
      <div>
        <dt>To</dt>
        <dd>{detail.toLabel}</dd>
      </div>
      <div>
        <dt>Kind</dt>
        <dd>{detail.edge.edge_kind}</dd>
      </div>
    </dl>
  {:else if detail?.kind === 'influence'}
    <dl>
      <div>
        <dt>Influence</dt>
        <dd>{detail.influence.influence_kind}</dd>
      </div>
      <div>
        <dt>Layer</dt>
        <dd>{detail.influence.source_layer}</dd>
      </div>
      <div>
        <dt>Confidence</dt>
        <dd>{Math.round(detail.influence.confidence * 100)}%</dd>
      </div>
      {#if detail.nodeLabel}
        <div>
          <dt>Node</dt>
          <dd>{detail.nodeLabel}</dd>
        </div>
      {/if}
      {#if detail.edgeLabel}
        <div>
          <dt>Edge</dt>
          <dd>{detail.edgeLabel}</dd>
        </div>
      {/if}
      <div>
        <dt>Reason</dt>
        <dd>{detail.influence.reason}</dd>
      </div>
      <div>
        <dt>Provenance</dt>
        <dd>{detail.influence.provenance}</dd>
      </div>
    </dl>
  {:else if detail?.kind === 'context_layer'}
    <dl>
      <div>
        <dt>Timeline Node</dt>
        <dd>{detail.timelineNodeId}</dd>
      </div>
      <div>
        <dt>Influences</dt>
        <dd>{detail.influenceCount}</dd>
      </div>
    </dl>
  {:else if detail?.kind === 'neighborhood'}
    <dl>
      <div>
        <dt>Node</dt>
        <dd>{detail.node.label}</dd>
      </div>
      <div>
        <dt>Connections</dt>
        <dd>{detail.connectedLabels.length}</dd>
      </div>
      {#if detail.connectedLabels.length > 0}
        <div>
          <dt>Connected</dt>
          <dd>{detail.connectedLabels.join(', ')}</dd>
        </div>
      {/if}
      <div>
        <dt>Edges</dt>
        <dd>{detail.neighborhood?.edge_ids.length ?? 0}</dd>
      </div>
    </dl>
  {:else}
    <p>No graph selection detail available in this projection.</p>
  {/if}
</section>

<style>
  .graph-selection-detail {
    display: flex;
    flex-direction: column;
    height: 100%;
    min-height: 0;
    overflow: auto;
    color: var(--color-text-primary);
  }

  header {
    display: flex;
    align-items: center;
    justify-content: space-between;
    gap: 8px;
    padding: 10px 12px;
    border-bottom: 1px solid var(--color-border-default);
  }

  h2 {
    margin: 0;
    font-size: 0.85rem;
    font-weight: 600;
  }

  button {
    width: 26px;
    height: 26px;
    border: 1px solid var(--color-border-default);
    border-radius: 4px;
    background: var(--color-bg-surface);
    color: var(--color-text-secondary);
    cursor: pointer;
  }

  button:hover,
  button:focus-visible {
    color: var(--color-text-primary);
    border-color: var(--color-accent);
  }

  dl {
    margin: 0;
    padding: 10px 12px;
  }

  div {
    padding: 8px 0;
    border-bottom: 1px solid var(--color-border-subtle);
  }

  dt {
    margin-bottom: 4px;
    color: var(--color-text-muted);
    font-size: 0.7rem;
    text-transform: uppercase;
  }

  dd {
    margin: 0;
    color: var(--color-text-primary);
    font-size: 0.82rem;
    line-height: 1.35;
    overflow-wrap: anywhere;
  }

  p {
    margin: 0;
    padding: 12px;
    color: var(--color-text-muted);
    font-size: 0.82rem;
    line-height: 1.4;
  }
</style>
