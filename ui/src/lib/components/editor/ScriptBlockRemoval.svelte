<script lang="ts">
  import type { createScriptBlockEditDraft } from './scriptBlockEditDraft.svelte.js';
  let { editor }: { editor: ReturnType<typeof createScriptBlockEditDraft> } = $props();
  const removal = $derived(editor.state.removal);
</script>

<section aria-label="Remove saved screenplay block">
  <strong>Remove this saved block?</strong>
  <pre>{removal.text}</pre>
  <p>Its history remains saved. Scenes that used it may need review.</p>
  {#if removal.error}<p role="alert">{removal.error}</p>{/if}
  {#if removal.uncertain}
    <p>The removal may have completed. Retry the same removal to confirm it.</p>
  {/if}
  <button type="button" onclick={editor.remove} disabled={removal.saving}
    >{removal.saving
      ? 'Removing…'
      : removal.uncertain
        ? 'Retry same removal'
        : 'Remove saved block'}</button
  >
  <button
    type="button"
    onclick={editor.cancelRemoval}
    disabled={removal.saving || removal.uncertain}>Keep block</button
  >
</section>

<style>
  section {
    break-inside: avoid;
    padding: 8px;
    border: 1px solid var(--color-border-subtle);
  }
  pre {
    white-space: pre-wrap;
    overflow-wrap: anywhere;
  }
  p {
    font-size: 0.8rem;
  }
</style>
