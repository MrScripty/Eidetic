<script lang="ts">
  import type { TimelineNotesInput } from '$lib/propagationProposalTypes.js';

  let {
    previous,
    current,
    ancestor = false,
    absentRevision,
  }: {
    previous?: TimelineNotesInput | null;
    current?: TimelineNotesInput | null;
    ancestor?: boolean;
    absentRevision?: string;
  } = $props();
</script>

{#if previous || current}
  <section
    aria-label={ancestor ? 'Recorded ancestor Notes evidence' : 'Recorded timeline Notes evidence'}
  >
    <details open>
      <summary
        >{ancestor
          ? 'Ancestor Notes used for this update'
          : 'Timeline Notes used for this update'}</summary
      >
      {#if !previous}<p>Original Notes consumption is unknown.</p>{/if}
      {#each [{ label: 'Originally consumed Notes', input: previous }, { label: 'Current preview Notes', input: current }] as group (group.label)}
        {#if group.input}
          <strong>{group.label}</strong>
          <pre aria-label={group.label}>{group.input.notes || '(cleared)'}</pre>
          <small
            >{ancestor ? 'Ancestor' : 'Clip'}
            {group.input.node_id} · Revision {group.input.revision_event_id ??
              'unknown; unbound history'}</small
          >
        {/if}
      {/each}
      {#if ancestor && absentRevision}<p>Ancestor removed · Revision {absentRevision}</p>{/if}
      <p>
        Saving Notes and previewing preserve saved screenplay. Accept update replaces only the
        selected block.
      </p>
    </details>
  </section>
{/if}

<style>
  section {
    overflow-wrap: anywhere;
  }
  pre {
    white-space: pre-wrap;
  }
  small {
    display: block;
  }
</style>
