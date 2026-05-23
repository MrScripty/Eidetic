<script lang="ts">
  import { onMount } from 'svelte';
  import {
    bibleState,
    selectBibleGraphContextLayer,
    selectBibleGraphEdge,
    selectBibleGraphInfluence,
    selectBibleGraphNeighborhood,
    selectBibleGraphNode,
    selectedBibleGraphNodeId,
  } from '$lib/stores/bible.svelte.js';
  import { editorState } from '$lib/stores/editor.svelte.js';
  import { getCachedBibleRenderGraphProjection } from '$lib/stores/bibleRenderGraphProjection.svelte.js';
  import { startGraphRendererCommandDrain } from '$lib/stores/graphRendererCommandDrain.js';
  import {
    clearContextStackProjection,
    getCachedContextStackProjection,
    refreshContextStackProjection,
  } from '$lib/stores/contextStackProjection.svelte.js';
  import BibleRenderGraphOutline from '../sidebar/bible/BibleRenderGraphOutline.svelte';
  import GraphWorkspaceSideLists from './GraphWorkspaceSideLists.svelte';
  import {
    graphWorkspaceEdgeItems,
    graphWorkspaceNeighborhoodItems,
  } from './graphWorkspaceItems.js';
  import BevyViewportPanel from './BevyViewportPanel.svelte';

  const renderGraphProjection = $derived(getCachedBibleRenderGraphProjection());
  const graph = $derived(renderGraphProjection?.payload ?? null);
  const contextStackProjection = $derived(getCachedContextStackProjection());
  const contextLayers = $derived(contextStackProjection?.payload.layers ?? []);
  const selectedGraphNodeId = $derived(selectedBibleGraphNodeId());
  const graphSelection = $derived(bibleState.graphSelection);
  const edgeItems = $derived(graph ? graphWorkspaceEdgeItems(graph) : []);
  const neighborhoodItems = $derived(graph ? graphWorkspaceNeighborhoodItems(graph) : []);
  const hasSideLists = $derived(
    graph
      ? contextLayers.length > 0 ||
          graph.influences.length > 0 ||
          edgeItems.length > 0 ||
          neighborhoodItems.length > 0
      : false,
  );

  onMount(() => startGraphRendererCommandDrain());

  $effect(() => {
    const selectedTimelineNodeId = editorState.selectedNodeId;
    if (!selectedTimelineNodeId) {
      clearContextStackProjection();
      return;
    }
    void refreshContextStackProjection(selectedTimelineNodeId).catch(() => {});
  });

  function handleSelect(id: string) {
    selectBibleGraphNode(selectedGraphNodeId === id ? null : id);
  }

  function handleSelectInfluence(id: string) {
    selectBibleGraphInfluence(
      graphSelection.kind === 'influence' && graphSelection.influenceId === id ? null : id,
    );
  }

  function handleSelectContextLayer(id: string) {
    selectBibleGraphContextLayer(
      graphSelection.kind === 'context_layer' && graphSelection.timelineNodeId === id ? null : id,
    );
  }

  function handleSelectEdge(id: string) {
    selectBibleGraphEdge(
      graphSelection.kind === 'edge' && graphSelection.edgeId === id ? null : id,
    );
  }

  function handleSelectNeighborhood(id: string) {
    selectBibleGraphNeighborhood(
      graphSelection.kind === 'neighborhood' && graphSelection.nodeId === id ? null : id,
    );
  }
</script>

<section class="graph-workspace" aria-label="Bible graph workspace">
  {#if graph}
    <div class="graph-summary" aria-label="Bible graph projection summary">
      <span>{graph.nodes.length} nodes</span>
      <span>{graph.edges.length} edges</span>
      <span>{graph.influences.length} influences</span>
    </div>

    <div class="graph-surface" class:has-side-lists={hasSideLists}>
      <div class="graph-viewport-shell">
        <BevyViewportPanel viewportId="graph-main" kind="graph" ariaLabel="Bible graph viewport" />
        <div class="graph-outline-fallback">
          <BibleRenderGraphOutline
            projection={graph}
            selectedNodeId={selectedGraphNodeId}
            onselect={handleSelect}
          />
        </div>
      </div>

      {#if hasSideLists}
        <GraphWorkspaceSideLists
          selection={graphSelection}
          influences={graph.influences}
          {contextLayers}
          {edgeItems}
          {neighborhoodItems}
          onselectinfluence={handleSelectInfluence}
          onselectcontextlayer={handleSelectContextLayer}
          onselectedge={handleSelectEdge}
          onselectneighborhood={handleSelectNeighborhood}
        />
      {/if}
    </div>
  {:else}
    <div class="empty-state">No graph projection</div>
  {/if}
</section>

<style>
  .graph-workspace {
    display: flex;
    flex-direction: column;
    min-height: 0;
    height: 100%;
    background: var(--color-bg-primary);
  }

  .graph-summary {
    display: flex;
    align-items: center;
    gap: 10px;
    min-height: 30px;
    padding: 4px 10px;
    border-bottom: 1px solid var(--color-border-subtle);
    color: var(--color-text-muted);
    font-size: 0.76rem;
    font-variant-numeric: tabular-nums;
  }

  .graph-surface {
    display: grid;
    grid-template-columns: minmax(0, 1fr);
    min-height: 0;
    flex: 1;
    overflow: hidden;
  }

  .graph-surface.has-side-lists {
    grid-template-columns: minmax(0, 1fr) minmax(220px, 0.34fr);
  }

  .graph-surface :global(.graph-outline) {
    max-height: none;
    height: 100%;
    border: 0;
  }

  .graph-viewport-shell {
    position: relative;
    min-width: 0;
    min-height: 0;
  }

  .graph-viewport-shell :global(.bevy-viewport-panel) {
    position: absolute;
    inset: 0;
  }

  .graph-outline-fallback {
    position: relative;
    z-index: 1;
    min-width: 0;
    min-height: 0;
    height: 100%;
    background: var(--color-bg-primary);
  }

  .empty-state {
    display: flex;
    align-items: center;
    justify-content: center;
    flex: 1;
    color: var(--color-text-muted);
    font-size: 0.85rem;
  }
</style>
