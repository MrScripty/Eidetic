<script lang="ts">
  import type { BibleGraphNodeId, FieldValue } from '$lib/types.js';
  import {
    getBibleGraphNodeProjectionError,
    getCachedBibleGraphNodeProjection,
    isBibleGraphNodeProjectionPending,
    refreshBibleGraphNodeProjection,
  } from '$lib/stores/bibleGraphNodeProjection.svelte.js';

  let {
    nodeId,
    onclose,
  }: {
    nodeId: BibleGraphNodeId;
    onclose: () => void;
  } = $props();

  const key = $derived({ node_id: nodeId });
  const projection = $derived(getCachedBibleGraphNodeProjection(key));
  const pending = $derived(isBibleGraphNodeProjectionPending(key));
  const error = $derived(getBibleGraphNodeProjectionError(key));

  $effect(() => {
    void refreshBibleGraphNodeProjection({ node_id: nodeId }).catch(() => {});
  });

  function formatFieldValue(value: FieldValue | null | undefined): string {
    if (!value) return 'Unset';
    switch (value.type) {
      case 'text':
        return value.value;
      case 'integer':
      case 'number':
        return value.value.toString();
      case 'bool':
        return value.value ? 'True' : 'False';
      case 'object_ref':
        return `${value.value.kind}: ${value.value.id}`;
      case 'asset_ref':
        return value.value;
    }
  }
</script>

<div class="graph-node-detail">
  <div class="detail-header">
    <button class="close-btn" onclick={onclose}>&times; Close</button>
    {#if projection}
      <span class="schema-label">{projection.payload.node.schema_key}</span>
    {/if}
  </div>

  {#if projection}
    <div class="detail-body">
      <h2>{projection.payload.node.name}</h2>
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
        <section class="part-section">
          <h3>{partProjection.part.name}</h3>
          {#if partProjection.fields.length > 0}
            <dl class="field-list">
              {#each partProjection.fields as field (field.id)}
                <div>
                  <dt>{field.field_key}</dt>
                  <dd>{formatFieldValue(field.value)}</dd>
                </div>
              {/each}
            </dl>
          {:else}
            <p class="muted">No fields</p>
          {/if}
        </section>
      {/each}

      {#if projection.payload.parts.length === 0}
        <p class="muted">No parts</p>
      {/if}
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
  h3,
  p {
    margin: 0;
  }

  h2 {
    color: var(--color-text-primary);
    font-size: 1rem;
    font-weight: 600;
  }

  h3 {
    color: var(--color-text-primary);
    font-size: 0.85rem;
    font-weight: 600;
  }

  .metadata,
  .field-list {
    display: grid;
    gap: 8px;
    margin: 12px 0 0;
  }

  .metadata div,
  .field-list div {
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

  .part-section {
    margin-top: 16px;
    padding-top: 12px;
    border-top: 1px solid var(--color-border-subtle);
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
