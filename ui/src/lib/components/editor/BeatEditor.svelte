<script lang="ts">
  import { onDestroy } from 'svelte';
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
    getEditorSessionGeneration,
    setGenerationError,
  } from '$lib/stores/editor.svelte.js';
  import { zoomToRange } from '$lib/stores/timeline.svelte.js';
  import { getChildPlanListProjection } from '$lib/projectionApi.js';
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
  import BeatChildContext from './BeatChildContext.svelte';
  import BeatEditorHeader from './BeatEditorHeader.svelte';
  import TimelinePlacementEditor from './TimelinePlacementEditor.svelte';
  import TimelineTitleEditor from './TimelineTitleEditor.svelte';
  import BeatNotesPanel from './BeatNotesPanel.svelte';
  import BeatPlanningActions from './BeatPlanningActions.svelte';
  import ChildPlanReview from './ChildPlanReview.svelte';
  import { createChildPlanReview } from './childPlanReview.svelte.js';
  import { beatContentStatusLabel } from './beatEditorStatus.js';
  import { createDebouncedNodeNotesSave } from './debouncedNodeNotesSave.js';
  import { createContextRequestLifecycle } from './contextRequestLifecycle.js';
  import './beatEditor.css';
  import { createSelectedTimelineChild } from './createSelectedTimelineChild.js';
  import { scriptDocumentProjectionState } from '$lib/stores/scriptDocumentProjection.svelte.js';

  const debouncedNotesSave = createDebouncedNodeNotesSave({
    delayMs: 500,
    async save(nodeId, notes) {
      await applyTimelineNodeNotesCommand({
        node_id: nodeId,
        notes,
      });
      if (editorState.selectedNodeId === nodeId) {
        await refreshSelectedProjection();
      }
    },
  });
  const childPlanReview = createChildPlanReview({
    owner: () => ({ nodeId: editorState.selectedNodeId, session: getEditorSessionGeneration() }),
    mounted: () => editorMounted,
    generate: generateChildren,
    load: getChildPlanListProjection,
    apply: applyTimelineChildrenCommand,
    async accepted(parent) {
      await refreshSelectedProjection();
      if (editorMounted && editorState.selectedNodeId === parent) {
        const range = selectedNodeRange();
        if (range) zoomToRange(range.start_ms, range.end_ms);
      }
    },
  });
  $effect(() => childPlanReview.syncOwner());
  let creatingChild = $state(false);
  let childCreateError = $state<string | null>(null);
  let editorMounted = true;
  let nodeContext: { system: string; user: string } | null = $state(null);
  let contextLoading = $state(false);
  const contextRequests = createContextRequestLifecycle({
    selectedNodeId: () => editorState.selectedNodeId,
    fetchContext: getAiContext,
    setContext: (context) => (nodeContext = context),
    setLoading: (loading) => (contextLoading = loading),
  });

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
    const revision = scriptDocumentProjectionState.contextRevision;
    contextRequests.update(nodeId, notes, revision);
  });

  function loadContext(nodeId: string) {
    void contextRequests.load(nodeId);
  }

  function refreshContext() {
    if (editorState.selectedNodeId) loadContext(editorState.selectedNodeId);
  }

  function handleNotesInput(event: Event) {
    const nodeId = editorState.selectedNodeId;
    if (!nodeId) return;
    const value = (event.target as HTMLTextAreaElement).value;
    debouncedNotesSave.schedule(nodeId, value);
  }

  onDestroy(() => {
    editorMounted = false;
    debouncedNotesSave.dispose();
    contextRequests.invalidate();
  });

  async function handleAddChild() {
    if (!selectedNodeIsReady() || !childLevelName || creatingChild) return;
    const parentId = editorState.selectedNodeId;
    if (!parentId) return;
    const session = getEditorSessionGeneration();
    creatingChild = true;
    childCreateError = null;
    try {
      await createSelectedTimelineChild(parentId, () => editorMounted);
    } catch (error) {
      if (
        editorMounted &&
        session === getEditorSessionGeneration() &&
        editorState.selectedNodeId === parentId
      ) {
        childCreateError = error instanceof Error ? error.message : 'Unable to add child';
      }
    } finally {
      creatingChild = false;
    }
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
    const nodeId = editorState.selectedNodeId;
    const session = getEditorSessionGeneration();
    try {
      await generateContent(nodeId);
    } catch (error) {
      if (session === getEditorSessionGeneration()) {
        setGenerationError(nodeId, error instanceof Error ? error.message : 'Generation refused');
      }
    }
  }

  async function handleGenerateChildren() {
    if (!editorState.selectedNodeId || !selectedStoryNodeIsReady()) return;
    await childPlanReview.generate();
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
      onaddchild={handleAddChild}
      {creatingChild}
    />

    {#if selectedProjectionNode}
      <TimelineTitleEditor node={selectedProjectionNode} />
      <TimelinePlacementEditor node={selectedProjectionNode} />
    {/if}

    {#if childCreateError}<p role="alert">{childCreateError}</p>{/if}

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

    {#if childLevelName}
      <BeatPlanningActions
        {childLevelName}
        {hasChildren}
        planning={childPlanReview.state.busy || childPlanReview.state.uncertain}
        notes={node.content.notes}
        onplan={handleGenerateChildren}
      />
    {/if}

    <ChildPlanReview
      recoverable={childLevelName != null}
      savedPlans={childPlanReview.state.savedPlans}
      reading={childPlanReview.state.reading}
      onrecover={() => void childPlanReview.recover()}
      onreviewsaved={childPlanReview.reviewSaved}
      plan={childPlanReview.state.plan}
      busy={childPlanReview.state.busy}
      uncertain={childPlanReview.state.uncertain}
      error={childPlanReview.state.error}
      onaccept={() => void childPlanReview.accept()}
      onclose={childPlanReview.close}
    />

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
