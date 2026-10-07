<script lang="ts">
  import type { StoryArcFieldInput } from '$lib/storyArcTypes.js';

  let {
    previous,
    current,
    absent,
  }: {
    previous?: StoryArcFieldInput[] | null;
    current?: StoryArcFieldInput[] | null;
    absent?: [string, string][] | null;
  } = $props();

  let changes = $derived(
    (previous ?? []).flatMap((before) => {
      const after = current?.find(
        (input) => input.arc_id === before.arc_id && input.field === before.field,
      );
      return !after ||
        before.value !== after.value ||
        before.revision_event_id !== after.revision_event_id
        ? [{ before, after }]
        : [];
    }),
  );
</script>

{#if previous?.length || current?.length || absent?.length}
  <section aria-label="Recorded screenplay arc evidence">
    {#each changes as { before, after } (`${before.arc_id}.${before.field}`)}
      <p aria-label="Arc input change">
        <strong>Arc {before.field.replaceAll('_', ' ')}:</strong>
        {before.value} → {after ? after.value || '(cleared)' : '(removed)'}
      </p>
    {/each}
    <details>
      <summary>Arc context used for this preview</summary>
      {#if previous == null}<p>Original arc consumption is unknown.</p>{/if}
      {#each [{ label: 'Originally consumed', inputs: previous }, { label: 'Current preview input', inputs: current }] as group (group.label)}
        {#if group.inputs?.length}
          <strong>{group.label}</strong>
          <ul>
            {#each group.inputs as input (`${input.arc_id}.${input.field}`)}
              <li>
                {input.arc_id}
                {input.field.replaceAll('_', ' ')}: {input.value || '(cleared)'}
                <small>Revision {input.revision_event_id ?? 'unknown; unbound history'}</small>
              </li>
            {/each}
          </ul>
        {/if}
      {/each}
      {#each absent ?? [] as [arcId, eventId] (arcId)}
        <p>Removed arc {arcId} <small>Deletion revision {eventId}</small></p>
      {/each}
    </details>
  </section>
{/if}

<style>
  section {
    overflow-wrap: anywhere;
  }
  p {
    white-space: pre-wrap;
  }
  small {
    display: block;
  }
</style>
