<script lang="ts">
  import type { YTextEvent } from 'yjs';
  import { childLevel } from '$lib/timelineTypes.js';
  import type { StoryNode } from '$lib/timelineTypes.js';
  import {
    editorState,
    startBatchGeneration,
    startGeneration,
    setBatchTotalCount,
  } from '$lib/stores/editor.svelte.js';
  import { childrenOf, findNode, timelineState, zoomToRange } from '$lib/stores/timeline.svelte.js';
  import { generateBatch, generateChildren, generateContent, getAiContext } from '$lib/api.js';
  import {
    applyTimelineChildrenCommand,
    applyTimelineNodeLockCommand,
    applyTimelineNodeNotesCommand,
  } from '$lib/stores/timelineRenderProjection.svelte.js';
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
  let isChildNode = $derived(editorState.selectedNode?.parent_id != null);
  let childNodes = $derived.by(() =>
    editorState.selectedNodeId ? childrenOf(editorState.selectedNodeId) : [],
  );
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
    siblingNodes.findIndex((node) => node.id === editorState.selectedNodeId),
  );
  let adjacentParents = $derived.by(() => {
    if (!isChildNode || !parentNode) return { before: null, after: null };
    const allAtLevel =
      timelineState.timeline?.nodes
        .filter((node) => node.level === parentNode.level)
        .sort((a, b) => a.time_range.start_ms - b.time_range.start_ms) ?? [];
    const index = allAtLevel.findIndex((node) => node.id === parentNode.id);
    if (index < 0) return { before: null, after: null };
    return {
      before: index > 0 ? (allAtLevel[index - 1] ?? null) : null,
      after: index < allAtLevel.length - 1 ? (allAtLevel[index + 1] ?? null) : null,
    };
  });
  let childLevelName = $derived(
    editorState.selectedNode ? childLevel(editorState.selectedNode.level) : null,
  );

  $effect(() => {
    const nodeId = editorState.selectedNodeId;
    const notes = editorState.selectedNode?.content.notes;
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
      if (editorState.selectedNode && editorState.selectedNodeId === nodeId) {
        editorState.selectedNode.content.notes = yNotes.toString();
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

  function selectNode(node: StoryNode) {
    editorState.selectedNodeId = node.id;
    editorState.selectedNode = node;
  }

  function handleNotesInput(event: Event) {
    const value = (event.target as HTMLTextAreaElement).value;
    if (editorState.selectedNode) editorState.selectedNode.content.notes = value;
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
    await applyTimelineNodeLockCommand({ node_id: editorState.selectedNodeId, locked });
    if (editorState.selectedNode) editorState.selectedNode.locked = locked;
  }

  async function handleGenerate() {
    if (!editorState.selectedNodeId || !editorState.selectedNode) return;
    if (editorState.selectedNode.locked || !editorState.selectedNode.content.notes.trim()) return;
    if (isGenerating) return;

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

  async function handleGenerateChildren() {
    if (!editorState.selectedNodeId || !editorState.selectedNode) return;
    const parentNodeId = editorState.selectedNodeId;
    const selectedParent = editorState.selectedNode;
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
      zoomToRange(selectedParent.time_range.start_ms, selectedParent.time_range.end_ms);
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
  {#if editorState.selectedNode}
    {@const node = editorState.selectedNode}
    <BeatEditorHeader
      {node}
      statusLabel={beatContentStatusLabel(node.content.status)}
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
