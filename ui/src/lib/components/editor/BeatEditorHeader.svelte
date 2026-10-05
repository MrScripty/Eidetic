<script lang="ts">
  import type { StoryNode } from '$lib/timelineTypes.js';

  let {
    node,
    statusLabel,
    isGenerating,
    hasChildren,
    childLevelName,
    ontogglelock,
    ongenerate,
    onaddchild,
    creatingChild = false,
  }: {
    node: StoryNode;
    statusLabel: string;
    isGenerating: boolean;
    hasChildren: boolean;
    childLevelName: string | null;
    ontogglelock: () => void;
    ongenerate: () => void;
    onaddchild: () => void;
    creatingChild?: boolean;
  } = $props();
</script>

<div class="editor-header">
  <h3 class="clip-title">{node.name}</h3>
  <span class="level-badge">{node.level}</span>
  <span class="status-badge" data-status={node.content.status}>
    {statusLabel}
  </span>
  <button type="button" class="lock-toggle" class:locked={node.locked} onclick={ontogglelock}>
    {node.locked ? 'Unlock' : 'Lock'}
  </button>
  {#if childLevelName}
    <button type="button" onclick={onaddchild} disabled={creatingChild}>
      {creatingChild ? 'Adding...' : `Add ${childLevelName}`}
    </button>
  {/if}
  <button
    type="button"
    class="generate-btn"
    class:generating={isGenerating}
    onclick={ongenerate}
    disabled={!node.content.notes.trim() || node.locked || isGenerating}
  >
    {#if isGenerating}
      Generating...
    {:else if hasChildren}
      Generate {childLevelName}s
    {:else}
      Generate
    {/if}
  </button>
</div>
