<script lang="ts">
  import type { StoryNode } from '$lib/timelineTypes.js';

  let {
    node,
    selectedNodeId,
    parentNode,
    siblingNodes,
    currentNodeIndex,
    adjacentParents,
    onnavigate,
    onselectnode,
  }: {
    node: StoryNode;
    selectedNodeId: string | null;
    parentNode: StoryNode;
    siblingNodes: StoryNode[];
    currentNodeIndex: number;
    adjacentParents: { before: StoryNode | null; after: StoryNode | null };
    onnavigate: (direction: -1 | 1) => void;
    onselectnode: (node: StoryNode) => void;
  } = $props();
</script>

<div class="sub-beat-context">
  <div class="sub-beat-nav">
    <button
      type="button"
      class="nav-btn"
      disabled={currentNodeIndex <= 0}
      onclick={() => onnavigate(-1)}>&lsaquo; Prev</button
    >
    <span class="nav-position">{node.level} {currentNodeIndex + 1} of {siblingNodes.length}</span>
    <button
      type="button"
      class="nav-btn"
      disabled={currentNodeIndex >= siblingNodes.length - 1}
      onclick={() => onnavigate(1)}>Next &rsaquo;</button
    >
  </div>

  <div class="context-card">
    <div class="context-card-header">{parentNode.level}: {parentNode.name}</div>
    <p class="context-card-body">{parentNode.content.notes || 'No notes'}</p>
  </div>

  {#if siblingNodes.length > 1}
    <details class="context-card" open>
      <summary class="context-card-header clickable">{node.level} Structure</summary>
      <div class="beat-structure">
        {#each siblingNodes as sibling, i}
          <button
            type="button"
            class="beat-outline-item"
            class:current={sibling.id === selectedNodeId}
            onclick={() => onselectnode(sibling)}
          >
            <span class="beat-outline-number">{i + 1}</span>
            <div class="beat-outline-info">
              <span class="beat-outline-name"
                >{sibling.name}
                {#if sibling.beat_type}<span class="beat-outline-type"
                    >{typeof sibling.beat_type === 'string'
                      ? sibling.beat_type
                      : sibling.beat_type.Custom}</span
                  >{/if}</span
              >
              <span class="beat-outline-text">{sibling.content.notes || '\u2014'}</span>
            </div>
          </button>
        {/each}
      </div>
    </details>
  {/if}

  {#if adjacentParents.before || adjacentParents.after}
    <details class="context-card">
      <summary class="context-card-header clickable">Adjacent {parentNode.level}s</summary>
      <div class="adjacent-scenes">
        {#if adjacentParents.before}
          <div class="adjacent-scene">
            <span class="adjacent-scene-label">Previous</span>
            <span class="adjacent-scene-name">{adjacentParents.before.name}</span>
            <p class="adjacent-scene-notes">
              {adjacentParents.before.content.notes || 'No notes'}
            </p>
          </div>
        {/if}
        {#if adjacentParents.after}
          <div class="adjacent-scene">
            <span class="adjacent-scene-label">Next</span>
            <span class="adjacent-scene-name">{adjacentParents.after.name}</span>
            <p class="adjacent-scene-notes">
              {adjacentParents.after.content.notes || 'No notes'}
            </p>
          </div>
        {/if}
      </div>
    </details>
  {/if}
</div>
