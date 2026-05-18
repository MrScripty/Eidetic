<script lang="ts">
  import { onMount } from 'svelte';

  import type { BibleGraphNode, BibleGraphNodeId, EntityCategory } from '$lib/types.js';
  import {
    createBibleGraphNodeProjection,
    ensureCanonicalBibleRootProjections,
    getCachedBibleGraphNodeListProjection,
    refreshBibleGraphNodeListProjection,
  } from '$lib/stores/bibleGraphNodeProjection.svelte.js';
  import { bibleState, selectBibleGraphNode } from '$lib/stores/bible.svelte.js';
  import BibleGraphNodeCard from './BibleGraphNodeCard.svelte';

  let searchQuery = $state('');
  let activeFilter: EntityCategory | 'All' = $state('All');
  let loadError = $state<string | null>(null);

  const categories: (EntityCategory | 'All')[] = [
    'All',
    'Character',
    'Location',
    'Prop',
    'Theme',
    'Event',
  ];

  const canonicalParents: Record<EntityCategory, BibleGraphNodeId> = {
    Character: 'canonical.characters',
    Location: 'canonical.places',
    Prop: 'canonical.objects',
    Theme: 'canonical.themes',
    Event: 'canonical.events',
  };

  const schemaKeys: Record<EntityCategory, string> = {
    Character: 'character',
    Location: 'location',
    Prop: 'prop',
    Theme: 'theme',
    Event: 'event',
  };

  const defaultNames: Record<EntityCategory, string> = {
    Character: 'New Character',
    Location: 'New Location',
    Prop: 'New Prop',
    Theme: 'New Theme',
    Event: 'New Event',
  };

  const nodeListProjection = $derived(getCachedBibleGraphNodeListProjection());
  const graphNodes = $derived(nodeListProjection?.payload.nodes ?? []);

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
      await ensureRootsIfMissing();
      const nodeId = `node.${schemaKeys[category]}.${crypto.randomUUID()}`;
      await createBibleGraphNodeProjection({
        node_id: nodeId,
        parent_id: canonicalParents[category],
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

  function filterLabel(cat: EntityCategory | 'All'): string {
    if (cat === 'All') return 'All';
    return cat.slice(0, 3);
  }

  function filterColor(cat: EntityCategory | 'All'): string {
    switch (cat) {
      case 'Character':
        return 'var(--color-entity-character)';
      case 'Location':
        return 'var(--color-entity-location)';
      case 'Prop':
        return 'var(--color-entity-prop)';
      case 'Theme':
        return 'var(--color-entity-theme)';
      case 'Event':
        return 'var(--color-entity-event)';
      default:
        return 'var(--color-text-secondary)';
    }
  }

  function nodeCategory(node: BibleGraphNode): EntityCategory | 'Other' {
    switch (node.schema_key) {
      case 'canonical.root.characters':
      case 'character':
        return 'Character';
      case 'canonical.root.places':
      case 'location':
        return 'Location';
      case 'canonical.root.objects':
      case 'prop':
        return 'Prop';
      case 'canonical.root.themes':
      case 'theme':
        return 'Theme';
      case 'canonical.root.events':
      case 'event':
        return 'Event';
      default:
        return parentCategory(node.parent_id);
    }
  }

  function parentCategory(parentId: BibleGraphNodeId | null | undefined): EntityCategory | 'Other' {
    switch (parentId) {
      case 'canonical.characters':
        return 'Character';
      case 'canonical.places':
        return 'Location';
      case 'canonical.objects':
        return 'Prop';
      case 'canonical.themes':
        return 'Theme';
      case 'canonical.events':
        return 'Event';
      default:
        return 'Other';
    }
  }

  function nextSortOrder(category: EntityCategory): number {
    return graphNodes.filter((node) => node.parent_id === canonicalParents[category]).length;
  }

  async function ensureRootsIfMissing(): Promise<void> {
    if (graphNodes.length === 0) {
      await ensureCanonicalBibleRootProjections();
    }
  }

  async function loadBibleGraphNodes(): Promise<void> {
    try {
      loadError = null;
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

  <div class="filter-pills">
    {#each categories as cat}
      <button
        class="pill"
        class:active={activeFilter === cat}
        style={activeFilter === cat
          ? `color: ${filterColor(cat)}; border-color: ${filterColor(cat)}`
          : ''}
        onclick={() => (activeFilter = cat)}
      >
        {filterLabel(cat)}
      </button>
    {/each}
  </div>

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

  <div class="add-buttons">
    {#if activeFilter !== 'All'}
      <button class="add-btn" onclick={() => handleAdd(activeFilter as EntityCategory)}>
        + Add {activeFilter}
      </button>
    {:else}
      <div class="add-menu">
        <button
          class="add-btn-small"
          style="color: var(--color-entity-character)"
          onclick={() => handleAdd('Character')}>+ Chr</button
        >
        <button
          class="add-btn-small"
          style="color: var(--color-entity-location)"
          onclick={() => handleAdd('Location')}>+ Loc</button
        >
        <button
          class="add-btn-small"
          style="color: var(--color-entity-prop)"
          onclick={() => handleAdd('Prop')}>+ Prp</button
        >
        <button
          class="add-btn-small"
          style="color: var(--color-entity-theme)"
          onclick={() => handleAdd('Theme')}>+ Thm</button
        >
        <button
          class="add-btn-small"
          style="color: var(--color-entity-event)"
          onclick={() => handleAdd('Event')}>+ Evt</button
        >
      </div>
    {/if}
  </div>
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

  .filter-pills {
    display: flex;
    gap: 4px;
    padding: 6px 12px;
    flex-wrap: wrap;
  }

  .pill {
    padding: 2px 8px;
    font-size: 0.7rem;
    border: 1px solid var(--color-border-default);
    border-radius: 12px;
    background: none;
    color: var(--color-text-secondary);
    cursor: pointer;
    text-transform: uppercase;
    letter-spacing: 0.04em;
    font-weight: 500;
  }

  .pill.active {
    background: var(--color-bg-elevated);
  }

  .pill:hover {
    background: var(--color-bg-hover);
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

  .add-buttons {
    border-top: 1px solid var(--color-border-subtle);
  }

  .add-btn {
    width: 100%;
    padding: 8px 12px;
    background: none;
    border: none;
    color: var(--color-accent);
    cursor: pointer;
    font-size: 0.85rem;
    text-align: center;
  }

  .add-btn:hover {
    background: var(--color-bg-hover);
  }

  .add-menu {
    display: flex;
    justify-content: center;
    gap: 2px;
    padding: 4px;
  }

  .add-btn-small {
    padding: 6px 8px;
    background: none;
    border: none;
    cursor: pointer;
    font-size: 0.75rem;
    font-weight: 500;
    border-radius: 4px;
  }

  .add-btn-small:hover {
    background: var(--color-bg-hover);
  }
</style>
