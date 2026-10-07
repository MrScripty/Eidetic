<script lang="ts">
  import type { TimelineNotesInput } from '$lib/propagationProposalTypes.js';

  let {
    previous,
    current,
  }: {
    previous?: TimelineNotesInput | null;
    current?: TimelineNotesInput | null;
  } = $props();
</script>

{#if previous || current}
  <section aria-label="Recorded timeline Notes evidence">
    <details open>
      <summary>Timeline Notes used for this update</summary>
      {#if !previous}<p>Original Notes consumption is unknown.</p>{/if}
      {#each [{ label: 'Originally consumed Notes', input: previous }, { label: 'Current preview Notes', input: current }] as group (group.label)}
        {#if group.input}
          <strong>{group.label}</strong>
          <pre aria-label={group.label}>{group.input.notes || '(cleared)'}</pre>
          <small
            >Clip {group.input.node_id} · Revision {group.input.revision_event_id ??
              'unknown; unbound history'}</small
          >
        {/if}
      {/each}
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
