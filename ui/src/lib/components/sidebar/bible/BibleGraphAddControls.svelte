<script lang="ts">
  import {
    categoryColor,
    type BibleGraphCreateCategoryOption,
    type BibleGraphFilter,
    type BibleGraphRootCategory,
  } from './bibleGraphCategories.js';

  let {
    activeFilter,
    categories,
    onadd,
  }: {
    activeFilter: BibleGraphFilter;
    categories: BibleGraphCreateCategoryOption[];
    onadd: (category: BibleGraphRootCategory) => void;
  } = $props();

  const activeCategory = $derived(
    activeFilter === 'all'
      ? undefined
      : categories.find((category) => category.category === activeFilter),
  );
</script>

<div class="add-buttons">
  {#if activeCategory}
    <button class="add-btn" onclick={() => onadd(activeCategory.category)}>
      Add {activeCategory.label}
    </button>
  {:else}
    <div class="add-menu">
      {#each categories as category}
        <button
          class="add-btn-small"
          style="color: {categoryColor(category.category)}"
          onclick={() => onadd(category.category)}>Add {category.shortLabel}</button
        >
      {/each}
    </div>
  {/if}
</div>

<style>
  .add-buttons {
    border-top: 1px solid var(--color-border-subtle);
    border-bottom: 1px solid var(--color-border-subtle);
    background: var(--color-bg-panel);
  }

  .add-btn {
    width: 100%;
    padding: 8px 12px;
    background: color-mix(in srgb, var(--color-accent) 14%, var(--color-bg-surface));
    border: 1px solid color-mix(in srgb, var(--color-accent) 58%, var(--color-border-subtle));
    color: var(--color-accent);
    cursor: pointer;
    font-size: 0.85rem;
    font-weight: 600;
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
    justify-content: flex-start;
    gap: 4px;
    padding: 6px 8px;
    flex-wrap: wrap;
  }

  .add-btn-small {
    padding: 6px 7px;
    background: var(--color-bg-surface);
    border: 1px solid var(--color-border-subtle);
    cursor: pointer;
    font-size: 0.75rem;
    font-weight: 500;
    border-radius: 4px;
  }

  .add-btn-small:hover {
    background: var(--color-bg-hover);
  }
</style>
