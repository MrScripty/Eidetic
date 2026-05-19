<script lang="ts">
  import type { BibleGraphNodeId, BibleRenderGraphProjection } from '$lib/bibleGraphTypes.js';
  import { bibleRenderGraphOutlineItems } from './bibleRenderGraphOutline.js';

  let {
    projection,
    selectedNodeId = null,
    query = '',
    onselect,
  }: {
    projection: BibleRenderGraphProjection | null;
    selectedNodeId?: BibleGraphNodeId | null;
    query?: string;
    onselect: (id: BibleGraphNodeId) => void;
  } = $props();

  const items = $derived(
    projection ? bibleRenderGraphOutlineItems(projection, selectedNodeId, query) : [],
  );
</script>

{#if items.length > 0}
  <nav class="graph-outline" aria-label="Bible graph">
    <ul>
      {#each items as item (item.node_id)}
        <li>
          <button
            class:selected={item.selected}
            style:--graph-indent={`${Math.min(item.depth, 6) * 10}px`}
            aria-pressed={item.selected}
            onclick={() => onselect(item.node_id)}
          >
            <span class="node-label">{item.label}</span>
            <span
              class="connection-count"
              aria-label={`${item.connected_node_count} connected nodes, ${item.edge_count} edges`}
            >
              {item.connected_node_count}
            </span>
          </button>
        </li>
      {/each}
    </ul>
  </nav>
{/if}

<style>
  .graph-outline {
    border-top: 1px solid var(--color-border-subtle);
    border-bottom: 1px solid var(--color-border-subtle);
    max-height: 140px;
    overflow-y: auto;
  }

  ul {
    list-style: none;
    margin: 0;
    padding: 4px 0;
  }

  button {
    display: flex;
    align-items: center;
    gap: 8px;
    width: 100%;
    min-height: 28px;
    padding: 4px 10px 4px calc(10px + var(--graph-indent));
    border: 0;
    background: transparent;
    color: var(--color-text-secondary);
    cursor: pointer;
    text-align: left;
  }

  button:hover,
  button:focus-visible {
    background: var(--color-bg-hover);
    color: var(--color-text-primary);
  }

  button:focus-visible {
    outline: 2px solid var(--color-accent);
    outline-offset: -2px;
  }

  button.selected {
    background: var(--color-bg-surface);
    color: var(--color-text-primary);
    box-shadow: inset 2px 0 0 var(--color-accent);
  }

  .node-label {
    flex: 1;
    min-width: 0;
    overflow: hidden;
    text-overflow: ellipsis;
    white-space: nowrap;
    font-size: 0.78rem;
  }

  .connection-count {
    display: inline-flex;
    align-items: center;
    justify-content: center;
    min-width: 18px;
    height: 18px;
    border: 1px solid var(--color-border-default);
    border-radius: 4px;
    color: var(--color-text-muted);
    font-size: 0.68rem;
    font-variant-numeric: tabular-nums;
  }
</style>
