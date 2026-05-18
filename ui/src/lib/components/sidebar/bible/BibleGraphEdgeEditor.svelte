<script lang="ts">
  import { setBibleGraphEdgeProjection } from '$lib/stores/bibleGraphNodeProjection.svelte.js';
  import type { BibleGraphEdgeKind, BibleGraphNodeId } from '$lib/bibleGraphTypes.js';

  let {
    sourceNodeId,
    nextSortOrder,
  }: {
    sourceNodeId: BibleGraphNodeId;
    nextSortOrder: number;
  } = $props();

  const edgeKinds: Exclude<BibleGraphEdgeKind, { custom: string }>[] = [
    'references',
    'located_in',
    'owns',
    'member_of',
    'conflicts_with',
    'supports_theme',
  ];

  let targetNodeId = $state('');
  let label = $state('');
  let edgeKind = $state<Exclude<BibleGraphEdgeKind, { custom: string }>>('references');
  let saving = $state(false);
  let error = $state<string | undefined>(undefined);

  async function addEdge(): Promise<void> {
    const target = targetNodeId.trim();
    const edgeLabel = label.trim();
    if (!target || !edgeLabel) return;

    saving = true;
    error = undefined;
    try {
      await setBibleGraphEdgeProjection({
        edge_id: `edge.${crypto.randomUUID()}`,
        from_node_id: sourceNodeId,
        to_node_id: target,
        edge_kind: edgeKind,
        label: edgeLabel,
        directed: true,
        sort_order: nextSortOrder,
      });
      targetNodeId = '';
      label = '';
      edgeKind = 'references';
    } catch (caught) {
      error = caught instanceof Error ? caught.message : 'Failed to add edge';
    } finally {
      saving = false;
    }
  }
</script>

<section class="edge-editor">
  <h3>New Edge</h3>
  <div class="edge-form">
    <label>
      <span>Target Node</span>
      <input bind:value={targetNodeId} />
    </label>
    <label>
      <span>Kind</span>
      <select bind:value={edgeKind}>
        {#each edgeKinds as kind}
          <option value={kind}>{kind.replace('_', ' ')}</option>
        {/each}
      </select>
    </label>
    <label>
      <span>Label</span>
      <input bind:value={label} />
    </label>
  </div>
  <div class="edge-actions">
    {#if error}
      <p class="edge-error">{error}</p>
    {/if}
    <button
      type="button"
      disabled={saving || !targetNodeId.trim() || !label.trim()}
      onclick={addEdge}
    >
      {saving ? 'Saving' : 'Add Edge'}
    </button>
  </div>
</section>

<style>
  .edge-editor {
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

  .edge-form {
    display: grid;
    gap: 8px;
    margin-top: 12px;
  }

  label {
    display: grid;
    gap: 4px;
  }

  label span {
    color: var(--color-text-muted);
    font-size: 0.65rem;
    font-weight: 600;
    text-transform: uppercase;
  }

  input,
  select {
    border: 1px solid var(--color-border-subtle);
    border-radius: 4px;
    background: var(--color-bg-input, var(--color-bg-panel));
    color: var(--color-text-secondary);
    font: inherit;
    font-size: 0.8rem;
    padding: 6px;
  }

  input:focus,
  select:focus {
    border-color: var(--color-accent);
    outline: none;
  }

  .edge-actions {
    display: flex;
    align-items: center;
    justify-content: flex-end;
    gap: 8px;
    margin-top: 8px;
  }

  button {
    border: 1px solid var(--color-border-subtle);
    border-radius: 4px;
    background: var(--color-bg-button, var(--color-bg-panel));
    color: var(--color-text-secondary);
    cursor: pointer;
    font-size: 0.75rem;
    padding: 4px 8px;
  }

  button:disabled {
    cursor: default;
    opacity: 0.65;
  }

  .edge-error {
    margin-right: auto;
    color: var(--color-danger, #b74c4c);
    font-size: 0.8rem;
  }
</style>
