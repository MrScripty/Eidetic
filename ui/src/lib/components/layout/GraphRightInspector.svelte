<script lang="ts">
  import BibleGraphNodeDetail from '../sidebar/bible/BibleGraphNodeDetail.svelte';
  import {
    bibleState,
    clearBibleGraphSelection,
    selectedBibleGraphNodeId,
  } from '$lib/stores/bible.svelte.js';
  import { getCachedBibleRenderGraphProjection } from '$lib/stores/bibleRenderGraphProjection.svelte.js';
  import { getCachedContextStackProjection } from '$lib/stores/contextStackProjection.svelte.js';
  import GraphSelectionDetail from './GraphSelectionDetail.svelte';

  const graphSelection = $derived(bibleState.graphSelection);
  const selectedGraphNodeId = $derived(selectedBibleGraphNodeId());
  const renderGraphProjection = $derived(getCachedBibleRenderGraphProjection());
  const renderGraph = $derived(renderGraphProjection?.payload ?? null);
  const contextStackProjection = $derived(getCachedContextStackProjection());
  const contextStack = $derived(contextStackProjection?.payload ?? null);
</script>

{#if selectedGraphNodeId}
  <div class="entity-detail-panel">
    <BibleGraphNodeDetail nodeId={selectedGraphNodeId} onclose={clearBibleGraphSelection} />
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
