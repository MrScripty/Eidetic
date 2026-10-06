<script lang="ts">
  import type { SelectedNodeEditorNode } from '$lib/selectedNodeEditorTypes.js';
  import { getSessionTimelinePlacementDraft } from '$lib/stores/timelinePlacementSession.svelte.js';
  let { node }: { node: SelectedNodeEditorNode } = $props();
  let draft = $derived(getSessionTimelinePlacementDraft(node));
  $effect(() => draft.initialize(node.range_read));
</script>

<section aria-label="Clip placement" class="placement-editor">
  <details>
    <summary>Placement: {node.start_ms / 1000}–{node.end_ms / 1000} seconds</summary>
    <p>
      Screen time. Applying moves or resizes the clip and its children. Affected screenplay stays
      available for review.
    </p>
    <label
      >Start (seconds)<input
        aria-label="Placement start seconds"
        inputmode="decimal"
        bind:value={draft.state.start}
        disabled={draft.state.busy || draft.state.uncertain}
      /></label
    >
    <label
      >End (seconds)<input
        aria-label="Placement end seconds"
        inputmode="decimal"
        bind:value={draft.state.end}
        disabled={draft.state.busy || draft.state.uncertain}
      /></label
    >
    <button
      type="button"
      onclick={() => void draft.apply()}
      disabled={draft.state.busy || !draft.state.base}
    >
      {draft.state.busy
        ? 'Applying…'
        : draft.state.uncertain
          ? 'Retry placement'
          : 'Apply placement'}
    </button>
    <button
      type="button"
      onclick={() => void draft.reload()}
      disabled={draft.state.busy || draft.state.uncertain}
      >Discard placement draft and reload</button
    >
    {#if draft.state.error}<p role="alert">{draft.state.error}</p>{/if}
    {#if draft.state.uncertain}<p>
        Completion is unknown. Retry the exact placement before changing it.
      </p>{/if}
    {#if draft.state.saved}<p role="status">
        Placement saved. Review affected screenplay before replacing it.
      </p>{/if}
  </details>
</section>

<style>
  .placement-editor {
    margin: 0.5rem 0;
    font-size: 0.8rem;
  }
  summary {
    cursor: pointer;
  }
  label {
    display: inline-flex;
    flex-direction: column;
    margin: 0.4rem;
    gap: 0.2rem;
  }
  input {
    width: 8rem;
    padding: 0.25rem;
    border: 1px solid var(--color-border-default);
  }
  button {
    margin: 0.25rem;
    padding: 0.3rem;
    border: 1px solid var(--color-border-default);
  }
</style>
