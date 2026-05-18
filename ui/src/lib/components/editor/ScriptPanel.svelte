<script lang="ts">
  import ScriptView from './ScriptView.svelte';
  import { scriptDocumentBlockCount, scriptDocumentText } from '$lib/scriptDocumentFormat.js';
  import {
    getCachedScriptDocumentProjection,
    getScriptDocumentProjectionError,
    isScriptDocumentProjectionPending,
    MAIN_SCRIPT_DOCUMENT_ID,
    refreshScriptDocumentProjection,
  } from '$lib/stores/scriptDocumentProjection.svelte.js';

  const scriptDocumentKey = { document_id: MAIN_SCRIPT_DOCUMENT_ID };
  let projection = $derived(getCachedScriptDocumentProjection(scriptDocumentKey));
  let pending = $derived(isScriptDocumentProjectionPending(scriptDocumentKey));
  let error = $derived(getScriptDocumentProjectionError(scriptDocumentKey));
  let text = $derived(projection ? scriptDocumentText(projection.payload) : '');
  let blockCount = $derived(projection ? scriptDocumentBlockCount(projection.payload) : 0);

  $effect(() => {
    if (!projection && !pending && !error) {
      void refreshScriptDocumentProjection(scriptDocumentKey).catch(() => {});
    }
  });
</script>

<div class="script-panel">
  <div class="script-panel-header">
    <span class="script-panel-title">Script</span>
    <span class="script-panel-count">{blockCount} blocks</span>
  </div>

  <div class="script-panel-body">
    {#if text}
      <ScriptView {text} />
    {:else if pending}
      <p class="script-empty">Loading script document.</p>
    {:else if error}
      <p class="script-empty">No script document yet.</p>
    {:else}
      <p class="script-empty">No script document yet.</p>
    {/if}
  </div>
</div>

<style>
  .script-panel {
    display: flex;
    flex-direction: column;
    height: 100%;
    overflow: hidden;
    background: var(--color-bg-secondary);
  }

  .script-panel-header {
    display: flex;
    align-items: center;
    justify-content: space-between;
    padding: 4px 12px;
    border-bottom: 1px solid var(--color-border-subtle);
    flex-shrink: 0;
  }

  .script-panel-title {
    font-size: 0.8rem;
    font-weight: 600;
    color: var(--color-text-primary);
  }

  .script-panel-count {
    font-size: 0.7rem;
    color: var(--color-text-muted);
  }

  .script-panel-body {
    flex: 1;
    overflow: auto;
    padding: 8px 12px;
    column-width: 40ch;
    column-gap: 24px;
    column-rule: 1px solid var(--color-border-subtle);
    column-fill: auto;
  }

  .script-empty {
    color: var(--color-text-muted);
    font-size: 0.8rem;
    text-align: center;
    padding: 24px 0;
    margin: 0;
  }
</style>
