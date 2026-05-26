<script lang="ts">
  import {
    categoryColor,
    type BibleGraphCategoryFilterOption,
    type BibleGraphFilter,
  } from './bibleGraphCategories.js';

  let {
    activeFilter,
    filters,
    onselect,
  }: {
    activeFilter: BibleGraphFilter;
    filters: BibleGraphCategoryFilterOption[];
    onselect: (filter: BibleGraphFilter) => void;
  } = $props();
</script>

<div class="filter-pills">
  {#each filters as filter}
    <button
      class="pill"
      class:active={activeFilter === filter.filter}
      style={activeFilter === filter.filter
        ? `color: ${categoryColor(filter.filter)}; border-color: ${categoryColor(filter.filter)}`
        : ''}
      onclick={() => onselect(filter.filter)}
    >
      {filter.label}
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
