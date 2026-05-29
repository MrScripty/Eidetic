<script lang="ts">
  import BibleGraphNodeDetail from '../sidebar/bible/BibleGraphNodeDetail.svelte';
  import {
    bibleState,
    clearBibleGraphSelection,
    selectBibleGraphNode,
    selectedBibleGraphNodeId,
  } from '$lib/stores/bible.svelte.js';
  import { getCachedBibleRenderGraphProjection } from '$lib/stores/bibleRenderGraphProjection.svelte.js';
  import { getCachedBibleGraphNodeListProjection } from '$lib/stores/bibleGraphNodeProjection.svelte.js';
  import { getCachedContextStackProjection } from '$lib/stores/contextStackProjection.svelte.js';
  import GraphSelectionDetail from './GraphSelectionDetail.svelte';

  const graphSelection = $derived(bibleState.graphSelection);
  const selectedGraphNodeId = $derived(selectedBibleGraphNodeId());
  const nodeListProjection = $derived(getCachedBibleGraphNodeListProjection());
  const graphNodes = $derived(nodeListProjection?.payload.nodes);
  const renderGraphProjection = $derived(getCachedBibleRenderGraphProjection());
  const renderGraph = $derived(renderGraphProjection?.payload ?? null);
  const contextStackProjection = $derived(getCachedContextStackProjection());
  const contextStack = $derived(contextStackProjection?.payload ?? null);
</script>

{#if selectedGraphNodeId}
  <div class="entity-detail-panel">
    <BibleGraphNodeDetail
      nodeId={selectedGraphNodeId}
      onclose={clearBibleGraphSelection}
      onselect={selectBibleGraphNode}
      oncreated={selectBibleGraphNode}
      onrenamed={selectBibleGraphNode}
      {graphNodes}
      edgeTargetNodes={renderGraph?.nodes ?? []}
    />
  </div>
{:else}
  <div class="entity-detail-panel">
    <GraphSelectionDetail projection={renderGraph} selection={graphSelection} {contextStack} />
  </div>
{/if}

<style>
  .entity-detail-panel {
    flex: 1;
    overflow: hidden;
    min-height: 0;
  }
</style>
