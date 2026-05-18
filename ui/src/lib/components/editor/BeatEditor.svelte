<script lang="ts">
  import type { YTextEvent } from 'yjs';
  import { childLevel } from '$lib/timelineTypes.js';
  import {
    editorState,
    startGeneration,
    startBatchGeneration,
    setBatchTotalCount,
  } from '$lib/stores/editor.svelte.js';
  import { timelineState, zoomToRange, childrenOf, findNode } from '$lib/stores/timeline.svelte.js';
  import { generateContent, generateChildren, generateBatch, getAiContext } from '$lib/api.js';
  import {
    applyTimelineChildrenCommand,
    applyTimelineNodeLockCommand,
    applyTimelineNodeNotesCommand,
  } from '$lib/stores/timelineRenderProjection.svelte.js';
  import { getNodeNotes } from '$lib/yjs.js';

  let debounceTimer: ReturnType<typeof setTimeout> | null = null;

  let isGenerating = $derived(
    (editorState.streamingNodeId != null &&
      editorState.streamingNodeId === editorState.selectedNodeId) ||
      (editorState.batchParentNodeId != null &&
        editorState.batchParentNodeId === editorState.selectedNodeId),
  );

  // Child planning state
  let planning = $state(false);

  // Node hierarchy context
  let isChildNode = $derived(editorState.selectedNode?.parent_id != null);

  let childNodes = $derived.by(() => {
    if (!editorState.selectedNodeId) return [];
    return childrenOf(editorState.selectedNodeId);
  });

  let hasChildren = $derived(childNodes.length > 0);

  let parentNode = $derived.by(() => {
    if (!isChildNode || !editorState.selectedNode) return null;
    return findNode(editorState.selectedNode.parent_id!) ?? null;
  });

  let siblingNodes = $derived.by(() => {
    if (!isChildNode || !editorState.selectedNode?.parent_id) return [];
    return childrenOf(editorState.selectedNode.parent_id);
  });

  let currentNodeIndex = $derived(
    siblingNodes.findIndex((n) => n.id === editorState.selectedNodeId),
  );

  // Adjacent nodes at the parent level.
  let adjacentParents = $derived.by(() => {
    if (!isChildNode || !parentNode) return { before: null, after: null };
    const parentLevel = parentNode.level;
    const allAtLevel =
      timelineState.timeline?.nodes
        .filter((n) => n.level === parentLevel)
        .sort((a, b) => a.time_range.start_ms - b.time_range.start_ms) ?? [];
    const idx = allAtLevel.findIndex((n) => n.id === parentNode!.id);
    if (idx < 0) return { before: null, after: null };
    return {
      before: idx > 0 ? allAtLevel[idx - 1] : null,
      after: idx < allAtLevel.length - 1 ? allAtLevel[idx + 1] : null,
    };
  });

  // Persistent context preview
  let nodeContext: { system: string; user: string } | null = $state(null);
  let contextLoading = $state(false);
  let contextNodeId: string | null = $state(null);

  // Auto-fetch context when the selected node changes.
  $effect(() => {
    const nodeId = editorState.selectedNodeId;
    const notes = editorState.selectedNode?.content.notes;
    if (!nodeId || !notes?.trim()) {
      nodeContext = null;
      contextNodeId = null;
      return;
    }
    if (nodeId === contextNodeId && nodeContext) return;
    contextLoading = true;
    contextNodeId = nodeId;
    getAiContext(nodeId)
      .then((ctx) => {
        if (editorState.selectedNodeId === nodeId) {
          nodeContext = ctx;
        }
      })
      .catch(() => {
        if (editorState.selectedNodeId === nodeId) {
          nodeContext = null;
        }
      })
      .finally(() => {
        if (editorState.selectedNodeId === nodeId) {
          contextLoading = false;
        }
      });
  });

  function refreshContext() {
    const nodeId = editorState.selectedNodeId;
    if (!nodeId) return;
    contextLoading = true;
    contextNodeId = nodeId;
    getAiContext(nodeId)
      .then((ctx) => {
        if (editorState.selectedNodeId === nodeId) {
          nodeContext = ctx;
        }
      })
      .catch(() => {
        if (editorState.selectedNodeId === nodeId) {
          nodeContext = null;
        }
      })
      .finally(() => {
        if (editorState.selectedNodeId === nodeId) {
          contextLoading = false;
        }
      });
  }

  // Observe Y.Text changes for node notes while the legacy notes path remains on Y.Doc.
  $effect(() => {
    const nodeId = editorState.selectedNodeId;
    if (!nodeId) return;

    const yNotes = getNodeNotes(nodeId);

    const onNotesChange = (_event: YTextEvent) => {
      if (editorState.selectedNode && editorState.selectedNodeId === nodeId) {
        editorState.selectedNode.content.notes = yNotes.toString();
      }
    };

    yNotes.observe(onNotesChange);

    return () => {
      yNotes.unobserve(onNotesChange);
    };
  });

  function statusLabel(status: string): string {
    switch (status) {
      case 'Empty':
        return 'No content';
      case 'NotesOnly':
        return 'Notes written';
      case 'Generating':
        return 'AI generating...';
      case 'HasContent':
        return 'Has content';
      default:
        return status;
    }
  }

  function handleNotesInput(e: Event) {
    const value = (e.target as HTMLTextAreaElement).value;
    if (editorState.selectedNode) {
      editorState.selectedNode.content.notes = value;
    }
    if (debounceTimer) clearTimeout(debounceTimer);
    debounceTimer = setTimeout(async () => {
      if (editorState.selectedNodeId) {
        await applyTimelineNodeNotesCommand({
          node_id: editorState.selectedNodeId,
          notes: value,
        });
      }
    }, 500);
  }

  async function handleToggleLock() {
    if (!editorState.selectedNodeId || !editorState.selectedNode) return;
    const locked = !editorState.selectedNode.locked;
    await applyTimelineNodeLockCommand({
      node_id: editorState.selectedNodeId,
      locked,
    });
    if (editorState.selectedNode) editorState.selectedNode.locked = locked;
  }

  async function handleGenerate() {
    if (!editorState.selectedNodeId || !editorState.selectedNode) return;
    if (editorState.selectedNode.locked) return;
    if (!editorState.selectedNode.content.notes.trim()) return;
    if (isGenerating) return;

    // If this node has children, generate all children instead.
    if (hasChildren) {
      startBatchGeneration(editorState.selectedNodeId);
      const result = await generateBatch(editorState.selectedNodeId);
      setBatchTotalCount(result.child_count);
      return;
    }

    startGeneration(editorState.selectedNodeId);
    editorState.selectedNode.content.status = 'Generating';
    await generateContent(editorState.selectedNodeId);
  }

  // Child planning — generates children and applies to timeline
  async function handleGenerateChildren() {
    if (!editorState.selectedNodeId || !editorState.selectedNode) return;
    planning = true;
    try {
      const plan = await generateChildren(editorState.selectedNodeId);
      await applyTimelineChildrenCommand({
        parent_id: editorState.selectedNodeId,
        children: plan.children.map((child) => ({
          node_id: crypto.randomUUID(),
          name: child.name,
          outline: child.outline,
          weight: child.weight,
          beat_type: child.beat_type,
        })),
      });
      const node = editorState.selectedNode;
      zoomToRange(node.time_range.start_ms, node.time_range.end_ms);
    } finally {
      planning = false;
    }
  }

  function navigateNode(direction: -1 | 1) {
    const targetIdx = currentNodeIndex + direction;
    if (targetIdx < 0 || targetIdx >= siblingNodes.length) return;
    const target = siblingNodes[targetIdx]!;
    editorState.selectedNodeId = target.id;
    editorState.selectedNode = target;
  }

  let childLevelName = $derived(
    editorState.selectedNode ? childLevel(editorState.selectedNode.level) : null,
  );
</script>

<div class="beat-editor">
  {#if editorState.selectedNode}
    {@const node = editorState.selectedNode}
    <div class="editor-header">
      <h3 class="clip-title">{node.name}</h3>
      <span class="level-badge">{node.level}</span>
      <span class="status-badge" data-status={node.content.status}>
        {statusLabel(node.content.status)}
      </span>
      <button class="lock-toggle" class:locked={node.locked} onclick={handleToggleLock}>
        {node.locked ? 'Unlock' : 'Lock'}
      </button>
      <button
        class="generate-btn"
        class:generating={isGenerating}
        onclick={handleGenerate}
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

    <!-- Child node: context panel with parent and sibling structure -->
    {#if isChildNode && parentNode}
      <div class="sub-beat-context">
        <div class="sub-beat-nav">
          <button class="nav-btn" disabled={currentNodeIndex <= 0} onclick={() => navigateNode(-1)}
            >&lsaquo; Prev</button
          >
          <span class="nav-position"
            >{node.level} {currentNodeIndex + 1} of {siblingNodes.length}</span
          >
          <button
            class="nav-btn"
            disabled={currentNodeIndex >= siblingNodes.length - 1}
            onclick={() => navigateNode(1)}>Next &rsaquo;</button
          >
        </div>

        <!-- Parent node notes -->
        <div class="context-card">
          <div class="context-card-header">{parentNode.level}: {parentNode.name}</div>
          <p class="context-card-body">{parentNode.content.notes || 'No notes'}</p>
        </div>

        <!-- Sibling structure -->
        {#if siblingNodes.length > 1}
          <details class="context-card" open>
            <summary class="context-card-header clickable">{node.level} Structure</summary>
            <div class="beat-structure">
              {#each siblingNodes as sibling, i}
                <button
                  class="beat-outline-item"
                  class:current={sibling.id === editorState.selectedNodeId}
                  onclick={() => {
                    editorState.selectedNodeId = sibling.id;
                    editorState.selectedNode = sibling;
                  }}
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

        <!-- Adjacent parent-level nodes -->
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
    {/if}

    <!-- Parent node: generate children actions -->
    {#if !isChildNode && childLevelName}
      <div class="beat-actions">
        <button
          class="plan-btn"
          onclick={handleGenerateChildren}
          disabled={planning || !node.content.notes.trim()}
        >
          {planning
            ? 'Planning...'
            : hasChildren
              ? `Replan ${childLevelName}s`
              : `Plan ${childLevelName}s`}
        </button>
      </div>
    {/if}

    <div class="editor-body">
      <label class="section-label" for={`notes-${node.id}`}>Notes</label>
      <textarea
        id={`notes-${node.id}`}
        class="notes-input"
        placeholder="Describe what happens in this {node.level.toLowerCase()}..."
        value={node.content.notes}
        oninput={handleNotesInput}
        disabled={node.locked}
      ></textarea>
      {#if isGenerating}
        <div class="section-label section-label-row">
          Generating
          <span class="token-count">{editorState.streamingTokenCount} tokens generated</span>
        </div>
      {/if}

      {#if editorState.generationError}
        <div class="error-banner">{editorState.generationError}</div>
      {/if}

      <!-- Raw AI prompt preview (debug) -->
      {#if node.content.notes.trim()}
        <details class="context-panel">
          <summary class="context-panel-summary">
            Raw AI Prompt
            <button
              class="context-refresh-btn"
              onclick={(e) => {
                e.stopPropagation();
                refreshContext();
              }}
              disabled={contextLoading}
            >
              {contextLoading ? 'Loading...' : 'Refresh'}
            </button>
          </summary>
          {#if nodeContext}
            <div class="context-display">
              <details>
                <summary class="context-heading">System Prompt</summary>
                <pre class="context-text">{nodeContext.system}</pre>
              </details>
              <details open>
                <summary class="context-heading">User Prompt</summary>
                <pre class="context-text">{nodeContext.user}</pre>
              </details>
            </div>
          {:else if contextLoading}
            <p class="context-loading">Loading context...</p>
          {:else}
            <p class="context-loading">No context available</p>
          {/if}
        </details>
      {/if}
    </div>
  {:else}
    <div class="empty-state">
      <p>Select a node on the timeline to edit</p>
    </div>
  {/if}
</div>

<style>
  .beat-editor {
    height: 100%;
    display: flex;
    flex-direction: column;
    padding: 12px 16px;
    overflow: auto;
  }

  .editor-header {
    display: flex;
    align-items: center;
    gap: 8px;
    margin-bottom: 12px;
    flex-shrink: 0;
  }

  .clip-title {
    margin: 0;
    font-size: 1.1rem;
    font-weight: 600;
    color: var(--color-text-primary);
  }

  .level-badge {
    font-size: 0.6rem;
    padding: 1px 6px;
    border-radius: 8px;
    background: var(--color-bg-hover);
    color: var(--color-text-muted);
    text-transform: uppercase;
    letter-spacing: 0.05em;
  }

  .status-badge {
    font-size: 0.7rem;
    padding: 2px 8px;
    border-radius: 10px;
    background: var(--color-bg-surface);
    color: var(--color-text-secondary);
  }

  .status-badge[data-status='NotesOnly'] {
    background: var(--color-status-notes);
    color: var(--color-text-on-light);
  }
  .status-badge[data-status='Generating'] {
    background: var(--color-status-generating);
    color: var(--color-text-on-dark);
  }
  .status-badge[data-status='HasContent'] {
    background: var(--color-status-generated);
    color: var(--color-text-on-light);
  }

  .lock-toggle {
    font-size: 0.75rem;
    padding: 2px 10px;
    border-radius: 10px;
    border: 1px solid var(--color-border-default);
    background: var(--color-bg-surface);
    color: var(--color-text-secondary);
    cursor: pointer;
  }

  .lock-toggle.locked {
    background: var(--color-bg-hover);
    color: var(--color-text-primary);
  }

  .generate-btn {
    font-size: 0.75rem;
    padding: 2px 10px;
    border-radius: 10px;
    border: 1px solid var(--color-accent);
    background: var(--color-bg-surface);
    color: var(--color-accent);
    cursor: pointer;
    margin-left: auto;
    transition:
      background 0.15s,
      color 0.15s;
  }

  .generate-btn:hover:not(:disabled) {
    background: var(--color-accent);
    color: var(--color-text-on-dark);
  }

  .generate-btn:disabled {
    opacity: 0.5;
    cursor: not-allowed;
  }

  .generate-btn.generating {
    animation: pulse 1.5s ease-in-out infinite;
  }

  @keyframes pulse {
    0%,
    100% {
      opacity: 1;
    }
    50% {
      opacity: 0.5;
    }
  }

  .editor-body {
    flex: 1;
    display: flex;
    flex-direction: column;
    gap: 8px;
  }

  .section-label {
    font-size: 0.75rem;
    color: var(--color-text-muted);
    text-transform: uppercase;
    letter-spacing: 0.05em;
    display: flex;
    align-items: center;
    gap: 8px;
  }

  .token-count {
    font-size: 0.65rem;
    color: var(--color-text-muted);
    text-transform: none;
    letter-spacing: normal;
  }

  .context-display {
    flex: 1;
    overflow: auto;
    display: flex;
    flex-direction: column;
    gap: 8px;
    min-height: 0;
  }

  .context-heading {
    font-size: 0.7rem;
    font-weight: 600;
    color: var(--color-text-secondary);
    text-transform: uppercase;
    letter-spacing: 0.05em;
    cursor: pointer;
    user-select: none;
  }

  .context-text {
    font-family: 'Courier New', monospace;
    font-size: 0.75rem;
    line-height: 1.4;
    color: var(--color-text-secondary);
    background: var(--color-bg-surface);
    border: 1px solid var(--color-border-subtle);
    border-radius: 4px;
    padding: 8px 12px;
    margin: 0;
    white-space: pre-wrap;
    word-wrap: break-word;
    overflow: auto;
    max-height: 300px;
  }

  .context-loading {
    font-size: 0.8rem;
    color: var(--color-text-muted);
    font-style: italic;
    margin: 8px 0;
    animation: pulse 1.5s ease-in-out infinite;
  }

  .context-panel {
    margin-top: 4px;
    border: 1px solid var(--color-border-subtle);
    border-radius: 4px;
    background: var(--color-bg-surface);
  }

  .context-panel-summary {
    font-size: 0.75rem;
    font-weight: 600;
    color: var(--color-text-secondary);
    text-transform: uppercase;
    letter-spacing: 0.05em;
    cursor: pointer;
    user-select: none;
    padding: 6px 10px;
    display: flex;
    align-items: center;
    gap: 8px;
  }

  .context-panel-summary:hover {
    color: var(--color-text-primary);
  }

  .context-refresh-btn {
    font-size: 0.6rem;
    padding: 1px 6px;
    border-radius: 6px;
    border: 1px solid var(--color-border-default);
    background: var(--color-bg-primary);
    color: var(--color-text-muted);
    cursor: pointer;
    margin-left: auto;
  }

  .context-refresh-btn:hover:not(:disabled) {
    background: var(--color-bg-hover);
    color: var(--color-text-secondary);
  }

  .context-refresh-btn:disabled {
    opacity: 0.5;
    cursor: not-allowed;
  }

  .context-panel > .context-display,
  .context-panel > .context-loading {
    padding: 0 10px 8px;
  }

  .notes-input {
    width: 100%;
    min-height: 100px;
    padding: 8px 12px;
    background: var(--color-bg-surface);
    color: var(--color-text-primary);
    border: 1px solid var(--color-border-default);
    border-radius: 4px;
    font-family: inherit;
    font-size: 0.9rem;
    resize: vertical;
  }

  .notes-input:focus {
    outline: none;
    border-color: var(--color-accent);
  }

  .notes-input:disabled {
    opacity: 0.6;
    cursor: not-allowed;
  }

  .error-banner {
    padding: 8px 12px;
    background: var(--color-danger-bg);
    border: 1px solid var(--color-danger-border);
    border-radius: 4px;
    color: var(--color-danger-light);
    font-size: 0.85rem;
  }

  .sub-beat-context {
    display: flex;
    flex-direction: column;
    gap: 6px;
    margin-bottom: 8px;
  }

  .sub-beat-nav {
    display: flex;
    align-items: center;
    gap: 8px;
  }

  .nav-btn {
    font-size: 0.7rem;
    padding: 2px 8px;
    border-radius: 4px;
    border: 1px solid var(--color-border-default);
    background: var(--color-bg-surface);
    color: var(--color-text-secondary);
    cursor: pointer;
  }

  .nav-btn:hover:not(:disabled) {
    background: var(--color-bg-hover);
  }

  .nav-btn:disabled {
    opacity: 0.4;
    cursor: not-allowed;
  }

  .nav-position {
    font-size: 0.7rem;
    color: var(--color-text-muted);
  }

  .context-card {
    background: var(--color-bg-surface);
    border: 1px solid var(--color-border-subtle);
    border-radius: 4px;
    padding: 6px 8px;
  }

  .context-card-header {
    font-size: 0.7rem;
    font-weight: 600;
    color: var(--color-text-secondary);
    text-transform: uppercase;
    letter-spacing: 0.03em;
    user-select: none;
  }

  .context-card-header.clickable {
    cursor: pointer;
  }

  .context-card-header.clickable:hover {
    color: var(--color-text-primary);
  }

  .context-card-body {
    font-size: 0.75rem;
    color: var(--color-text-muted);
    margin: 4px 0 0;
    white-space: pre-wrap;
    line-height: 1.4;
  }

  .beat-structure {
    display: flex;
    flex-direction: column;
    gap: 2px;
    margin-top: 4px;
  }

  .beat-outline-item {
    display: flex;
    align-items: flex-start;
    gap: 6px;
    padding: 3px 6px;
    border-radius: 3px;
    border: 1px solid transparent;
    background: none;
    text-align: left;
    cursor: pointer;
    font-family: inherit;
    color: var(--color-text-secondary);
    width: 100%;
  }

  .beat-outline-item:hover {
    background: var(--color-bg-hover);
  }

  .beat-outline-item.current {
    background: rgba(var(--color-accent-rgb, 99, 102, 241), 0.1);
    border-color: var(--color-accent);
  }

  .beat-outline-number {
    font-size: 0.6rem;
    font-weight: 700;
    color: var(--color-text-muted);
    min-width: 14px;
    text-align: center;
    padding-top: 1px;
    flex-shrink: 0;
  }

  .beat-outline-info {
    display: flex;
    flex-direction: column;
    gap: 1px;
    min-width: 0;
  }

  .beat-outline-name {
    font-size: 0.7rem;
    font-weight: 600;
    color: var(--color-text-primary);
    white-space: nowrap;
    overflow: hidden;
    text-overflow: ellipsis;
  }

  .beat-outline-type {
    font-weight: 400;
    color: var(--color-text-muted);
    font-size: 0.6rem;
  }

  .beat-outline-text {
    font-size: 0.65rem;
    color: var(--color-text-muted);
    line-height: 1.3;
    display: -webkit-box;
    -webkit-line-clamp: 2;
    line-clamp: 2;
    -webkit-box-orient: vertical;
    overflow: hidden;
  }

  .adjacent-scenes {
    display: flex;
    flex-direction: column;
    gap: 6px;
    margin-top: 4px;
  }

  .adjacent-scene {
    display: flex;
    flex-direction: column;
    gap: 1px;
  }

  .adjacent-scene-label {
    font-size: 0.55rem;
    font-weight: 700;
    color: var(--color-text-muted);
    text-transform: uppercase;
    letter-spacing: 0.5px;
  }

  .adjacent-scene-name {
    font-size: 0.7rem;
    font-weight: 600;
    color: var(--color-text-primary);
  }

  .adjacent-scene-notes {
    font-size: 0.65rem;
    color: var(--color-text-muted);
    margin: 0;
    line-height: 1.3;
    white-space: pre-wrap;
    display: -webkit-box;
    -webkit-line-clamp: 3;
    line-clamp: 3;
    -webkit-box-orient: vertical;
    overflow: hidden;
  }

  .beat-actions {
    display: flex;
    gap: 8px;
    margin-bottom: 8px;
  }

  .plan-btn {
    font-size: 0.75rem;
    padding: 3px 12px;
    border-radius: 10px;
    border: 1px solid var(--color-accent);
    background: var(--color-bg-surface);
    color: var(--color-accent);
    cursor: pointer;
  }

  .plan-btn:hover:not(:disabled) {
    background: var(--color-accent);
    color: var(--color-text-on-dark);
  }

  .plan-btn:disabled {
    opacity: 0.5;
    cursor: not-allowed;
  }

  .empty-state {
    flex: 1;
    display: flex;
    align-items: center;
    justify-content: center;
    color: var(--color-text-muted);
  }
</style>
