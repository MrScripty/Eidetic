<script lang="ts">
  import {
    bibleGraphFilters,
    categoryColor,
    filterLabel,
    type BibleGraphFilter,
  } from './bibleGraphCategories.js';

  let {
    activeFilter,
    onselect,
  }: {
    activeFilter: BibleGraphFilter;
    onselect: (filter: BibleGraphFilter) => void;
  } = $props();
</script>

<div class="filter-pills">
  {#each bibleGraphFilters as filter}
    <button
      class="pill"
      class:active={activeFilter === filter}
      style={activeFilter === filter
        ? `color: ${categoryColor(filter)}; border-color: ${categoryColor(filter)}`
        : ''}
      onclick={() => onselect(filter)}
    >
      {filterLabel(filter)}
    </button>
  {/each}
</div>

<style>
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
</style>
