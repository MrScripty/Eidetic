<script lang="ts">
  import type { SelectedNodeEditorNode } from '$lib/selectedNodeEditorTypes.js';
  import { getSessionTimelineTitleDraft } from '$lib/stores/timelineTitleSession.svelte.js';
  let { node }: { node: SelectedNodeEditorNode } = $props();
  let draft = $derived(getSessionTimelineTitleDraft(node));
  $effect(() => draft.initialize(node.name_read));
</script>

<section aria-label="Clip title" class="title-editor">
  <label
    >Clip title<input
      aria-label="Clip title"
      bind:value={draft.state.name}
      disabled={draft.state.busy || draft.state.uncertain}
    /></label
  >
  <button
    type="button"
    onclick={() => void draft.apply()}
    disabled={draft.state.busy || !draft.state.base}
  >
    {draft.state.busy ? 'Saving title…' : draft.state.uncertain ? 'Retry title save' : 'Save title'}
  </button>
  <button
    type="button"
    onclick={() => void draft.reload()}
    disabled={draft.state.busy || draft.state.uncertain}>Discard title draft and reload</button
  >
  {#if draft.state.error}<p role="alert">{draft.state.error}</p>{/if}
  {#if draft.state.uncertain}<p>
      Completion is unknown. Retry the exact title before changing it.
    </p>{/if}
  {#if draft.state.saved}<p role="status">
      Title saved. Review affected screenplay before replacing it.
    </p>{/if}
</section>

<style>
  .title-editor {
    margin: 0.5rem 0;
    font-size: 0.8rem;
  }
  label {
    display: flex;
    gap: 0.4rem;
    align-items: center;
  }
  input {
    flex: 1;
    padding: 0.25rem;
    border: 1px solid var(--color-border-default);
  }
  button {
    margin: 0.25rem;
    padding: 0.3rem;
    border: 1px solid var(--color-border-default);
  }
</style>
