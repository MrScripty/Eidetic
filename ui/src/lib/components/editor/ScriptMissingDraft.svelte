<script lang="ts">
  import type { createScriptBlockEditDraft } from './scriptBlockEditDraft.svelte.js';
  import type { SelectedNodeEditorNode } from '$lib/selectedNodeEditorTypes.js';
  import { getSessionScriptBlockCreationDraft } from '$lib/stores/scriptBlockCreationSession.svelte.js';
  import { copyMissingScriptDraft } from './scriptMissingDraft.js';
  let {
    editor,
    source,
  }: {
    editor: ReturnType<typeof createScriptBlockEditDraft>;
    source: SelectedNodeEditorNode | null;
  } = $props();
  let draft = $derived(editor.state);
  let composer = $derived(getSessionScriptBlockCreationDraft());
  let busy = $derived(draft.saving || draft.comparing || draft.uncertain);
</script>

<section class="retained-draft" aria-label="Retained screenplay draft">
  <strong>Your draft is retained</strong>
  <p>
    This saved block is unavailable in the current screenplay view. Check saved screenplay before
    deciding what to do.
  </p>
  <details>
    <summary>Original source</summary>
    <p>Block: {editor.blockId}</p>
    <p>Saved revision: {draft.baseRevision}</p>
  </details>
  <label for={`retained-${editor.blockId}`}>Retained screenplay text</label>
  <textarea id={`retained-${editor.blockId}`} bind:value={draft.text} disabled={busy} rows="6"
  ></textarea>
  {#if draft.error}<p role="alert">{draft.error} Your draft is still here.</p>{/if}
  {#if draft.uncertain}<p>
      The original save may have completed. Retry that exact save before copying or discarding.
    </p>{/if}
  <div class="actions">
    {#if draft.uncertain}<button
        type="button"
        onclick={editor.save}
        disabled={draft.saving || draft.comparing}>Retry original save</button
      >{/if}
    <button type="button" onclick={editor.compare} disabled={busy}>Check saved screenplay</button>
    <button
      type="button"
      onclick={() => copyMissingScriptDraft(editor, composer, source)}
      disabled={busy ||
        !source ||
        composer.state.writing ||
        composer.state.saving ||
        composer.state.uncertain}>Copy draft into new screenplay</button
    >
    <button type="button" onclick={editor.cancel} disabled={busy}>Discard retained draft</button>
  </div>
  {#if source}<p>
      Copy starts a new block for <strong>{source.name}</strong>. Review it in Write screenplay and
      explicitly Save. This original draft stays here until discarded.
    </p>{:else}<p>Select a timeline clip to copy this text into a new screenplay draft.</p>{/if}
</section>

<style>
  .retained-draft {
    break-inside: avoid;
    margin-bottom: 12px;
    padding: 8px;
    border: 1px solid var(--color-border-subtle);
  }
  p,
  label,
  details {
    font-size: 0.8rem;
  }
  label {
    display: block;
  }
  textarea {
    width: 100%;
    box-sizing: border-box;
    font-family: 'Courier New', monospace;
    background: var(--color-bg-surface);
    color: var(--color-text-primary);
  }
  .actions {
    display: flex;
    flex-wrap: wrap;
    gap: 8px;
  }
</style>
