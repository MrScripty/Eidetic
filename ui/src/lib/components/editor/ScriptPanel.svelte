<script lang="ts">
  import ScriptBlockEditor from './ScriptBlockEditor.svelte';
  import ScriptBlockRemoval from './ScriptBlockRemoval.svelte';
  import { getOrphanedScriptBlockRemovals } from '$lib/stores/scriptBlockEditSession.svelte.js';
  import ScriptSegmentSource from './ScriptSegmentSource.svelte';
  import ScriptBlockComposer from './ScriptBlockComposer.svelte';
  import { getScriptCreationSource, refreshRetimedScriptSource } from './scriptCreationSource.js';
  import { editorState } from '$lib/stores/editor.svelte.js';
  import { timelineRenderProjectionState } from '$lib/stores/timelineRenderProjection.svelte.js';
  import {
    selectedNodeEditorProjectionState,
    refreshSelectedNodeEditorProjection,
  } from '$lib/stores/selectedNodeEditorProjection.svelte.js';
  import ScriptImpactNotice from './ScriptImpactNotice.svelte';
  import ScriptImpactReview from './ScriptImpactReview.svelte';
  import ScriptFactReconciliation from './ScriptFactReconciliation.svelte';
  import { scriptDocumentBlockCount } from '$lib/scriptDocumentFormat.js';
  import {
    getCachedScriptDocumentProjection,
    getScriptDocumentProjectionError,
    isScriptDocumentProjectionPending,
    MAIN_SCRIPT_DOCUMENT_ID,
    refreshScriptDocumentProjection,
  } from '$lib/stores/scriptDocumentProjection.svelte.js';

  const source = $derived(getScriptCreationSource());
  const selectedNodePlacementIsStale = $derived.by(() => {
    const node = selectedNodeEditorProjectionState.projection?.payload.node;
    const clip = timelineRenderProjectionState.projection?.payload.clips.find(
      (item) => item.node_id === editorState.selectedNodeId,
    );
    if (!node || node.node_id !== editorState.selectedNodeId || !clip) return false;
    return clip.start_ms !== node.start_ms || clip.end_ms !== node.end_ms;
  });
  $effect(() => {
    void refreshRetimedScriptSource().catch(() => {});
  });
  $effect(() => {
    const nodeId = editorState.selectedNodeId;
    if (
      nodeId &&
      !selectedNodeEditorProjectionState.pending &&
      (selectedNodeEditorProjectionState.selectedNodeId !== nodeId ||
        (!selectedNodeEditorProjectionState.projection && !selectedNodeEditorProjectionState.error))
    ) {
      void refreshSelectedNodeEditorProjection(nodeId).catch(() => {});
    }
  });

  const scriptDocumentKey = { document_id: MAIN_SCRIPT_DOCUMENT_ID };
  let projection = $derived(getCachedScriptDocumentProjection(scriptDocumentKey));
  let pending = $derived(isScriptDocumentProjectionPending(scriptDocumentKey));
  let error = $derived(getScriptDocumentProjectionError(scriptDocumentKey));
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
    {#each getOrphanedScriptBlockRemovals(MAIN_SCRIPT_DOCUMENT_ID, projection?.payload.segments.flatMap( (segment) => segment.blocks.map((block) => block.block.id), ) ?? []) as editor}
      <ScriptBlockRemoval {editor} />
    {/each}
    <ScriptBlockComposer {source} />
    {#if !source && (selectedNodeEditorProjectionState.error || selectedNodePlacementIsStale) && selectedNodeEditorProjectionState.selectedNodeId === editorState.selectedNodeId}
      {#if selectedNodeEditorProjectionState.error}
        <p role="alert">{selectedNodeEditorProjectionState.error}</p>
      {/if}
      <button
        type="button"
        disabled={selectedNodeEditorProjectionState.pending}
        onclick={() =>
          editorState.selectedNodeId &&
          void refreshSelectedNodeEditorProjection(editorState.selectedNodeId).catch(() => {})}
        >Refresh selected clip</button
      >
    {/if}
    {#if blockCount > 0}
      {#each projection?.payload.segments ?? [] as segment (segment.segment.id)}
        <ScriptSegmentSource sourceNodeId={segment.segment.source_node_id} />
        {#if segment.impact}<ScriptImpactNotice impact={segment.impact} />{/if}
        <ScriptImpactReview documentId={MAIN_SCRIPT_DOCUMENT_ID} {segment} />
        <ScriptFactReconciliation documentId={MAIN_SCRIPT_DOCUMENT_ID} {segment} />
        {#each segment.blocks as block (block.block.id)}
          <ScriptBlockEditor documentId={MAIN_SCRIPT_DOCUMENT_ID} {block} />
        {/each}
      {/each}
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
