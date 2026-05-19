<script lang="ts">
  import type { StoryNode } from '$lib/timelineTypes.js';
  import AiPromptPreview from './AiPromptPreview.svelte';

  let {
    node,
    isGenerating,
    streamingTokenCount,
    generationError,
    nodeContext,
    contextLoading,
    onnotesinput,
    onrefreshcontext,
  }: {
    node: StoryNode;
    isGenerating: boolean;
    streamingTokenCount: number;
    generationError: string | null;
    nodeContext: { system: string; user: string } | null;
    contextLoading: boolean;
    onnotesinput: (event: Event) => void;
    onrefreshcontext: () => void;
  } = $props();
</script>

<div class="editor-body">
  <label class="section-label" for={`notes-${node.id}`}>Notes</label>
  <textarea
    id={`notes-${node.id}`}
    class="notes-input"
    placeholder="Describe what happens in this {node.level.toLowerCase()}..."
    value={node.content.notes}
    oninput={onnotesinput}
    disabled={node.locked}
  ></textarea>
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
