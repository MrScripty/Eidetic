<script lang="ts">
  import type { TimelineTitleInput } from '$lib/propagationProposalTypes.js';
  let {
    previous,
    current,
    absent,
  }: {
    previous?: TimelineTitleInput[] | null;
    current?: TimelineTitleInput[] | null;
    absent?: [string, string][] | null;
  } = $props();
  let changed = $derived(
    (previous ?? []).filter((original) => {
      const live = current?.find((input) => input.node_id === original.node_id);
      return (
        !live ||
        live.name !== original.name ||
        live.revision_event_id !== original.revision_event_id
      );
    }),
  );
</script>

{#if previous?.length}
  <section aria-label="Recorded timeline title evidence">
    <details open>
      <summary>Timeline titles used for this update</summary>
      {#each changed as original (original.node_id)}
        {@const live = current?.find((input) => input.node_id === original.node_id)}
        {@const deletion = absent?.find(([node]) => node === original.node_id)}
        <strong>Originally consumed title</strong>
        <pre aria-label="Originally consumed title">{original.name}</pre>
        <small
          >Clip {original.node_id} · {original.revision_event_id
            ? `Title revision ${original.revision_event_id}`
            : 'Known baseline; no recorded title revision'}</small
        >
        {#if live}
          <strong>Current preview title</strong>
          <pre aria-label="Current preview title">{live.name}</pre>
          <small
            >{live.revision_event_id
              ? `Title revision ${live.revision_event_id}`
              : 'Known baseline; no recorded title revision'}</small
          >
        {:else if deletion}
          <p>Title source removed · Revision {deletion[1]}</p>
        {/if}
      {/each}
      {#if previous.length > changed.length}<p>
          {previous.length - changed.length} other supplied titles are unchanged.
        </p>{/if}
      <p>
        Saving a title and previewing preserve saved screenplay. Accept update replaces only the
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
