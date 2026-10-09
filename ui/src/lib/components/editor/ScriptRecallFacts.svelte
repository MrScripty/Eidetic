<script lang="ts">
  import { untrack } from 'svelte';
  import { bibleRecallState } from '$lib/stores/bibleRecallProjection.svelte.js';
  import { getEditorSessionGeneration } from '$lib/stores/editor.svelte.js';
  import { createScriptRecallDraft } from './scriptRecallDraft.svelte.js';
  let { scope, disabled = false }: { scope: string; disabled?: boolean } = $props();
  const draft = createScriptRecallDraft();
  const packet = $derived(bibleRecallState.projection?.payload ?? null);
  const anchorNameKnown = $derived(
    !!packet?.nodes.find((node) => node.node_id === packet.request.anchor_node_id)
      ?.name_revision_event_id,
  );
  $effect(() => {
    const current = packet;
    const owner = JSON.stringify([scope, getEditorSessionGeneration()]);
    untrack(() => draft.observe(owner, current));
  });
  export function getSelection() {
    return draft.request(JSON.stringify([scope, getEditorSessionGeneration()]), packet);
  }
</script>

<details>
  <summary>Use recalled facts for this preview ({draft.state.fieldIds.length}/8 selected)</summary>
  <p>
    Choose supplemental baseline facts from the Bible recall. This selection applies only to a new
    preview of this block. Existing scene context remains in use.
  </p>
  {#if !packet}
    <p>
      Recall related story facts in Bible at unspecified story time first. Changed or closed recall
      clears this selection.
    </p>
  {:else}
    {#if packet.request.story_time_ms !== null}
      <p role="status">
        Timed recall cannot be used for this preview. Recall again with story time unspecified.
        Snapshot-backed facts are inspection only.
      </p>
    {/if}
    {#each packet.nodes as node (node.node_id)}
      {#each node.fields as field (`${field.part_key}.${field.field_key}`)}
        {@const id =
          field.source.kind === 'baseline' ? field.source.field_id : field.source.snapshot_field_id}
        <label>
          <input
            type="checkbox"
            checked={draft.state.fieldIds.includes(id)}
            disabled={disabled ||
              packet.request.story_time_ms !== null ||
              field.source.kind !== 'baseline' ||
              !node.name_revision_event_id ||
              (node.node_id !== packet.request.anchor_node_id && !anchorNameKnown) ||
              (draft.state.fieldIds.length >= 8 && !draft.state.fieldIds.includes(id))}
            onchange={(event) => {
              draft.select(
                JSON.stringify([scope, getEditorSessionGeneration()]),
                packet,
                id,
                event.currentTarget.checked,
              );
              event.currentTarget.checked = draft.state.fieldIds.includes(id);
            }}
          />
          {node.name} · {field.part_key}.{field.field_key}: {JSON.stringify(field.value.value)}
          {#if field.source.kind === 'snapshot'}
            — Snapshot-backed; inspection only.{/if}
          {#if !node.name_revision_event_id}
            — Name source unknown; cannot select.{/if}
          {#if node.node_id !== packet.request.anchor_node_id && !anchorNameKnown}
            — Connecting anchor name source unknown; cannot select.
          {/if}
        </label>
      {/each}
      {#each node.unresolved_timed_fields as field}
        <p>{node.name} · {field.part_key}.{field.field_key} — Unresolved; cannot select.</p>
      {/each}
      {#if node.omitted_fields}<p>
          {node.omitted_fields} omitted facts for {node.name}; cannot select omitted records.
        </p>{/if}
    {/each}
    {#if packet.omitted_neighbors || packet.omitted_edges}<p>
        Omitted neighbors and paths cannot be selected.
      </p>{/if}
    <p>
      Connecting paths and their endpoint names accompany selected related facts as untimed
      associations; they do not establish fictional-time validity.
    </p>
  {/if}
  {#if draft.state.error}<p role="alert">{draft.state.error}</p>{/if}
</details>

<style>
  label {
    display: block;
    margin: 6px 0;
  }
</style>
