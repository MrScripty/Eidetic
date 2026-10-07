<script lang="ts">
  import type { BibleGraphEdgeKind } from '$lib/bibleGraphTypes.js';
  import type { FieldValue } from '$lib/projectionTypes.js';
  import {
    bibleRecallState,
    invalidateBibleRecall,
    recallBibleFacts,
    selectBibleRecallAnchor,
  } from '$lib/stores/bibleRecallProjection.svelte.js';

  let { nodeId }: { nodeId: string } = $props();
  let time = $state('');
  let direction = $state<'incoming' | 'outgoing' | 'both'>('both');
  let kind = $state('');
  let inputError = $state('');
  const projection = $derived(
    bibleRecallState.anchor === nodeId ? bibleRecallState.projection : null,
  );
  const kinds: string[] = [
    'references',
    'located_in',
    'owns',
    'member_of',
    'conflicts_with',
    'supports_theme',
  ];

  $effect(() => {
    selectBibleRecallAnchor(nodeId);
    return () => invalidateBibleRecall();
  });

  function changed(): void {
    inputError = '';
    invalidateBibleRecall();
  }
  async function recall(): Promise<void> {
    const at = time.trim() === '' ? null : Number(time);
    if (at !== null && (!Number.isSafeInteger(at) || at < 0)) {
      inputError = 'Story time must be a nonnegative whole number of milliseconds.';
      return;
    }
    inputError = '';
    await recallBibleFacts({
      anchor_node_id: nodeId,
      story_time_ms: at,
      direction,
      edge_kinds: kind ? [kind as BibleGraphEdgeKind] : [],
      neighbor_limit: 8,
    });
  }
  function valueText(value: FieldValue): string {
    return value.type === 'object_ref' ? value.value.id : String(value.value);
  }
  function name(id: string): string {
    return projection?.payload.nodes.find((node) => node.node_id === id)?.name ?? id;
  }
  function edgeKind(value: BibleGraphEdgeKind): string {
    return typeof value === 'string' ? value.replaceAll('_', ' ') : value.custom;
  }
</script>

<section class="recall" aria-label="Related story facts">
  <details>
    <summary>Related story facts</summary>
    <p class="muted">
      Inspect connected facts. Recall leaves screenplay and generation context unchanged.
    </p>
    <div class="controls">
      <label
        >Story time (ms)
        <input
          aria-label="Recall story time (ms)"
          type="text"
          inputmode="numeric"
          placeholder="Unspecified"
          bind:value={time}
          oninput={changed}
        />
      </label>
      <label
        >Direction
        <select aria-label="Recall direction" bind:value={direction} onchange={changed}>
          <option value="both">Both</option><option value="incoming">Incoming</option><option
            value="outgoing">Outgoing</option
          >
        </select>
      </label>
      <label
        >Relationship kind
        <select aria-label="Recall relationship kind" bind:value={kind} onchange={changed}>
          <option value="">All kinds</option>
          {#each kinds as option}<option value={option}>{option.replaceAll('_', ' ')}</option
            >{/each}
        </select>
      </label>
    </div>
    <button type="button" onclick={recall} disabled={bibleRecallState.pending}
      >Recall related story facts</button
    >
    {#if bibleRecallState.pending}<p role="status">Recalling…</p>{/if}
    {#if inputError || bibleRecallState.error}<p class="error" role="alert">
        {inputError || bibleRecallState.error}
      </p>{/if}
    {#if bibleRecallState.invalidated}<p role="status">
        Facts changed. Recall again to inspect current evidence.
      </p>{/if}
    {#if projection}
      <p class="muted">
        One hop · {projection.payload.request.story_time_ms === null
          ? 'story time unspecified'
          : `story time ${projection.payload.request.story_time_ms} ms`} · revision {projection.version}
      </p>
      <p class="muted">
        Relationships are untimed associations. Connectedness does not establish truth or validity
        at this story time.
      </p>
      {#each projection.payload.nodes as node, index (node.node_id)}
        <article>
          <h4>{node.name} <span class="muted">{index === 0 ? '(selected)' : '(related)'}</span></h4>
          {#each projection.payload.paths.filter((path) => path.neighbor_node_id === node.node_id) as path (path.relationship.edge.edge_id)}
            {@const edge = path.relationship.edge}
            <p class="path">
              {name(edge.from_node_id)}
              {edge.directed ? '→' : '↔'}
              {name(edge.to_node_id)} · {edgeKind(edge.edge_kind)}{edge.label
                ? ` · ${edge.label}`
                : ''}
            </p>
            <details class="sources">
              <summary>Relationship source</summary>
              <p>{edge.edge_id} · revision {path.relationship.revision_event_id} · untimed</p>
            </details>
          {/each}
          {#each node.fields as field (`${field.part_key}.${field.field_key}`)}
            <p class="fact">
              <strong>{field.part_key}.{field.field_key}</strong>: {valueText(field.value)}
            </p>
            <details class="sources">
              <summary>Fact source</summary>
              {#if field.source.kind === 'baseline'}
                <p>Baseline {field.source.field_id} · revision {field.source.revision_event_id}</p>
              {:else}
                <p>
                  Assertion “{field.source.label}” at {field.source.at_ms} ms · snapshot {field
                    .source.snapshot_id} · revision {field.source.snapshot_revision_event_id}
                </p>
                <p>
                  Field {field.source.snapshot_field_id} · revision {field.source
                    .field_revision_event_id}
                </p>
              {/if}
            </details>
          {/each}
          {#each node.unresolved_timed_fields as field}
            <p class="muted">
              Unresolved: {field.part_key}.{field.field_key} — no established value at the requested story
              time.
            </p>
          {/each}
          {#if node.omitted_fields}<p class="muted">
              {node.omitted_fields} whole field records omitted by the 32 KiB evidence limit.
            </p>{/if}
          <details class="sources">
            <summary>Entity source</summary>
            <p>
              {node.node_id} · {node.schema_key} · name revision {node.name_revision_event_id ??
                'unknown'}
            </p>
          </details>
        </article>
      {/each}
      {#if projection.payload.nodes.length === 1}<p>No matching live neighbors.</p>{/if}
      <p class="muted">
        Omitted: {projection.payload.omitted_neighbors} matching neighbors · {projection.payload
          .omitted_edges} matching relationships. Limits: 8 neighbors, 32 relationships, 32 KiB.
      </p>
    {/if}
  </details>
</section>

<style>
  .recall {
    border-top: 1px solid var(--border-color, #333);
    padding: 0.65rem 0;
    font-size: 0.78rem;
  }
  summary {
    cursor: pointer;
  }
  .controls {
    display: grid;
    gap: 0.4rem;
    margin: 0.5rem 0;
  }
  label {
    display: grid;
    gap: 0.15rem;
  }
  input,
  select {
    width: 100%;
    background: var(--bg-tertiary, #222);
    color: inherit;
    border: 1px solid var(--border-color, #444);
    border-radius: 3px;
    padding: 0.3rem;
  }
  button {
    border: 1px solid var(--border-color, #444);
    border-radius: 3px;
    padding: 0.4rem;
    cursor: pointer;
  }
  article {
    border-top: 1px solid var(--border-color, #333);
    margin-top: 0.6rem;
    padding-top: 0.4rem;
  }
  h4 {
    margin: 0.3rem 0;
  }
  p {
    margin: 0.35rem 0;
    overflow-wrap: anywhere;
    white-space: pre-wrap;
  }
  .muted,
  .sources {
    color: var(--text-secondary, #999);
  }
  .sources {
    font-size: 0.7rem;
  }
  .error {
    color: var(--error-color, #ef7777);
  }
</style>
