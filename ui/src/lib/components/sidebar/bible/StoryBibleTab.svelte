<script lang="ts">
  import { onMount } from 'svelte';

  import { applyGraphRendererCameraCommand } from '$lib/graphRendererApi.js';
  import {
    ensureCanonicalBibleRootProjections,
    getCachedBibleGraphNodeListProjection,
    refreshBibleGraphNodeListProjection,
  } from '$lib/stores/bibleGraphNodeProjection.svelte.js';
  import {
    getCachedBibleGraphSchemaListProjection,
    refreshBibleGraphSchemaListProjection,
  } from '$lib/stores/bibleGraphSchemaProjection.svelte.js';
  import {
    bibleRenderGraphRequestForWorkspaceSelection,
    getCachedBibleRenderGraphProjection,
    refreshBibleRenderGraphProjection,
  } from '$lib/stores/bibleRenderGraphProjection.svelte.js';
  import { selectBibleGraphNode, selectedBibleGraphNodeId } from '$lib/stores/bible.svelte.js';
  import { editorState } from '$lib/stores/editor.svelte.js';
  import BibleGraphAddControls from './BibleGraphAddControls.svelte';
  import BibleGraphCategoryFilters from './BibleGraphCategoryFilters.svelte';
  import BibleGraphNodeCard from './BibleGraphNodeCard.svelte';
  import BibleGraphNodeDetail from './BibleGraphNodeDetail.svelte';
  import BibleRenderGraphOutline from './BibleRenderGraphOutline.svelte';
  import {
    bibleGraphCreateOptions,
    bibleGraphFilterOptions,
    categoryForRenderNode,
    categoryProjection,
    nodeCategory,
    type BibleGraphFilter,
    type BibleGraphRootCategory,
  } from './bibleGraphCategories.js';
  import { createBibleGraphNodeForCategory } from './bibleGraphNodeCreateFlow.js';

  let searchQuery = $state('');
  let activeFilter: BibleGraphFilter = $state('all');
  let loadError = $state<string | null>(null);
  let lastRenderGraphRequestKey: string | null = null;

  const nodeListProjection = $derived(getCachedBibleGraphNodeListProjection());
  const schemaProjection = $derived(getCachedBibleGraphSchemaListProjection());
  const renderGraphProjection = $derived(getCachedBibleRenderGraphProjection());
  const selectedGraphNodeId = $derived(selectedBibleGraphNodeId());
  const graphNodes = $derived(nodeListProjection?.payload.nodes ?? []);
  const graphFilterOptions = $derived(bibleGraphFilterOptions(schemaProjection?.payload));
  const graphCreateOptions = $derived(bibleGraphCreateOptions(schemaProjection?.payload));
  const activeFilterRootId = $derived(
    activeFilter === 'all'
      ? null
      : (categoryProjection(activeFilter, schemaProjection?.payload)?.root_node_id ?? null),
  );

  const filteredEntities = $derived(() => {
    let list = graphNodes;
    if (activeFilter !== 'all') {
      const filterCategory = activeFilter as BibleGraphRootCategory;
      list = list.filter((node) => nodeCategory(node) === categoryForRenderNode(filterCategory));
    }
    if (searchQuery.trim()) {
      const q = searchQuery.toLowerCase();
      list = list.filter((node) => node.name.toLowerCase().includes(q));
    }
    return list;
  });

  async function handleAdd(category: BibleGraphRootCategory) {
    try {
      loadError = null;
      const projection = await createBibleGraphNodeForCategory(category, {
        graphNodes,
      });
      await refreshActiveRenderGraphProjection();
      selectBibleGraphNode(projection.projection.payload.node.id);
    } catch (error) {
      loadError = error instanceof Error ? error.message : 'Failed to create bible graph node';
    }
  }

  function handleSelect(id: string, event?: MouseEvent) {
    if (event?.ctrlKey) {
      void applyGraphRendererCameraCommand({
        type: 'frame_node',
        node_id: id,
      }).catch(() => {});
    }
    selectBibleGraphNode(selectedGraphNodeId === id ? null : id);
  }

  function activeRenderGraphQuery() {
    return bibleRenderGraphRequestForWorkspaceSelection({
      selectedTimelineNodeId: editorState.selectedNodeId,
      focusedRootId: activeFilterRootId,
      search: searchQuery,
    });
  }

  async function refreshActiveRenderGraphProjection(): Promise<void> {
    await refreshBibleRenderGraphProjection(activeRenderGraphQuery());
  }

  async function handleNodeDeleted(): Promise<void> {
    selectBibleGraphNode(null);
    await refreshActiveRenderGraphProjection();
  }

  async function loadBibleGraphNodes(): Promise<void> {
    try {
      loadError = null;
      await refreshBibleGraphSchemaListProjection();
      const projection = await refreshBibleGraphNodeListProjection();
      if (projection.payload.nodes.length === 0) {
        await ensureCanonicalBibleRootProjections();
      }
      await refreshActiveRenderGraphProjection();
    } catch (error) {
      loadError = error instanceof Error ? error.message : 'Failed to load bible graph nodes';
    }
  }

  onMount(() => {
    void loadBibleGraphNodes();
  });

  $effect(() => {
    const request = activeRenderGraphQuery();
    const requestKey = JSON.stringify(request);
    if (requestKey === lastRenderGraphRequestKey) {
      return;
    }
    lastRenderGraphRequestKey = requestKey;
    void refreshBibleRenderGraphProjection(request).catch((error) => {
      loadError = error instanceof Error ? error.message : 'Failed to load bible graph nodes';
    });
  });
</script>

<div class="bible-tab">
  <div class="search-bar">
    <input
      class="search-input"
      type="text"
      placeholder="Search entities..."
      bind:value={searchQuery}
    />
  </div>

  <BibleGraphCategoryFilters
    {activeFilter}
    filters={graphFilterOptions}
    onselect={(filter) => (activeFilter = filter)}
  />

  <BibleGraphAddControls {activeFilter} categories={graphCreateOptions} onadd={handleAdd} />

  <BibleRenderGraphOutline
    projection={renderGraphProjection?.payload ?? null}
    selectedNodeId={selectedGraphNodeId}
    query={searchQuery}
    onselect={handleSelect}
  />

  {#if loadError}
    <div class="load-error">{loadError}</div>
  {/if}

  <ul class="entity-list">
    {#each filteredEntities() as entity (entity.id)}
      <li>
        <BibleGraphNodeCard
          node={entity}
          category={nodeCategory(entity)}
          selected={selectedGraphNodeId === entity.id}
          onselect={handleSelect}
        />
      </li>
    {/each}
    {#if filteredEntities().length === 0}
      <li class="empty-state">
        {searchQuery ? 'No matching entities' : 'No entities yet'}
      </li>
    {/if}
  </ul>

  {#if selectedGraphNodeId}
    <div class="inline-detail">
      <BibleGraphNodeDetail
        nodeId={selectedGraphNodeId}
        onclose={() => selectBibleGraphNode(null)}
        ondeleted={handleNodeDeleted}
        edgeTargetNodes={graphNodes}
      />
    </div>
  {/if}
</div>

<style>
  .bible-tab {
    display: flex;
    flex-direction: column;
    height: 100%;
  }

  .search-bar {
    padding: 8px 12px 4px;
  }

  .search-input {
    width: 100%;
    padding: 6px 10px;
    background: var(--color-bg-surface);
    color: var(--color-text-primary);
    border: 1px solid var(--color-border-default);
    border-radius: 4px;
    font-size: 0.85rem;
    box-sizing: border-box;
  }

  .search-input:focus {
    outline: none;
    border-color: var(--color-accent);
  }

  .search-input::placeholder {
    color: var(--color-text-muted);
  }

  .entity-list {
    list-style: none;
    margin: 0;
    padding: 0;
    flex: 0 1 auto;
    max-height: 34%;
    overflow-y: auto;
    border-bottom: 1px solid var(--color-border-subtle);
  }

  .inline-detail {
    flex: 1;
    min-height: 0;
    overflow: hidden;
  }

  .empty-state {
    padding: 16px 12px;
    color: var(--color-text-muted);
    font-size: 0.85rem;
    text-align: center;
  }

  .load-error {
    margin: 8px 12px 0;
    padding: 6px 8px;
    border: 1px solid var(--color-danger, #b74c4c);
    border-radius: 4px;
    color: var(--color-danger, #b74c4c);
    font-size: 0.75rem;
  }
</style>
