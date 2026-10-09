<script lang="ts">
  import type { StoryNode } from '$lib/timelineTypes.js';
  import type { SelectedNodeEditorNode } from '$lib/selectedNodeEditorTypes.js';
  import TimelineNotesEditor from './TimelineNotesEditor.svelte';
  import AiPromptPreview from './AiPromptPreview.svelte';

  let {
    node,
    isGenerating,
    streamingTokenCount,
    generationError,
    nodeContext,
    contextLoading,
    editorNode,
    onrefreshcontext,
  }: {
    node: StoryNode;
    isGenerating: boolean;
    streamingTokenCount: number;
    generationError: string | null;
    nodeContext: { system: string; user: string } | null;
    contextLoading: boolean;
    editorNode: SelectedNodeEditorNode;
    onrefreshcontext: () => void;
  } = $props();
</script>

<div class="editor-body">
  <TimelineNotesEditor node={editorNode} />
  {#if isGenerating}
    <div class="section-label section-label-row">
      Generating
      <span class="token-count">{streamingTokenCount} tokens generated</span>
    </div>
  {/if}

  {#if generationError}
    <div class="error-banner">{generationError}</div>
  {/if}

  <AiPromptPreview
    notes={node.content.notes}
    context={nodeContext}
    loading={contextLoading}
    onrefresh={onrefreshcontext}
  />
</div>
