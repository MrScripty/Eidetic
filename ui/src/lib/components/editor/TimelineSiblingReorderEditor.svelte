<script lang="ts">
  import { untrack } from 'svelte';
  import type { SelectedNodeEditorNode } from '$lib/selectedNodeEditorTypes.js';
  import { getSessionTimelineReorderDraft } from '$lib/stores/timelineReorderSession.svelte.js';
  import { reordered, sameOrder } from './timelineReorderDraft.svelte.js';
  let { node }: { node: SelectedNodeEditorNode } = $props();
  const draft = $derived(getSessionTimelineReorderDraft(node));
  $effect(() => {
    const owner = draft,
      read = node.order_read;
    untrack(() => owner.observe(read));
  });
  const index = $derived(
    draft.state.base?.siblings.findIndex((n) => n.node_id === node.node_id) ?? -1,
  );
  const earlier = $derived(index > 0 ? draft.state.base?.siblings[index - 1] : null);
  const later = $derived(index >= 0 ? draft.state.base?.siblings[index + 1] : null);
  const preview = $derived(
    draft.state.base && draft.state.neighborId
      ? reordered(draft.state.base, draft.state.neighborId)
      : null,
  );
  const changed = $derived(
    !!draft.state.base &&
      !!draft.state.current &&
      !sameOrder(draft.state.base, draft.state.current),
  );
</script>

<fieldset disabled={!draft.state.base || draft.state.busy || draft.state.uncertain}>
  <legend>Sibling order</legend>
  <button
    type="button"
    disabled={!earlier || !draft.state.base || !reordered(draft.state.base, earlier.node_id)}
    onclick={() => {
      draft.state.neighborId = earlier!.node_id;
      draft.state.saved = false;
    }}>Move earlier</button
  >
  <button
    type="button"
    disabled={!later || !draft.state.base || !reordered(draft.state.base, later.node_id)}
    onclick={() => {
      draft.state.neighborId = later!.node_id;
      draft.state.saved = false;
    }}>Move later</button
  >
  {#if !earlier && !later}<p>No adjacent sibling clips on this track.</p>{/if}
  {#if draft.state.neighborId}
    <p>Unsaved reorder draft. Kept while you switch clips or panels in this project session.</p>
    {#if preview}<p>
        Swap with {draft.state.base?.siblings.find((n) => n.node_id === draft.state.neighborId)
          ?.name}. Durations and the gap stay the same; children move with their clip.
      </p>
      {#each preview.siblings.filter((n) => n.node_id === node.node_id || n.node_id === draft.state.neighborId) as clip (clip.node_id)}<p
        >
          {clip.name}: {clip.start_ms}–{clip.end_ms} ms
        </p>{/each}
    {:else}<p>
        These siblings overlap or their placement and order disagree. Read and adjust their
        placement before reordering.
      </p>{/if}
  {/if}
</fieldset>
<div class="actions">
  <button
    type="button"
    onclick={() => void draft.apply()}
    disabled={draft.state.busy ||
      !draft.state.base ||
      (!draft.state.uncertain && (!preview || changed))}
    >{draft.state.busy
      ? 'Applying…'
      : draft.state.uncertain
        ? 'Retry exact reorder'
        : 'Apply reorder'}</button
  >
  <button
    type="button"
    onclick={() => void draft.reload()}
    disabled={draft.state.busy || draft.state.uncertain}
    >{draft.state.neighborId
      ? 'Discard reorder draft and read saved siblings'
      : 'Read saved sibling order'}</button
  >
</div>
{#if changed}<p role="alert">
    Saved siblings changed since this draft was read. Your reorder draft is kept.
  </p>{/if}
{#if draft.state.saved}<p role="status">
    Sibling reorder saved. Affected generated screenplay can be reviewed before replacement.
  </p>{/if}
{#if draft.state.uncertain}<p role="status">
    Reorder completion is uncertain. Retry the original pair before discarding this draft.
  </p>{/if}
{#if draft.state.error}<p role="alert">{draft.state.error}</p>{/if}

<style>
  fieldset {
    border: 1px solid var(--color-border-default);
    border-radius: 6px;
    margin: 0.5rem 0;
    padding: 0.5rem;
  }
  legend,
  p {
    font-size: 0.8rem;
  }
  button {
    margin-right: 0.4rem;
  }
  .actions {
    display: flex;
    gap: 0.35rem;
    flex-wrap: wrap;
  }
</style>
