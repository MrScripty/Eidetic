<script lang="ts">
  import { setBibleGraphFieldProjection } from '$lib/stores/bibleGraphNodeProjection.svelte.js';
  import type {
    BibleGraphField,
    BibleGraphNodeId,
    BibleGraphPartProjection,
    FieldValue,
  } from '$lib/types.js';

  let {
    nodeId,
    partProjection,
  }: {
    nodeId: BibleGraphNodeId;
    partProjection: BibleGraphPartProjection;
  } = $props();

  let drafts = $state<Record<string, string>>({});
  let saving = $state<Record<string, boolean>>({});
  let errors = $state<Record<string, string | undefined>>({});

  function fieldInputValue(field: BibleGraphField): string {
    if (field.id in drafts) return drafts[field.id] ?? '';
    return formatFieldValue(field.value);
  }

  function formatFieldValue(value: FieldValue | null | undefined): string {
    if (!value) return '';
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

  function updateDraft(field: BibleGraphField, event: Event): void {
    drafts[field.id] = (event.currentTarget as HTMLTextAreaElement).value;
  }

  async function saveField(field: BibleGraphField): Promise<void> {
    saving[field.id] = true;
    errors[field.id] = undefined;
    const draft = fieldInputValue(field).trim();

    try {
      await setBibleGraphFieldProjection({
        node_id: nodeId,
        part_id: partProjection.part.id,
        part_key: partProjection.part.part_key,
        part_name: partProjection.part.name,
        part_sort_order: partProjection.part.sort_order,
        field_id: field.id,
        field_key: field.field_key,
        value: draft ? { type: 'text', value: draft } : null,
        field_sort_order: field.sort_order,
      });
      delete drafts[field.id];
    } catch (error) {
      errors[field.id] = error instanceof Error ? error.message : 'Failed to save field';
    } finally {
      saving[field.id] = false;
    }
  }
</script>

<section class="part-section">
  <h3>{partProjection.part.name}</h3>
  {#if partProjection.fields.length > 0}
    <div class="field-list">
      {#each partProjection.fields as field (field.id)}
        <label>
          <span>{field.field_key}</span>
          <textarea
            rows="2"
            value={fieldInputValue(field)}
            oninput={(event) => updateDraft(field, event)}
          ></textarea>
        </label>
        <div class="field-actions">
          {#if errors[field.id]}
            <p class="field-error">{errors[field.id]}</p>
          {/if}
          <button type="button" disabled={saving[field.id]} onclick={() => saveField(field)}>
            {saving[field.id] ? 'Saving' : 'Save'}
          </button>
        </div>
      {/each}
    </div>
  {:else}
    <p class="muted">No fields</p>
  {/if}
</section>

<style>
  .part-section {
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

  .field-list {
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

  textarea {
    min-height: 42px;
    resize: vertical;
    border: 1px solid var(--color-border-subtle);
    border-radius: 4px;
    background: var(--color-bg-input, var(--color-bg-panel));
    color: var(--color-text-secondary);
    font: inherit;
    font-size: 0.8rem;
    line-height: 1.35;
    padding: 6px;
  }

  textarea:focus {
    border-color: var(--color-accent);
    outline: none;
  }

  .field-actions {
    display: flex;
    align-items: center;
    justify-content: flex-end;
    gap: 8px;
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

  .muted,
  .field-error {
    color: var(--color-text-muted);
    font-size: 0.8rem;
  }

  .field-error {
    margin-right: auto;
    color: var(--color-danger, #b74c4c);
  }
</style>
