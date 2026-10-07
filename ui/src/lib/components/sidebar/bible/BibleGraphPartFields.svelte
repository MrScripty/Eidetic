<script lang="ts">
  import { setBibleGraphFieldProjection } from '$lib/stores/bibleGraphNodeProjection.svelte.js';
  import type { BibleGraphNodeId, BibleGraphPartProjection } from '$lib/bibleGraphTypes.js';
  import { getEditorSessionGeneration } from '$lib/stores/editor.svelte.js';
  import {
    createBibleGraphFieldDrafts,
    formatBibleFieldValue,
  } from './bibleGraphFieldDrafts.svelte.js';

  let {
    nodeId,
    partProjection,
  }: {
    nodeId: BibleGraphNodeId;
    partProjection: BibleGraphPartProjection;
  } = $props();

  const editor = createBibleGraphFieldDrafts({
    owner: () => JSON.stringify([getEditorSessionGeneration(), nodeId, partProjection.part.id]),
    fields: () => partProjection.fields,
    save: (field, text) =>
      setBibleGraphFieldProjection({
        node_id: nodeId,
        part_id: partProjection.part.id,
        part_key: partProjection.part.part_key,
        part_name: partProjection.part.name,
        part_sort_order: partProjection.part.sort_order,
        field_id: field.id,
        field_key: field.field_key,
        value: text ? { type: 'text', value: text } : null,
        field_sort_order: field.sort_order,
      }),
  });
  $effect(() => {
    editor.observe();
  });
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
            value={editor.value(field)}
            disabled={editor.state.saving[field.id]}
            oninput={(event) => editor.update(field, event.currentTarget.value)}
          ></textarea>
        </label>
        {#if editor.changed(field)}
          <p role="status">Saved fact changed while editing. Your draft is preserved.</p>
          <p>Draft started from</p>
          <pre>{editor.state.drafts[field.id]?.baseText}</pre>
          <p>Saved fact</p>
          <pre aria-label="Committed Bible fact">{formatBibleFieldValue(field.value)}</pre>
        {/if}
        <div class="field-actions">
          {#if editor.state.errors[field.id]}
            <p class="field-error">{editor.state.errors[field.id]}</p>
          {/if}
          {#if editor.state.drafts[field.id]}
            <button
              type="button"
              disabled={editor.state.saving[field.id]}
              onclick={() => editor.discard(field)}>Discard draft</button
            >
          {/if}
          <button
            type="button"
            disabled={editor.state.saving[field.id] || editor.changed(field)}
            onclick={() => editor.save(field)}
          >
            {editor.state.saving[field.id] ? 'Saving' : 'Save'}
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
