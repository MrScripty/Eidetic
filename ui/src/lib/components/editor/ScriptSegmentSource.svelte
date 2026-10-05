<script lang="ts">
  import { formatTime } from '$lib/timelineTypes.js';
  import { selectedNodeEditorProjectionState } from '$lib/stores/selectedNodeEditorProjection.svelte.js';
  import { getScriptSourceClip, showScriptSource } from './scriptSourceNavigation.js';
  let { sourceNodeId }: { sourceNodeId?: string | null } = $props();
  let clip = $derived(getScriptSourceClip(sourceNodeId));
  let ownsRead = $derived(selectedNodeEditorProjectionState.selectedNodeId === sourceNodeId);
</script>

<section class="source" aria-label="Screenplay source clip">
  {#if clip}
    <strong>{clip.name}</strong>
    <span>{formatTime(clip.start_ms)} – {formatTime(clip.end_ms)}</span>
    <button
      type="button"
      disabled={ownsRead && selectedNodeEditorProjectionState.pending}
      onclick={() => void showScriptSource(sourceNodeId).catch(() => {})}>Go to clip</button
    >
    {#if ownsRead && selectedNodeEditorProjectionState.error}<p role="alert">
        {selectedNodeEditorProjectionState.error}
      </p>{/if}
  {:else if sourceNodeId}
    <span>Source clip unavailable.</span>
  {:else}
    <span>Standalone screenplay.</span>
  {/if}
</section>

<style>
  .source {
    break-inside: avoid;
    display: flex;
    flex-wrap: wrap;
    align-items: center;
    gap: 8px;
    padding: 6px 0;
    font-size: 0.8rem;
  }
  p {
    width: 100%;
    margin: 0;
  }
</style>
