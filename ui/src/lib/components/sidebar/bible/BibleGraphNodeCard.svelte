<script lang="ts">
  import type { BibleGraphNode } from '$lib/bibleGraphTypes.js';
  import {
    categoryColor,
    categoryShortLabel,
    type BibleGraphCategory,
  } from './bibleGraphCategories.js';

  let {
    node,
    category,
    selected = false,
    onselect,
  }: {
    node: BibleGraphNode;
    category: BibleGraphCategory;
    selected?: boolean;
    onselect: (id: string) => void;
  } = $props();
</script>

<button class="node-card" class:selected onclick={() => onselect(node.id)}>
  <span class="color-dot" style="background: {categoryColor(category)}"></span>
  <span class="node-name">{node.name}</span>
  <span class="category-badge" style="color: {categoryColor(category)}">
    {categoryShortLabel(category)}
  </span>
  {#if node.system_owned}
    <span class="system-badge">ROOT</span>
  {/if}
</button>

<style>
  .node-card {
    display: flex;
    align-items: center;
    gap: 8px;
    width: 100%;
    padding: 8px 12px;
    background: none;
    border: none;
    cursor: pointer;
    transition: background 0.1s;
    text-align: left;
  }

  .node-card:hover {
    background: var(--color-bg-hover);
  }

  .node-card.selected {
    background: var(--color-bg-surface);
    border-left: 2px solid var(--color-accent);
    padding-left: 10px;
  }

  .color-dot {
    width: 10px;
    height: 10px;
    border-radius: 50%;
    flex-shrink: 0;
  }

  .node-name {
    flex: 1;
    color: var(--color-text-primary);
    font-size: 0.9rem;
    white-space: nowrap;
    overflow: hidden;
    text-overflow: ellipsis;
  }

  .category-badge,
  .system-badge {
    font-size: 0.65rem;
    font-weight: 600;
    letter-spacing: 0.05em;
    flex-shrink: 0;
  }

  .system-badge {
    color: var(--color-text-muted);
    background: var(--color-bg-elevated);
    padding: 1px 5px;
    border-radius: 8px;
  }
</style>
