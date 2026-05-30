import type { NodeId, StoryLevel } from '../timelineTypes.js';

/**
 * Transient UI state for the node editor panel. Frontend-owned.
 */
export const editorState = $state<{
  selectedNodeId: NodeId | null;
  /** Selected hierarchy level for the editor panel. */
  selectedLevel: StoryLevel | null;
  /** Node ID currently streaming generation. */
  streamingNodeId: NodeId | null;
  /** Accumulated streaming text during generation. */
  streamingText: string;
  /** Number of tokens received so far. */
  streamingTokenCount: number;
  /** Formatted AI prompt context (system + user) for the active generation. */
  generationContext: { system: string; user: string } | null;
  /** Node ID that the generationContext belongs to. */
  lastGenerationNodeId: NodeId | null;
  /** Error from a failed generation. */
  generationError: string | null;
  /** Parent node ID during batch child generation. */
  batchParentNodeId: NodeId | null;
  /** Total number of children in batch generation. */
  batchTotalCount: number;
  /** Number of children completed in batch generation. */
  batchCompletedCount: number;
}>({
  selectedNodeId: null,
  selectedLevel: null,
  streamingNodeId: null,
  streamingText: '',
  streamingTokenCount: 0,
  generationContext: null,
  lastGenerationNodeId: null,
  generationError: null,
  batchParentNodeId: null,
  batchTotalCount: 0,
  batchCompletedCount: 0,
});

export function resetEditorState(): void {
  editorState.selectedNodeId = null;
  editorState.selectedLevel = null;
  editorState.streamingNodeId = null;
  editorState.streamingText = '';
  editorState.streamingTokenCount = 0;
  editorState.generationContext = null;
  editorState.lastGenerationNodeId = null;
  editorState.generationError = null;
  editorState.batchParentNodeId = null;
  editorState.batchTotalCount = 0;
  editorState.batchCompletedCount = 0;
}

/** Reset streaming state and begin a new generation for a single node. */
export function startGeneration(nodeId: string): void {
  editorState.streamingNodeId = nodeId;
  editorState.streamingText = '';
  editorState.streamingTokenCount = 0;
  editorState.generationContext = null;
  editorState.generationError = null;
  editorState.batchParentNodeId = null;
}

/** Start batch generation tracking for all children of a parent node. */
export function startBatchGeneration(parentNodeId: string): void {
  editorState.batchParentNodeId = parentNodeId;
  editorState.batchTotalCount = 0;
  editorState.batchCompletedCount = 0;
  editorState.streamingNodeId = null;
  editorState.streamingText = '';
  editorState.streamingTokenCount = 0;
  editorState.generationContext = null;
  editorState.generationError = null;
}

/** Update the expected batch total count (called after API response). */
export function setBatchTotalCount(count: number): void {
  editorState.batchTotalCount = count;
  if (editorState.batchCompletedCount >= count && count > 0) {
    editorState.batchParentNodeId = null;
  }
}

/** Append a streaming token if it matches the active generation. */
export function appendStreamingToken(nodeId: string, token: string, count: number): void {
  if (editorState.streamingNodeId === nodeId) {
    editorState.streamingText += token;
    editorState.streamingTokenCount = count;
  } else if (editorState.batchParentNodeId != null) {
    editorState.streamingNodeId = nodeId;
    editorState.streamingText = token;
    editorState.streamingTokenCount = count;
    editorState.generationContext = null;
  }
}

/** Finalize a completed generation. */
export function completeGeneration(nodeId: string): void {
  if (editorState.streamingNodeId === nodeId) {
    editorState.streamingNodeId = null;
    editorState.streamingText = '';
  }
  if (editorState.batchParentNodeId != null) {
    editorState.batchCompletedCount++;
    if (
      editorState.batchCompletedCount >= editorState.batchTotalCount &&
      editorState.batchTotalCount > 0
    ) {
      editorState.batchParentNodeId = null;
    }
  }
}

/** Store the AI generation context (formatted prompts). */
export function setGenerationContext(nodeId: string, system: string, user: string): void {
  if (editorState.batchParentNodeId != null && editorState.streamingNodeId !== nodeId) {
    editorState.streamingNodeId = nodeId;
    editorState.streamingText = '';
    editorState.streamingTokenCount = 0;
  }
  if (editorState.streamingNodeId === nodeId) {
    editorState.generationContext = { system, user };
    editorState.lastGenerationNodeId = nodeId;
  }
}

/** Record a generation error. */
export function setGenerationError(nodeId: string, error: string): void {
  if (editorState.streamingNodeId === nodeId) {
    editorState.generationError = error;
    editorState.streamingNodeId = null;
    editorState.streamingText = '';
  }
  if (editorState.batchParentNodeId != null) {
    editorState.batchCompletedCount++;
    if (
      editorState.batchCompletedCount >= editorState.batchTotalCount &&
      editorState.batchTotalCount > 0
    ) {
      editorState.batchParentNodeId = null;
    }
  }
}
