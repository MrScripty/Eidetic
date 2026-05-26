<script lang="ts">
  import type {
    BibleGraphNode,
    BibleGraphNodeId,
    BibleRenderGraphNode,
  } from '$lib/bibleGraphTypes.js';
  import {
    deleteBibleGraphNodeProjection,
    getBibleGraphNodeProjectionError,
    getCachedBibleGraphNodeProjection,
    isBibleGraphNodeProjectionPending,
    refreshBibleGraphNodeProjection,
  } from '$lib/stores/bibleGraphNodeProjection.svelte.js';
  import BibleGraphEdgeEditor from './BibleGraphEdgeEditor.svelte';
  import { bibleGraphEdgeTargetOptions } from './bibleGraphEdgeTargetOptions.js';
  import BibleGraphEdgeList from './BibleGraphEdgeList.svelte';
  import BibleGraphPartFields from './BibleGraphPartFields.svelte';
  import BibleGraphSnapshotEditor from './BibleGraphSnapshotEditor.svelte';
  import BibleGraphSnapshotList from './BibleGraphSnapshotList.svelte';

  let {
    nodeId,
    onclose,
    ondeleted,
    edgeTargetNodes,
  }: {
    nodeId: BibleGraphNodeId;
    onclose: () => void;
    ondeleted?: () => void | Promise<void>;
    edgeTargetNodes: Array<BibleGraphNode | BibleRenderGraphNode>;
  } = $props();

  let deletingNode = $state(false);
  let deleteError = $state<string | undefined>(undefined);

  const key = $derived({ node_id: nodeId });
  const projection = $derived(getCachedBibleGraphNodeProjection(key));
  const pending = $derived(isBibleGraphNodeProjectionPending(key));
  const error = $derived(getBibleGraphNodeProjectionError(key));
  const edgeTargetOptions = $derived(bibleGraphEdgeTargetOptions(edgeTargetNodes, nodeId));
  const deleteDisabledReason = $derived(() => {
    if (!projection) return 'Node projection is not loaded';
    if (projection.payload.node.system_owned) return 'Canonical roots cannot be deleted';
    if (
      projection.payload.incoming_edges.length > 0 ||
      projection.payload.outgoing_edges.length > 0
    ) {
      return 'Delete connected edges before deleting this node';
    }
    return null;
  });

  $effect(() => {
    void refreshBibleGraphNodeProjection({ node_id: nodeId }).catch(() => {});
  });

  async function handleDeleteNode(): Promise<void> {
    if (!projection) return;
    if (deleteDisabledReason() !== null) return;
    if (!window.confirm(`Delete node "${projection.payload.node.name}"?`)) return;

    deletingNode = true;
    deleteError = undefined;
    try {
      await deleteBibleGraphNodeProjection(projection.payload.node.id);
      if (ondeleted) {
        await ondeleted();
      } else {
        onclose();
      }
    } catch (error) {
      deleteError = error instanceof Error ? error.message : 'Failed to delete bible graph node';
    } finally {
      deletingNode = false;
    }
  }
</script>

<div class="graph-node-detail">
  <div class="detail-header">
    <button class="close-btn" onclick={onclose}>&times; Close</button>
    {#if projection}
      <button
        type="button"
        class="delete-btn"
        disabled={deletingNode || deleteDisabledReason() !== null}
        title={deleteDisabledReason() ?? 'Delete node'}
        onclick={() => void handleDeleteNode()}
      >
        {deletingNode ? 'Deleting' : 'Delete'}
      </button>
      <span class="schema-label">{projection.payload.node.schema_key}</span>
    {/if}
  </div>

  {#if projection}
    <div class="detail-body">
      <h2>{projection.payload.node.name}</h2>
      {#if deleteError}
        <p class="status error">{deleteError}</p>
      {/if}
      <dl class="metadata">
        <div>
          <dt>ID</dt>
          <dd>{projection.payload.node.id}</dd>
        </div>
        {#if projection.payload.node.parent_id}
          <div>
            <dt>Parent</dt>
            <dd>{projection.payload.node.parent_id}</dd>
          </div>
        {/if}
      </dl>

      {#each projection.payload.parts as partProjection (partProjection.part.id)}
        <BibleGraphPartFields nodeId={projection.payload.node.id} {partProjection} />
      {/each}

      {#if projection.payload.parts.length === 0}
        <p class="muted">No parts</p>
      {/if}

      <BibleGraphSnapshotEditor
        nodeId={projection.payload.node.id}
        parts={projection.payload.parts}
        nextSortOrder={projection.payload.snapshots.length + 1}
      />
      <BibleGraphSnapshotList snapshots={projection.payload.snapshots} />

      <BibleGraphEdgeEditor
        sourceNodeId={projection.payload.node.id}
        nextSortOrder={projection.payload.outgoing_edges.length + 1}
        targetOptions={edgeTargetOptions}
      />
      <BibleGraphEdgeList
        title="Outgoing Edges"
        edges={projection.payload.outgoing_edges}
        direction="outgoing"
        ownerNodeId={projection.payload.node.id}
      />
      <BibleGraphEdgeList
        title="Incoming Edges"
        edges={projection.payload.incoming_edges}
        direction="incoming"
        ownerNodeId={projection.payload.node.id}
      />
    </div>
  {:else if pending}
    <p class="status">Loading</p>
  {:else if error}
    <p class="status error">{error}</p>
  {:else}
    <p class="status">No projection</p>
  {/if}
</div>

<style>
  .graph-node-detail {
    display: flex;
    flex-direction: column;
    height: 100%;
    background: var(--color-bg-panel);
  }

  .detail-header {
    display: flex;
    align-items: center;
    gap: 8px;
    padding: 8px 12px;
    border-bottom: 1px solid var(--color-border-subtle);
  }

  .close-btn {
    background: none;
    border: none;
    color: var(--color-text-secondary);
    cursor: pointer;
    font-size: 0.8rem;
  }

  .close-btn:hover {
    color: var(--color-text-primary);
  }

  .delete-btn {
    border: 1px solid var(--color-border-subtle);
    border-radius: 4px;
    background: transparent;
    color: var(--color-text-secondary);
    cursor: pointer;
    font-size: 0.75rem;
    padding: 4px 8px;
  }

  .delete-btn:hover:not(:disabled) {
    border-color: var(--color-danger, #b74c4c);
    color: var(--color-danger, #b74c4c);
  }

  .delete-btn:disabled {
    cursor: not-allowed;
    opacity: 0.55;
  }

  .schema-label {
    margin-left: auto;
    color: var(--color-accent);
    font-size: 0.7rem;
    font-weight: 600;
    text-transform: uppercase;
  }

  .detail-body {
    flex: 1;
    overflow-y: auto;
    padding: 12px;
  }

  h2,
  p {
    margin: 0;
  }

  h2 {
    color: var(--color-text-primary);
    font-size: 1rem;
    font-weight: 600;
  }

  .metadata {
    display: grid;
    gap: 8px;
    margin: 12px 0 0;
  }

  .metadata div {
    display: grid;
    gap: 2px;
  }

  dt {
    color: var(--color-text-muted);
    font-size: 0.65rem;
    font-weight: 600;
    text-transform: uppercase;
  }

  dd {
    margin: 0;
    color: var(--color-text-secondary);
    font-size: 0.8rem;
    overflow-wrap: anywhere;
  }

  .muted,
  .status {
    color: var(--color-text-muted);
    font-size: 0.8rem;
  }

  .status {
    padding: 12px;
  }

  .error {
    color: var(--color-danger, #b74c4c);
  }
</style>
