<script lang="ts">
  import type { YTextEvent } from 'yjs';
  import type { NodeId, StoryNode } from '$lib/timelineTypes.js';
  import type {
    SelectedNodeEditorNode,
    SelectedNodeEditorSummary,
  } from '$lib/selectedNodeEditorTypes.js';
  import {
    editorState,
    startBatchGeneration,
    startGeneration,
    setBatchTotalCount,
  } from '$lib/stores/editor.svelte.js';
  import { zoomToRange } from '$lib/stores/timeline.svelte.js';
  import { generateBatch, generateChildren, generateContent, getAiContext } from '$lib/api.js';
  import {
    applyTimelineChildrenCommand,
    applyTimelineNodeLockCommand,
    applyTimelineNodeNotesCommand,
  } from '$lib/stores/timelineRenderProjection.svelte.js';
  import {
    refreshSelectedNodeEditorProjection,
    selectedNodeEditorProjectionState,
  } from '$lib/stores/selectedNodeEditorProjection.svelte.js';
  import { getNodeNotes } from '$lib/yjs.js';
  import BeatChildContext from './BeatChildContext.svelte';
  import BeatEditorHeader from './BeatEditorHeader.svelte';
  import BeatNotesPanel from './BeatNotesPanel.svelte';
  import BeatPlanningActions from './BeatPlanningActions.svelte';
  import { beatContentStatusLabel } from './beatEditorStatus.js';
  import './beatEditor.css';

  let debounceTimer: ReturnType<typeof setTimeout> | null = null;
  let planning = $state(false);
  let nodeContext: { system: string; user: string } | null = $state(null);
  let contextLoading = $state(false);
  let contextNodeId: string | null = $state(null);

  let isGenerating = $derived(
    (editorState.streamingNodeId != null &&
      editorState.streamingNodeId === editorState.selectedNodeId) ||
      (editorState.batchParentNodeId != null &&
        editorState.batchParentNodeId === editorState.selectedNodeId),
  );
  let selectedProjection = $derived(selectedNodeEditorProjectionState.projection?.payload ?? null);
  let selectedProjectionNode = $derived(selectedProjection?.node ?? null);
  let node = $derived(
    selectedProjectionNode ? editorProjectionNodeToStoryNode(selectedProjectionNode) : null,
  );
  let isChildNode = $derived(node?.parent_id != null);
  let hasChildren = $derived(selectedProjection?.has_children ?? false);
  let parentNode = $derived(
    selectedProjection?.parent ? editorSummaryToStoryNode(selectedProjection.parent) : null,
  );
  let siblingNodes = $derived(selectedProjection?.siblings.map(editorSummaryToStoryNode) ?? []);
  let currentNodeIndex = $derived(selectedProjection?.current_sibling_index ?? -1);
  let adjacentParents = $derived({
    before: selectedProjection?.adjacent_parents.before
      ? editorSummaryToStoryNode(selectedProjection.adjacent_parents.before)
      : null,
    after: selectedProjection?.adjacent_parents.after
      ? editorSummaryToStoryNode(selectedProjection.adjacent_parents.after)
      : null,
  });
  let childLevelName = $derived(selectedProjection?.child_level ?? null);
  let selectedNotes = $derived(selectedProjectionNode?.notes ?? '');

  $effect(() => {
    if (
      editorState.selectedNodeId !== selectedNodeEditorProjectionState.selectedNodeId &&
      !selectedNodeEditorProjectionState.pending
    ) {
      void refreshSelectedNodeEditorProjection(editorState.selectedNodeId).catch(() => {});
    }
  });

  $effect(() => {
    if (
      selectedProjectionNode &&
      editorState.selectedNodeId &&
      selectedProjectionNode.node_id !== editorState.selectedNodeId
    ) {
      void refreshSelectedNodeEditorProjection(editorState.selectedNodeId).catch(() => {});
    }
  });

  $effect(() => {
    if (!editorState.selectedNodeId) {
      void refreshSelectedNodeEditorProjection(null).catch(() => {});
    }
  });

  $effect(() => {
    if (selectedProjectionNode && selectedProjectionNode.node_id === editorState.selectedNodeId) {
      editorState.selectedLevel = selectedProjectionNode.level;
    }
  });

  function editorProjectionNodeToStoryNode(editorNode: SelectedNodeEditorNode): StoryNode {
    return {
      id: editorNode.node_id,
      parent_id: editorNode.parent_id ?? null,
      level: editorNode.level,
      sort_order: editorNode.sort_order,
      time_range: {
        start_ms: editorNode.start_ms,
        end_ms: editorNode.end_ms,
      },
      name: editorNode.name,
      content: {
        notes: editorNode.notes,
        content: '',
        status: editorNode.content_status,
      },
      beat_type: editorNode.beat_type ?? null,
      locked: editorNode.locked,
    };
  }

  function editorSummaryToStoryNode(summary: SelectedNodeEditorSummary): StoryNode {
    return {
      id: summary.node_id,
      parent_id: summary.parent_id ?? null,
      level: summary.level,
      sort_order: summary.sort_order,
      time_range: {
        start_ms: summary.start_ms,
        end_ms: summary.end_ms,
      },
      name: summary.name,
      content: {
        notes: summary.notes,
        content: '',
        status: 'NotesOnly',
      },
      beat_type: summary.beat_type ?? null,
      locked: false,
    };
  }

  function selectNodeById(nodeId: NodeId) {
    editorState.selectedNodeId = nodeId;
    void refreshSelectedNodeEditorProjection(nodeId).catch(() => {});
  }

  function selectNode(node: StoryNode) {
    editorState.selectedLevel = node.level;
    selectNodeById(node.id);
  }

  async function refreshSelectedProjection() {
    await refreshSelectedNodeEditorProjection(editorState.selectedNodeId);
  }

  function selectedNodeIsReady(): boolean {
    return (
      selectedProjectionNode != null &&
      selectedProjectionNode.node_id === editorState.selectedNodeId
    );
  }

  function selectedStoryNodeIsReady(): boolean {
    return node != null && node.id === editorState.selectedNodeId;
  }

  function selectedNodeHasNotes(): boolean {
    return selectedNotes.trim().length > 0;
  }

  function selectedNodeIsLocked(): boolean {
    return selectedProjectionNode?.locked ?? false;
  }

  function selectedNodeRange() {
    return selectedProjectionNode
      ? { start_ms: selectedProjectionNode.start_ms, end_ms: selectedProjectionNode.end_ms }
      : null;
  }

  function selectedNodeContentStatus() {
    if (editorState.streamingNodeId === editorState.selectedNodeId) return 'Generating';
    return selectedProjectionNode?.content_status ?? 'Empty';
  }

  function selectedNodeForRender(): StoryNode | null {
    if (!node) return null;
    return {
      ...node,
      content: {
        ...node.content,
        status: selectedNodeContentStatus(),
      },
    };
  }

  let renderNode = $derived(selectedNodeForRender());

  let selectedNodeStatusLabel = $derived(
    beatContentStatusLabel(renderNode?.content.status ?? 'Empty'),
  );

  $effect(() => {
    const nodeId = editorState.selectedNodeId;
    const notes = selectedProjectionNode?.notes;
    if (!nodeId || !notes?.trim()) {
      nodeContext = null;
      contextNodeId = null;
      return;
    }
    if (nodeId === contextNodeId && nodeContext) return;
    loadContext(nodeId);
  });

  $effect(() => {
    const nodeId = editorState.selectedNodeId;
    if (!nodeId) return;

    const yNotes = getNodeNotes(nodeId);
    const onNotesChange = (_event: YTextEvent) => {
      if (editorState.selectedNodeId === nodeId) {
        void refreshSelectedNodeEditorProjection(nodeId).catch(() => {});
      }
    };

    yNotes.observe(onNotesChange);
    return () => yNotes.unobserve(onNotesChange);
  });

  function loadContext(nodeId: string) {
    contextLoading = true;
    contextNodeId = nodeId;
    getAiContext(nodeId)
      .then((context) => {
        if (editorState.selectedNodeId === nodeId) nodeContext = context;
      })
      .catch(() => {
        if (editorState.selectedNodeId === nodeId) nodeContext = null;
      })
      .finally(() => {
        if (editorState.selectedNodeId === nodeId) contextLoading = false;
      });
  }

  function refreshContext() {
    if (editorState.selectedNodeId) loadContext(editorState.selectedNodeId);
  }

  function handleNotesInput(event: Event) {
    const value = (event.target as HTMLTextAreaElement).value;
    if (debounceTimer) clearTimeout(debounceTimer);
    debounceTimer = setTimeout(async () => {
      if (editorState.selectedNodeId) {
        await applyTimelineNodeNotesCommand({
          node_id: editorState.selectedNodeId,
          notes: value,
        });
        await refreshSelectedProjection();
      }
    }, 500);
  }

  async function handleToggleLock() {
    if (!editorState.selectedNodeId || !selectedNodeIsReady()) return;
    const selectedEditorNode = selectedProjectionNode;
    if (!selectedEditorNode) return;
    const locked = !selectedEditorNode.locked;
    await applyTimelineNodeLockCommand({ node_id: editorState.selectedNodeId, locked });
    await refreshSelectedProjection();
  }

  async function handleGenerate() {
    if (!editorState.selectedNodeId || !selectedNodeIsReady()) return;
    if (selectedNodeIsLocked() || !selectedNodeHasNotes()) return;
    if (isGenerating) return;

    if (hasChildren) {
      startBatchGeneration(editorState.selectedNodeId);
      const result = await generateBatch(editorState.selectedNodeId);
      setBatchTotalCount(result.child_count);
      return;
    }

    startGeneration(editorState.selectedNodeId);
    await generateContent(editorState.selectedNodeId);
  }

  async function handleGenerateChildren() {
    if (!editorState.selectedNodeId || !selectedStoryNodeIsReady()) return;
    const parentNodeId = editorState.selectedNodeId;
    const selectedRange = selectedNodeRange();
    planning = true;
    try {
      const plan = await generateChildren(parentNodeId);
      if (plan.parent_node_id !== parentNodeId) {
        throw new Error('Generated child plan parent did not match the selected node');
      }
      await applyTimelineChildrenCommand({
        parent_id: parentNodeId,
        child_plan_id: plan.id,
        children: plan.children.map((child) => ({
          node_id: crypto.randomUUID(),
          name: child.name,
          outline: child.outline,
          weight: child.weight,
          beat_type: child.beat_type,
          characters: child.characters ?? [],
          location: child.location ?? null,
          props: child.props ?? [],
        })),
      });
      await refreshSelectedProjection();
      if (selectedRange) {
        zoomToRange(selectedRange.start_ms, selectedRange.end_ms);
      }
    } finally {
      planning = false;
    }
  }

  function navigateNode(direction: -1 | 1) {
    const targetIndex = currentNodeIndex + direction;
    if (targetIndex < 0 || targetIndex >= siblingNodes.length) return;
    selectNode(siblingNodes[targetIndex]!);
  }
</script>

<div class="beat-editor">
  {#if renderNode}
    {@const node = renderNode}
    <BeatEditorHeader
      {node}
      statusLabel={selectedNodeStatusLabel}
      {isGenerating}
      {hasChildren}
      {childLevelName}
      ontogglelock={handleToggleLock}
      ongenerate={handleGenerate}
    />

    {#if isChildNode && parentNode}
      <BeatChildContext
        {node}
        selectedNodeId={editorState.selectedNodeId}
        {parentNode}
        {siblingNodes}
        {currentNodeIndex}
        {adjacentParents}
        onnavigate={navigateNode}
        onselectnode={selectNode}
      />
    {/if}

    {#if !isChildNode && childLevelName}
      <BeatPlanningActions
        {childLevelName}
        {hasChildren}
        {planning}
        notes={node.content.notes}
        onplan={handleGenerateChildren}
      />
    {/if}

    <BeatNotesPanel
      {node}
      {isGenerating}
      streamingTokenCount={editorState.streamingTokenCount}
      generationError={editorState.generationError}
      {nodeContext}
      {contextLoading}
      onnotesinput={handleNotesInput}
      onrefreshcontext={refreshContext}
    />
  {:else}
    <div class="empty-state">
      <p>Select a node on the timeline to edit</p>
    </div>
  {/if}
</div>
