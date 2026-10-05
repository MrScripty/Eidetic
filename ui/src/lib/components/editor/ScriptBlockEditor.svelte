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
      disabled={draft.saving || draft.comparing || draft.uncertain}
      rows="8"
    ></textarea>
    {#if block.locks.length}<p>Protected text must remain unchanged.</p>{/if}
    {#if draft.error}<p role="alert">{draft.error} Your draft is still here.</p>{/if}
    {#if draft.uncertain}<p>
        The save may have completed. Retry to confirm it before changing this text.
      </p>{/if}
    <div class="actions">
      <button
        type="button"
        onclick={editor.compare}
        disabled={draft.saving || draft.comparing || draft.uncertain}
        >{draft.comparing ? 'Reading saved text…' : 'Compare saved text'}</button
      >
      <button
        type="button"
        onclick={editor.save}
        disabled={draft.saving || draft.comparing || !draft.baseRevision}
        >{draft.saving ? 'Saving…' : draft.uncertain ? 'Retry same save' : 'Save'}</button
      >
      <button
        type="button"
        onclick={editor.cancel}
        disabled={draft.saving || draft.comparing || draft.uncertain}>Cancel</button
      >
      {#if draft.error && !draft.uncertain}<button
          type="button"
          onclick={editor.reload}
          disabled={draft.saving || draft.comparing}>Discard draft and reload</button
        >{/if}
    </div>
    {#if draft.comparison}
      <section class="comparison" aria-label="Saved text comparison">
        <strong>Saved text</strong>
        <pre>{draft.comparison.text}</pre>
        {#if block.revision_event_id !== draft.comparison.revisionEventId}
          <p>Saved text changed again. Compare it before continuing.</p>
        {/if}
        <p>Your draft stays unchanged. Save applies it to this version.</p>
        <button
          type="button"
          onclick={() => editor.useComparedRevision(block)}
          disabled={draft.saving ||
            draft.comparing ||
            draft.uncertain ||
            block.revision_event_id !== draft.comparison.revisionEventId}
          >Continue draft from this version</button
        >
      </section>
    {/if}
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
  .comparison {
    margin-top: 8px;
    padding: 8px;
    border: 1px solid var(--color-border-subtle);
  }
  pre {
    white-space: pre-wrap;
    overflow-wrap: anywhere;
  }
  .actions {
    display: flex;
    gap: 8px;
    flex-wrap: wrap;
  }
</style>
