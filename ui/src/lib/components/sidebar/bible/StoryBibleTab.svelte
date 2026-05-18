<script lang="ts">
  import { onMount } from 'svelte';

  import type { BibleGraphNodeId, EntityCategory } from '$lib/types.js';
  import {
    createBibleGraphNodeProjection,
    ensureCanonicalBibleRootProjections,
    getCachedBibleGraphNodeListProjection,
    refreshBibleGraphNodeListProjection,
  } from '$lib/stores/bibleGraphNodeProjection.svelte.js';
  import {
    getCachedBibleGraphSchemaListProjection,
    refreshBibleGraphSchemaListProjection,
  } from '$lib/stores/bibleGraphSchemaProjection.svelte.js';
  import { bibleState, selectBibleGraphNode } from '$lib/stores/bible.svelte.js';
  import BibleGraphAddControls from './BibleGraphAddControls.svelte';
  import BibleGraphCategoryFilters from './BibleGraphCategoryFilters.svelte';
  import BibleGraphNodeCard from './BibleGraphNodeCard.svelte';
  import {
    canonicalParents,
    canonicalRootSchemaKeys,
    categorySchemaAvailable,
    defaultNames,
    nodeCategory,
    schemaKeys,
    type BibleGraphFilter,
  } from './bibleGraphCategories.js';

  let searchQuery = $state('');
  let activeFilter: BibleGraphFilter = $state('All');
  let loadError = $state<string | null>(null);

  const nodeListProjection = $derived(getCachedBibleGraphNodeListProjection());
  const schemaProjection = $derived(getCachedBibleGraphSchemaListProjection());
  const graphNodes = $derived(nodeListProjection?.payload.nodes ?? []);
  const disabledAddCategories = $derived(
    new Set(
      (['Character', 'Location', 'Prop', 'Theme', 'Event'] as EntityCategory[]).filter(
        (category) => !categorySchemaAvailable(category, schemaProjection?.payload),
      ),
    ),
  );

  const filteredEntities = $derived(() => {
    let list = graphNodes;
    if (activeFilter !== 'All') {
      list = list.filter((node) => nodeCategory(node) === activeFilter);
    }
    if (searchQuery.trim()) {
      const q = searchQuery.toLowerCase();
      list = list.filter((node) => node.name.toLowerCase().includes(q));
    }
    return list;
  });

  async function handleAdd(category: EntityCategory) {
    try {
      loadError = null;
      if (!categorySchemaAvailable(category, schemaProjection?.payload)) {
        await refreshBibleGraphSchemaListProjection();
      }
      if (!categorySchemaAvailable(category, getCachedBibleGraphSchemaListProjection()?.payload)) {
        throw new Error(`Schema unavailable for ${category}`);
      }
      const parentId = await ensureRootForCategory(category);
      const nodeId = `node.${schemaKeys[category]}.${crypto.randomUUID()}`;
      await createBibleGraphNodeProjection({
        node_id: nodeId,
        parent_id: parentId,
        schema_key: schemaKeys[category],
        name: defaultNames[category],
        sort_order: nextSortOrder(category),
      });
      selectBibleGraphNode(nodeId);
    } catch (error) {
      loadError = error instanceof Error ? error.message : 'Failed to create bible graph node';
    }
  }

  function handleSelect(id: string) {
    selectBibleGraphNode(bibleState.selectedGraphNodeId === id ? null : id);
  }

  function nextSortOrder(category: EntityCategory): number {
    return graphNodes.filter((node) => node.parent_id === canonicalParents[category]).length;
  }

  async function ensureRootForCategory(category: EntityCategory): Promise<BibleGraphNodeId> {
    const existingRoot = graphNodes.find(
      (node) => node.schema_key === canonicalRootSchemaKeys[category],
    );
    if (existingRoot) return existingRoot.id;

    const response = await ensureCanonicalBibleRootProjections();
    const ensuredRoot = response.projection.payload.nodes.find(
      (node) => node.schema_key === canonicalRootSchemaKeys[category],
    );
    return ensuredRoot?.id ?? canonicalParents[category];
  }

  async function loadBibleGraphNodes(): Promise<void> {
    try {
      loadError = null;
      await refreshBibleGraphSchemaListProjection();
      const projection = await refreshBibleGraphNodeListProjection();
      if (projection.payload.nodes.length === 0) {
        await ensureCanonicalBibleRootProjections();
      }
    } catch (error) {
      loadError = error instanceof Error ? error.message : 'Failed to load bible graph nodes';
    }
  }

  onMount(() => {
    void loadBibleGraphNodes();
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

  <BibleGraphCategoryFilters {activeFilter} onselect={(filter) => (activeFilter = filter)} />

  {#if loadError}
    <div class="load-error">{loadError}</div>
  {/if}

  <ul class="entity-list">
    {#each filteredEntities() as entity (entity.id)}
      <li>
        <BibleGraphNodeCard
          node={entity}
          category={nodeCategory(entity)}
          selected={bibleState.selectedGraphNodeId === entity.id}
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

  <BibleGraphAddControls
    {activeFilter}
    disabledCategories={disabledAddCategories}
    onadd={handleAdd}
  />
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
    flex: 1;
    overflow-y: auto;
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
