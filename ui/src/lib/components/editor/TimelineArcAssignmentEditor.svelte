<script lang="ts">
  import { untrack } from 'svelte';
  import type { SelectedNodeEditorNode } from '$lib/selectedNodeEditorTypes.js';
  import { storyArcProjectionState } from '$lib/stores/storyArcProjection.svelte.js';
  import { getSessionTimelineArcsDraft } from '$lib/stores/timelineArcsSession.svelte.js';
  import { sameArcIds } from './timelineArcsDraft.svelte.js';
  let { node }: { node: SelectedNodeEditorNode } = $props();
  const draft = $derived(getSessionTimelineArcsDraft(node));
  const arcs = $derived(storyArcProjectionState.projection?.payload.arcs ?? []);
  const unavailable = $derived(
    draft.state.arcIds.filter((id) => !arcs.some((arc) => arc.id === id)),
  );
  $effect(() => {
    const owner = draft;
    const read = node.arc_read;
    untrack(() => owner.observe(read));
  });
  const dirty = $derived(
    !!draft.state.base && !sameArcIds(draft.state.arcIds, draft.state.base.arc_ids),
  );
  const changed = $derived(
    !!draft.state.base &&
      !!draft.state.current &&
      (!sameArcIds(draft.state.base.arc_ids, draft.state.current.arc_ids) ||
        draft.state.base.revision_event_id !== draft.state.current.revision_event_id),
  );
  function toggle(id: string, checked: boolean) {
    draft.state.arcIds = checked
      ? [...draft.state.arcIds, id]
      : draft.state.arcIds.filter((value) => value !== id);
    draft.state.saved = false;
  }
</script>

<fieldset disabled={node.locked || !draft.state.base || draft.state.busy || draft.state.uncertain}>
  <legend>Story arcs</legend>
  {#each arcs as arc (arc.id)}
    <label
      ><input
        type="checkbox"
        checked={draft.state.arcIds.includes(arc.id)}
        onchange={(event) => toggle(arc.id, event.currentTarget.checked)}
      />{arc.name}</label
    >
  {/each}
  {#each unavailable as id (id)}
    <label
      ><input
        type="checkbox"
        checked
        onchange={(event) => toggle(id, event.currentTarget.checked)}
      />Unavailable arc · {id}</label
    >
  {/each}
  {#if arcs.length === 0 && unavailable.length === 0}<p>
      Create a story arc in the Arcs panel to assign it here.
    </p>{/if}
</fieldset>
<div class="arc-actions">
  <button
    type="button"
    onclick={() => void draft.apply()}
    disabled={(node.locked && !draft.state.uncertain) ||
      draft.state.busy ||
      !draft.state.base ||
      (!draft.state.uncertain && (!dirty || changed))}
    >{draft.state.busy
      ? 'Saving…'
      : draft.state.uncertain
        ? 'Retry exact arc save'
        : 'Save arc assignment'}</button
  >
  <button
    type="button"
    onclick={() => void draft.reload()}
    disabled={draft.state.busy || draft.state.uncertain}
    >{dirty ? 'Discard arc draft and read saved assignment' : 'Read saved arc assignment'}</button
  >
</div>
{#if dirty}<p role="status">
    Unsaved arc assignment. Kept while you switch clips or panels in this project session.
  </p>{/if}
{#if draft.state.saved}<p role="status">
    Arc assignment saved. Affected generated screenplay can be reviewed before replacement.
  </p>{/if}
{#if changed}<p role="alert">
    Saved arc assignment changed since this draft was read. Your draft is kept.
  </p>
  <details>
    <summary>Current saved arc assignment</summary>
    <p>
      {draft.state.current?.arc_ids
        .map((id) => arcs.find((arc) => arc.id === id)?.name ?? id)
        .join(', ') || '(no arcs)'}
    </p>
  </details>{/if}
{#if draft.state.uncertain}<p role="status">
    Save completion is uncertain. Retry the original arc save before discarding this draft.
  </p>{/if}
{#if draft.state.error}<p role="alert">{draft.state.error}</p>{/if}

<style>
  fieldset {
    border: 1px solid var(--color-border-default);
    border-radius: 6px;
    margin: 0.5rem 0;
    padding: 0.5rem;
  }
  legend {
    font-size: 0.8rem;
    color: var(--color-text-secondary);
  }
  label {
    display: inline-flex;
    gap: 0.35rem;
    align-items: center;
    margin-right: 0.75rem;
    font-size: 0.8rem;
  }
  .arc-actions {
    display: flex;
    flex-wrap: wrap;
    gap: 0.5rem;
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
  p {
    font-size: 0.8rem;
    overflow-wrap: anywhere;
  }
  [role='alert'] {
    color: var(--color-danger);
  }
</style>
