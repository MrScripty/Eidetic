<script lang="ts">
  import type { BibleGraphEdge, BibleGraphEdgeKind } from '$lib/types.js';

  let {
    title,
    edges,
    direction,
  }: {
    title: string;
    edges: BibleGraphEdge[];
    direction: 'incoming' | 'outgoing';
  } = $props();

  function edgeKindLabel(kind: BibleGraphEdgeKind): string {
    if (typeof kind === 'string') return kind.replaceAll('_', ' ');
    return kind.custom;
  }

  function endpointLabel(edge: BibleGraphEdge): string {
    return direction === 'incoming' ? edge.from_node_id : edge.to_node_id;
  }
</script>

<section class="edge-section">
  <h3>{title}</h3>
  {#if edges.length > 0}
    <ul class="edge-list">
      {#each edges as edge (edge.id)}
        <li>
          <span class="edge-label">{edge.label}</span>
          <span class="edge-kind">{edgeKindLabel(edge.edge_kind)}</span>
          <span class="edge-target">{endpointLabel(edge)}</span>
        </li>
      {/each}
    </ul>
  {:else}
    <p class="muted">No edges</p>
  {/if}
</section>

<style>
  .edge-section {
    margin-top: 16px;
    padding-top: 12px;
    border-top: 1px solid var(--color-border-subtle);
  }

  h3,
  p {
    margin: 0;
  }

  h3 {
    color: var(--color-text-primary);
    font-size: 0.85rem;
    font-weight: 600;
  }

  .edge-list {
    display: grid;
    gap: 8px;
    list-style: none;
    margin: 12px 0 0;
    padding: 0;
  }

  .edge-list li {
    display: grid;
    gap: 2px;
  }

  .edge-label {
    color: var(--color-text-primary);
    font-size: 0.8rem;
  }

  .edge-kind,
  .edge-target,
  .muted {
    color: var(--color-text-muted);
    font-size: 0.75rem;
    overflow-wrap: anywhere;
  }

  .edge-kind {
    text-transform: uppercase;
  }
</style>
