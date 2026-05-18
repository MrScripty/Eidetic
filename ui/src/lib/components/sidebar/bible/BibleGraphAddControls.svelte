<script lang="ts">
  import {
    bibleGraphCategories,
    categoryColor,
    categoryShortLabel,
    type BibleGraphFilter,
    type BibleGraphRootCategory,
  } from './bibleGraphCategories.js';

  let {
    activeFilter,
    disabledCategories,
    onadd,
  }: {
    activeFilter: BibleGraphFilter;
    disabledCategories: Set<BibleGraphRootCategory>;
    onadd: (category: BibleGraphRootCategory) => void;
  } = $props();
</script>

<div class="add-buttons">
  {#if activeFilter !== 'All'}
    <button
      class="add-btn"
      disabled={disabledCategories.has(activeFilter)}
      onclick={() => onadd(activeFilter)}
    >
      + Add {activeFilter}
    </button>
  {:else}
    <div class="add-menu">
      {#each bibleGraphCategories as category}
        <button
          class="add-btn-small"
          disabled={disabledCategories.has(category)}
          style="color: {categoryColor(category)}"
          onclick={() => onadd(category)}>+ {categoryShortLabel(category)}</button
        >
      {/each}
    </div>
  {/if}
</div>

<style>
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

  .add-btn:disabled,
  .add-btn-small:disabled {
    cursor: default;
    opacity: 0.45;
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
