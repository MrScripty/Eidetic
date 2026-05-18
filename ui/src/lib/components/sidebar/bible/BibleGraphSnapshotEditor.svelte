<script lang="ts">
  import { setBibleGraphSnapshotFieldProjection } from '$lib/stores/bibleGraphNodeProjection.svelte.js';
  import type { BibleGraphNodeId, BibleGraphPartProjection } from '$lib/bibleGraphTypes.js';

  let {
    nodeId,
    parts,
    nextSortOrder,
  }: {
    nodeId: BibleGraphNodeId;
    parts: BibleGraphPartProjection[];
    nextSortOrder: number;
  } = $props();

  const fieldOptions = $derived(
    parts.flatMap((partProjection) =>
      partProjection.fields.map((field) => ({
        id: `${partProjection.part.id}::${field.id}`,
        part: partProjection.part,
        field,
      })),
    ),
  );

  let selectedFieldId = $state('');
  let label = $state('');
  let atMs = $state(0);
  let value = $state('');
  let saving = $state(false);
  let error = $state<string | undefined>(undefined);

  const selectedOption = $derived(
    fieldOptions.find((option) => option.id === selectedFieldId) ?? fieldOptions[0],
  );

  async function addSnapshot(): Promise<void> {
    const snapshotLabel = label.trim();
    const snapshotValue = value.trim();
    if (!selectedOption || !snapshotLabel) return;

    saving = true;
    error = undefined;
    try {
      const snapshotId = `snapshot.${crypto.randomUUID()}`;
      await setBibleGraphSnapshotFieldProjection({
        snapshot_id: snapshotId,
        node_id: nodeId,
        at_ms: Math.max(0, Math.trunc(atMs)),
        label: snapshotLabel,
        snapshot_sort_order: nextSortOrder,
        field_id: `snapshot-field.${crypto.randomUUID()}`,
        part_key: selectedOption.part.part_key,
        part_name: selectedOption.part.name,
        field_key: selectedOption.field.field_key,
        value: snapshotValue ? { type: 'text', value: snapshotValue } : null,
        field_sort_order: selectedOption.field.sort_order,
      });
      label = '';
      atMs = 0;
      value = '';
      selectedFieldId = '';
    } catch (caught) {
      error = caught instanceof Error ? caught.message : 'Failed to add snapshot';
    } finally {
      saving = false;
    }
  }
</script>

<section class="snapshot-editor">
  <h3>New Snapshot</h3>
  <div class="snapshot-form">
    <label>
      <span>Label</span>
      <input bind:value={label} />
    </label>
    <label>
      <span>Time</span>
      <input type="number" min="0" step="1" bind:value={atMs} />
    </label>
    <label>
      <span>Field</span>
      <select bind:value={selectedFieldId} disabled={fieldOptions.length === 0}>
        {#each fieldOptions as option (option.id)}
          <option value={option.id}>{option.part.name} / {option.field.field_key}</option>
        {/each}
      </select>
    </label>
    <label>
      <span>Value</span>
      <textarea rows="2" bind:value></textarea>
    </label>
  </div>
  <div class="snapshot-actions">
    {#if error}
      <p class="snapshot-error">{error}</p>
    {/if}
    <button
      type="button"
      disabled={saving || !selectedOption || !label.trim()}
      onclick={addSnapshot}
    >
      {saving ? 'Saving' : 'Add Snapshot'}
    </button>
  </div>
</section>

<style>
  .snapshot-editor {
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

  .snapshot-form {
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
  select,
  textarea {
    border: 1px solid var(--color-border-subtle);
    border-radius: 4px;
    background: var(--color-bg-input, var(--color-bg-panel));
    color: var(--color-text-secondary);
    font: inherit;
    font-size: 0.8rem;
    padding: 6px;
  }

  textarea {
    min-height: 42px;
    resize: vertical;
  }

  input:focus,
  select:focus,
  textarea:focus {
    border-color: var(--color-accent);
    outline: none;
  }

  .snapshot-actions {
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

  .snapshot-error {
    margin-right: auto;
    color: var(--color-danger, #b74c4c);
    font-size: 0.8rem;
  }
</style>
