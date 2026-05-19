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
  }: {
    node: StoryNode;
    statusLabel: string;
    isGenerating: boolean;
    hasChildren: boolean;
    childLevelName: string | null;
    ontogglelock: () => void;
    ongenerate: () => void;
  } = $props();
</script>

<div class="editor-header">
  <h3 class="clip-title">{node.name}</h3>
  <span class="level-badge">{node.level}</span>
  <span class="status-badge" data-status={node.content.status}>
    {statusLabel}
  </span>
  <button class="lock-toggle" class:locked={node.locked} onclick={ontogglelock}>
    {node.locked ? 'Unlock' : 'Lock'}
  </button>
  <button
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
