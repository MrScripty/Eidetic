<script lang="ts">
  import type { ScriptBlockProjection } from '$lib/scriptTypes.js';
  import { getSessionScriptBlockEditDraft } from '$lib/stores/scriptBlockEditSession.svelte.js';
  import ScriptView from './ScriptView.svelte';

  let { documentId, block }: { documentId: string; block: ScriptBlockProjection } = $props();
  let editor = $derived(getSessionScriptBlockEditDraft(documentId, block.block.id));
  let draft = $derived(editor.state);
</script>

<section class="script-block" aria-label="Screenplay block">
  {#if draft.editing}
    <label for={`script-edit-${block.block.id}`}>Edit screenplay text</label>
    <textarea
      id={`script-edit-${block.block.id}`}
      bind:value={draft.text}
      disabled={draft.saving || draft.uncertain}
      rows="8"
    ></textarea>
    {#if block.locks.length}<p>Protected text must remain unchanged.</p>{/if}
    {#if draft.error}<p role="alert">{draft.error} Your draft is still here.</p>{/if}
    {#if draft.uncertain}<p>
        The save may have completed. Retry to confirm it before changing this text.
      </p>{/if}
    <div class="actions">
      <button type="button" onclick={editor.save} disabled={draft.saving || !draft.baseRevision}
        >{draft.saving ? 'Saving…' : draft.uncertain ? 'Retry same save' : 'Save'}</button
      >
      <button type="button" onclick={editor.cancel} disabled={draft.saving || draft.uncertain}
        >Cancel</button
      >
      {#if draft.error && !draft.uncertain}<button
          type="button"
          onclick={editor.reload}
          disabled={draft.saving}>Discard draft and reload</button
        >{/if}
    </div>
  {:else}
    <ScriptView text={block.block.text} />
    <button type="button" onclick={() => editor.begin(block)} disabled={!block.revision_event_id}
      >Edit</button
    >
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
