<script lang="ts">
  import type { ScriptBlockProjection } from '$lib/scriptTypes.js';
  import {
    applyScriptBlockEditCommand,
    refreshScriptDocumentProjection,
  } from '$lib/stores/scriptDocumentProjection.svelte.js';
  import ScriptView from './ScriptView.svelte';

  let { documentId, block }: { documentId: string; block: ScriptBlockProjection } = $props();
  let editing = $state(false);
  let draft = $state('');
  let baseRevision = $state<string | null>(null);
  let saving = $state(false);
  let error = $state<string | null>(null);

  function beginEdit() {
    draft = block.block.text;
    baseRevision = block.revision_event_id ?? null;
    error = null;
    editing = true;
  }

  async function save() {
    if (!baseRevision || saving) return;
    saving = true;
    error = null;
    try {
      await applyScriptBlockEditCommand({
        document_id: documentId,
        block_id: block.block.id,
        expected_revision_event_id: baseRevision,
        text: draft,
      });
      editing = false;
    } catch (failure) {
      error = failure instanceof Error ? failure.message : 'Could not save script block';
    } finally {
      saving = false;
    }
  }

  async function reload() {
    try {
      await refreshScriptDocumentProjection({ document_id: documentId });
      editing = false;
      error = null;
    } catch (failure) {
      error = failure instanceof Error ? failure.message : 'Could not reload script block';
    }
  }
</script>

<section class="script-block" aria-label="Screenplay block">
  {#if editing}
    <label for={`script-edit-${block.block.id}`}>Edit screenplay text</label>
    <textarea id={`script-edit-${block.block.id}`} bind:value={draft} disabled={saving} rows="8"
    ></textarea>
    {#if block.locks.length}<p>Protected text must remain unchanged.</p>{/if}
    {#if error}<p role="alert">{error} Your draft is still here.</p>{/if}
    <div class="actions">
      <button type="button" onclick={save} disabled={saving || !baseRevision}
        >{saving ? 'Saving…' : 'Save'}</button
      >
      <button
        type="button"
        onclick={() => {
          editing = false;
          error = null;
        }}
        disabled={saving}>Cancel</button
      >
      {#if error}<button type="button" onclick={reload} disabled={saving}
          >Discard draft and reload</button
        >{/if}
    </div>
  {:else}
    <ScriptView text={block.block.text} />
    <button type="button" onclick={beginEdit} disabled={!block.revision_event_id}>Edit</button>
  {/if}
</section>

<style>
  .script-block {
    break-inside: avoid;
    margin-bottom: 12px;
  }
  textarea {
    width: 100%;
    box-sizing: border-box;
    font-family: 'Courier New', monospace;
    background: var(--color-bg-surface);
    color: var(--color-text-primary);
  }
  label,
  p {
    font-size: 0.8rem;
  }
  .actions {
    display: flex;
    gap: 8px;
  }
</style>
