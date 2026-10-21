<script lang="ts">
  import type { TimelineArcMembershipInput } from '$lib/timelineCommandTypes.js';
  import type { StoryArcFieldInput } from '$lib/storyArcTypes.js';
  let {
    previous,
    current,
    previousFields,
    currentFields,
  }: {
    previous?: TimelineArcMembershipInput | null;
    current?: TimelineArcMembershipInput | null;
    previousFields?: StoryArcFieldInput[] | null;
    currentFields?: StoryArcFieldInput[] | null;
  } = $props();
</script>

{#if previous || current}
  <details open aria-label="Recorded story arc assignment evidence">
    <summary>Story arc assignment used for this update</summary>
    {#if !previous}<p>Original arc assignment consumption is unknown.</p>{/if}
    {#each [{ label: 'Originally consumed arc assignment', input: previous, fields: previousFields }, { label: 'Current preview arc assignment', input: current, fields: currentFields }] as group (group.label)}
      {#if group.input}
        <strong>{group.label}</strong>
        <ul>
          {#each group.input.arc_ids as id (id)}<li>
              {group.fields?.find((field) => field.arc_id === id && field.field === 'name')
                ?.value ?? 'Arc'} · {id}
            </li>{/each}
        </ul>
        {#if group.input.arc_ids.length === 0}<p>(no arcs)</p>{/if}
        <small
          >Clip {group.input.node_id} · Revision {group.input.revision_event_id ??
            'known baseline; no assignment event'}</small
        >
      {/if}
    {/each}
    <p>
      Saving arc assignment and previewing preserve saved screenplay. Accept update replaces only
      the selected block.
    </p>
  </details>
{/if}

<style>
  details {
    overflow-wrap: anywhere;
  }
  small {
    display: block;
  }
</style>
