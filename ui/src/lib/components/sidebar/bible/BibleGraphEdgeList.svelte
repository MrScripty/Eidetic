<script lang="ts">
  import type {
    BibleGraphEdge,
    BibleGraphEdgeKind,
    BibleGraphNodeId,
  } from '$lib/bibleGraphTypes.js';
  import {
    deleteBibleGraphEdgeProjection,
    refreshBibleGraphNodeProjection,
    setBibleGraphEdgeLabelProjection,
  } from '$lib/stores/bibleGraphNodeProjection.svelte.js';

  import { createBibleGraphEdgeLabelEditor } from './bibleGraphEdgeLabelEditor.svelte.js';

  let {
    title,
    edges,
    direction,
    ownerNodeId,
    revisions = {},
  }: {
    title: string;
    edges: BibleGraphEdge[];
    direction: 'incoming' | 'outgoing';
    ownerNodeId: BibleGraphNodeId;
    revisions?: Record<string, string>;
  } = $props();

  const editor = createBibleGraphEdgeLabelEditor({
    edges: () => edges,
    revisions: () => revisions,
    save: setBibleGraphEdgeLabelProjection,
    remove: deleteBibleGraphEdgeProjection,
    refresh: async (edge) => {
      if (edge.from_node_id !== ownerNodeId) {
        await refreshBibleGraphNodeProjection({ node_id: ownerNodeId });
      }
    },
    confirm: (edge) => window.confirm(`Delete edge "${edge.label}"?`),
  });
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
  {#if editor.state.error}
    <p class="error" role="alert">{editor.state.error}</p>
  {/if}
  {#if edges.length > 0}
    <ul class="edge-list">
      {#each edges as edge (edge.id)}
        <li>
          <div class="edge-main">
            <span class="edge-label">{edge.label}</span>
            <button
              type="button"
              aria-label={`Edit relationship label ${edge.label}`}
              disabled={editor.busy()}
              onclick={() => editor.edit(edge)}>Edit</button
            >
            <button
              type="button"
              class="edge-delete"
              aria-label={`Delete edge ${edge.label}`}
              title="Delete edge"
              disabled={editor.busy()}
              onclick={() => void editor.remove(edge)}
            >
              {editor.state.deleting === edge.id ? '...' : '×'}
            </button>
          </div>
          <span class="edge-kind">{edgeKindLabel(edge.edge_kind)}</span>
          <span class="edge-target">{endpointLabel(edge)}</span>
        </li>
      {/each}
    </ul>
  {:else}
    <p class="muted">No edges</p>
  {/if}
  {#if editor.state.edge}
    <label
      >Relationship label
      <input
        aria-label={`Relationship label ${editor.state.edge.id}`}
        bind:value={editor.state.label}
        disabled={editor.busy()}
      />
    </label>
    <div>
      <button
        type="button"
        disabled={editor.busy() || !editor.state.label.trim()}
        onclick={() => void editor.save()}>Save relationship label</button
      >
      <button type="button" disabled={editor.busy()} onclick={() => editor.cancel()}
        >Cancel label edit</button
      >
    </div>
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

  .edge-main {
    display: grid;
    grid-template-columns: minmax(0, 1fr) auto 24px;
    align-items: center;
    gap: 8px;
  }

  .edge-label {
    color: var(--color-text-primary);
    font-size: 0.8rem;
    overflow-wrap: anywhere;
  }

  .edge-delete {
    display: grid;
    width: 24px;
    height: 24px;
    place-items: center;
    border: 1px solid var(--color-border-subtle);
    border-radius: 4px;
    background: transparent;
    color: var(--color-text-muted);
    cursor: pointer;
    font-size: 1rem;
    line-height: 1;
  }

  .edge-delete:hover:not(:disabled) {
    border-color: var(--color-danger, #b74c4c);
    color: var(--color-danger, #b74c4c);
  }

  .edge-delete:disabled {
    cursor: wait;
    opacity: 0.6;
  }

  .edge-kind,
  .edge-target,
  .muted,
  .error {
    color: var(--color-text-muted);
    font-size: 0.75rem;
    overflow-wrap: anywhere;
  }

  .edge-kind {
    text-transform: uppercase;
  }

  .error {
    margin-top: 8px;
    color: var(--color-danger, #b74c4c);
  }
</style>
