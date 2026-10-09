<script lang="ts">
  import { untrack } from 'svelte';
  import type { SelectedNodeEditorNode } from '$lib/selectedNodeEditorTypes.js';
  import { getSessionTimelineNotesDraft } from '$lib/stores/timelineNotesSession.svelte.js';
  let { node }: { node: SelectedNodeEditorNode } = $props();
  const draft = $derived(getSessionTimelineNotesDraft(node));
  $effect(() => {
    const owner = draft;
    const read = node.notes_read;
    untrack(() => owner.observe(read));
  });
  const dirty = $derived(draft.state.base !== null && draft.state.notes !== draft.state.base.notes);
  const changed = $derived(
    draft.state.base !== null &&
      draft.state.current !== null &&
      (draft.state.base.notes !== draft.state.current.notes ||
        draft.state.base.revision_event_id !== draft.state.current.revision_event_id),
  );
</script>

<label class="section-label" for={`notes-${node.node_id}`}>Notes</label>
<textarea
  id={`notes-${node.node_id}`}
  class="notes-input"
  placeholder="Describe what happens in this {node.level.toLowerCase()}..."
  value={draft.state.base ? draft.state.notes : node.notes}
  oninput={(event) => {
    draft.state.notes = event.currentTarget.value;
    draft.state.saved = false;
  }}
  disabled={node.locked || !draft.state.base || draft.state.busy || draft.state.uncertain}
></textarea>
<div class="notes-actions">
  <button
    type="button"
    class="notes-save"
    onclick={() => void draft.apply()}
    disabled={(node.locked && !draft.state.uncertain) ||
      draft.state.busy ||
      !draft.state.base ||
      (!draft.state.uncertain && (!dirty || changed))}
  >
    {draft.state.busy ? 'Saving…' : draft.state.uncertain ? 'Retry exact save' : 'Save Notes'}
  </button>
  <button
    type="button"
    onclick={() => void draft.reload()}
    disabled={draft.state.busy || draft.state.uncertain}
  >
    {dirty ? 'Discard draft and read saved Notes' : 'Read saved Notes'}
  </button>
</div>
{#if dirty}<p class="notes-status" role="status">
    Unsaved Notes draft. Kept while you switch clips or panels in this project session.
  </p>{/if}
{#if draft.state.saved}<p class="notes-status" role="status">Notes saved.</p>{/if}
{#if changed}
  <p class="error-banner" role="alert">
    Saved Notes changed since this draft was read. Your draft is kept.
  </p>
  <details>
    <summary>Current saved Notes</summary>
    <pre>{draft.state.current?.notes}</pre>
  </details>
{/if}
{#if draft.state.uncertain}<p class="notes-status" role="status">
    Save completion is uncertain. Retry the original save before discarding this draft.
  </p>{/if}
{#if draft.state.error}<p class="error-banner" role="alert">{draft.state.error}</p>{/if}

<style>
  .notes-actions {
    display: flex;
    gap: 0.5rem;
    flex-wrap: wrap;
    margin-top: 0.5rem;
  }
  .notes-status {
    font-size: 0.8rem;
    color: var(--color-text-secondary);
  }
  button {
    font-size: 0.75rem;
    padding: 3px 10px;
    border: 1px solid var(--color-border-default);
    border-radius: 6px;
    background: var(--color-bg-surface);
    color: var(--color-text-secondary);
    cursor: pointer;
  }
  button:disabled {
    opacity: 0.5;
    cursor: not-allowed;
  }
  .notes-save {
    border-color: var(--color-accent);
    color: var(--color-accent);
  }
  pre {
    white-space: pre-wrap;
    overflow-wrap: anywhere;
  }
</style>
