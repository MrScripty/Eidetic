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
    refreshBibleGraphNodeListProjection,
    refreshBibleGraphNodeProjection,
    setBibleGraphNodeNameProjection,
  } from '$lib/stores/bibleGraphNodeProjection.svelte.js';
  import { createConnectedBibleGraphChildNode } from './bibleGraphNodeCreateFlow.js';
  import BibleGraphEdgeEditor from './BibleGraphEdgeEditor.svelte';
  import { bibleGraphEdgeTargetOptions } from './bibleGraphEdgeTargetOptions.js';
  import BibleGraphEdgeList from './BibleGraphEdgeList.svelte';
  import BibleGraphPartFields from './BibleGraphPartFields.svelte';
  import BibleGraphSnapshotEditor from './BibleGraphSnapshotEditor.svelte';
  import BibleGraphSnapshotList from './BibleGraphSnapshotList.svelte';

  let {
    nodeId,
    onclose,
    onselect,
    ondeleted,
    oncreated,
    onrenamed,
    graphNodes = undefined,
    edgeTargetNodes,
  }: {
    nodeId: BibleGraphNodeId;
    onclose: () => void;
    onselect?: (nodeId: BibleGraphNodeId) => void;
    ondeleted?: () => void | Promise<void>;
    oncreated?: (nodeId: BibleGraphNodeId) => void | Promise<void>;
    onrenamed?: (nodeId: BibleGraphNodeId) => void | Promise<void>;
    graphNodes?: BibleGraphNode[];
    edgeTargetNodes: Array<BibleGraphNode | BibleRenderGraphNode>;
  } = $props();

  let deletingNode = $state(false);
  let creatingChild = $state(false);
  let renamingNode = $state(false);
  let editingName = $state(false);
  let nameDraft = $state('');
  let nameInput = $state<HTMLInputElement | undefined>(undefined);
  let deleteError = $state<string | undefined>(undefined);
  let createError = $state<string | undefined>(undefined);
  let renameError = $state<string | undefined>(undefined);

  const key = $derived({ node_id: nodeId });
  const projection = $derived(getCachedBibleGraphNodeProjection(key));
  const pending = $derived(isBibleGraphNodeProjectionPending(key));
  const error = $derived(getBibleGraphNodeProjectionError(key));
  const edgeTargetOptions = $derived(bibleGraphEdgeTargetOptions(edgeTargetNodes, nodeId));
  const childNodes = $derived(
    graphNodes
      ? graphNodes
          .filter((node) => node.parent_id === nodeId)
          .sort(
            (left, right) =>
              left.sort_order - right.sort_order || left.name.localeCompare(right.name),
          )
      : [],
  );
  const childCreateDisabledReason = $derived(() => {
    if (!projection) return 'Node projection is not loaded';
    return null;
  });
  const renameDisabledReason = $derived(() => {
    if (!projection) return 'Node projection is not loaded';
    if (projection.payload.node.system_owned) return 'Canonical roots cannot be renamed';
    return null;
  });
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

  $effect(() => {
    if (!editingName && projection) {
      nameDraft = projection.payload.node.name;
    }
  });

  $effect(() => {
    if (editingName && nameInput) {
      nameInput.focus();
      nameInput.select();
    }
  });

  function startNameEdit(): void {
    if (!projection) return;
    if (renameDisabledReason() !== null) return;
    nameDraft = projection.payload.node.name;
    renameError = undefined;
    editingName = true;
  }

  function cancelNameEdit(): void {
    nameDraft = projection?.payload.node.name ?? '';
    renameError = undefined;
    editingName = false;
  }

  async function handleRenameNode(): Promise<void> {
    if (!projection) return;
    if (renameDisabledReason() !== null) return;
    const name = nameDraft.trim();
    if (name.length === 0) {
      renameError = 'Name is required';
      return;
    }
    if (name === projection.payload.node.name) {
      editingName = false;
      return;
    }

    renamingNode = true;
    renameError = undefined;
    try {
      const response = await setBibleGraphNodeNameProjection({
        node_id: projection.payload.node.id,
        name,
      });
      await refreshBibleGraphNodeListProjection();
      editingName = false;
      nameDraft = response.projection.payload.node.name;
      if (onrenamed) {
        await onrenamed(response.projection.payload.node.id);
      }
    } catch (error) {
      renameError = error instanceof Error ? error.message : 'Failed to rename bible graph node';
    } finally {
      renamingNode = false;
    }
  }

  function handleNameFormSubmit(event: SubmitEvent): void {
    event.preventDefault();
    void handleRenameNode();
  }

  async function handleCreateChild(): Promise<void> {
    if (!projection) return;
    if (childCreateDisabledReason() !== null) return;

    creatingChild = true;
    createError = undefined;
    try {
      const response = await createConnectedBibleGraphChildNode(projection.payload.node.id);
      await refreshBibleGraphNodeListProjection();
      await refreshBibleGraphNodeProjection({ node_id: projection.payload.node.id });
      if (oncreated) {
        await oncreated(response.projection.payload.node.id);
      }
    } catch (error) {
      createError = error instanceof Error ? error.message : 'Failed to create bible graph child';
    } finally {
      creatingChild = false;
    }
  }

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
        class="child-add-btn"
        disabled={creatingChild || childCreateDisabledReason() !== null}
        title={childCreateDisabledReason() ?? 'Add child node'}
        onclick={() => void handleCreateChild()}
      >
        {creatingChild ? 'Adding' : 'Add Child'}
      </button>
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
      <form class="name-editor" onsubmit={handleNameFormSubmit}>
        {#if editingName}
          <input
            bind:this={nameInput}
            aria-label="Node name"
            bind:value={nameDraft}
            disabled={renamingNode}
            onkeydown={(event) => {
              if (event.key === 'Escape') {
                event.preventDefault();
                cancelNameEdit();
              }
            }}
          />
          <div class="name-actions">
            <button type="submit" disabled={renamingNode}>
              {renamingNode ? 'Saving' : 'Save'}
            </button>
            <button type="button" disabled={renamingNode} onclick={cancelNameEdit}>Cancel</button>
          </div>
        {:else}
          <button
            type="button"
            class="title-button"
            disabled={renameDisabledReason() !== null}
            title={renameDisabledReason() ?? 'Rename node'}
            ondblclick={startNameEdit}
            onclick={startNameEdit}
          >
            {projection.payload.node.name}
          </button>
        {/if}
      </form>
      {#if deleteError}
        <p class="status error">{deleteError}</p>
      {/if}
      {#if createError}
        <p class="status error">{createError}</p>
      {/if}
      {#if renameError}
        <p class="status error">{renameError}</p>
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

      <section class="child-section">
        <h3>Children</h3>
        {#if childNodes.length > 0}
          <ul class="child-list">
            {#each childNodes as childNode (childNode.id)}
              <li>
                <button type="button" onclick={() => onselect?.(childNode.id)}>
                  {childNode.name}
                </button>
              </li>
            {/each}
          </ul>
        {:else}
          <p class="muted">No child nodes</p>
        {/if}
      </section>

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

  .child-add-btn,
  .delete-btn {
    border: 1px solid var(--color-border-subtle);
    border-radius: 4px;
    background: transparent;
    color: var(--color-text-secondary);
    cursor: pointer;
    font-size: 0.75rem;
    padding: 4px 8px;
  }

  .child-add-btn:hover:not(:disabled) {
    border-color: var(--color-accent);
    color: var(--color-accent);
  }

  .delete-btn:hover:not(:disabled) {
    border-color: var(--color-danger, #b74c4c);
    color: var(--color-danger, #b74c4c);
  }

  .child-add-btn:disabled,
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

  p {
    margin: 0;
  }

  .name-editor {
    display: grid;
    gap: 8px;
  }

  .title-button {
    width: 100%;
    border: 1px solid transparent;
    border-radius: 4px;
    background: transparent;
    color: var(--color-text-primary);
    font-size: 1rem;
    font-weight: 600;
    padding: 4px 0;
    text-align: left;
  }

  .title-button:hover:not(:disabled) {
    border-color: var(--color-border-subtle);
    color: var(--color-accent);
    cursor: text;
  }

  .title-button:disabled {
    opacity: 0.75;
  }

  .name-editor input {
    width: 100%;
    box-sizing: border-box;
    border: 1px solid var(--color-border-subtle);
    border-radius: 4px;
    background: var(--color-bg-surface);
    color: var(--color-text-primary);
    font-size: 0.95rem;
    font-weight: 600;
    padding: 6px 8px;
  }

  .name-actions {
    display: flex;
    gap: 6px;
  }

  .name-actions button {
    border: 1px solid var(--color-border-subtle);
    border-radius: 4px;
    background: transparent;
    color: var(--color-text-secondary);
    cursor: pointer;
    font-size: 0.75rem;
    padding: 4px 8px;
  }

  .metadata {
    display: grid;
    gap: 8px;
    margin: 12px 0 0;
  }

  .child-section {
    display: grid;
    gap: 8px;
    margin-top: 16px;
  }

  .child-section h3 {
    margin: 0;
    color: var(--color-text-secondary);
    font-size: 0.78rem;
    font-weight: 700;
    text-transform: uppercase;
  }

  .child-list {
    display: grid;
    gap: 4px;
    list-style: none;
    margin: 0;
    padding: 0;
  }

  .child-list button {
    width: 100%;
    border: 1px solid var(--color-border-subtle);
    border-radius: 4px;
    background: transparent;
    color: var(--color-text-secondary);
    cursor: pointer;
    font-size: 0.78rem;
    padding: 6px 8px;
    text-align: left;
  }

  .child-list button:hover {
    border-color: var(--color-accent);
    color: var(--color-accent);
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
