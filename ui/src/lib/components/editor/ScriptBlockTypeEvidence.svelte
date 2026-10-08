<script lang="ts">
  import type { ScriptContextBlock } from '$lib/scriptTypes.js';
  let {
    previous,
    current,
  }: { previous?: ScriptContextBlock[] | null; current: ScriptContextBlock[] } = $props();
  let changes = $derived(
    (previous ?? []).filter(
      (input) =>
        input.block_kind &&
        current.some(
          (now) =>
            now.block_id === input.block_id &&
            now.block_kind &&
            now.block_kind !== input.block_kind,
        ),
    ),
  );
</script>

{#each changes as original (original.block_id)}
  {@const now = current.find((input) => input.block_id === original.block_id)!}
  <section aria-label="Consumed screenplay type changed">
    <p>Consumed screenplay type changed.</p>
    <p>Originally consumed type:</p>
    <pre aria-label="Originally consumed block type">{original.block_kind?.replaceAll(
        '_',
        ' ',
      )}</pre>
    <pre aria-label="Originally consumed typed text">{original.text}</pre>
    <p>Current type:</p>
    <pre aria-label="Current screenplay block type">{now.block_kind?.replaceAll('_', ' ')}</pre>
    <pre aria-label="Current typed screenplay text">{now.text}</pre>
  </section>
{/each}

<style>
  section {
    margin: 0.5rem 0;
    border: 1px solid var(--color-border-subtle);
    padding: 0.5rem;
  }
  pre {
    white-space: pre-wrap;
    overflow-wrap: anywhere;
  }
</style>
