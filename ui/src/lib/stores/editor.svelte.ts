import type { NodeId, StoryNode, StoryLevel, AiStatus, ConsistencySuggestion, DiffusionStatus } from '../types.js';

/**
 * Transient UI state for the node editor panel. Frontend-owned.
 */
export const editorState = $state<{
	selectedNodeId: NodeId | null;
	selectedNode: StoryNode | null;
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
	/** Last-known AI backend status. */
	aiStatus: AiStatus | null;
	/** Last-known diffusion LLM status. */
	diffusionStatus: DiffusionStatus | null;
	/** Pending consistency suggestions from the AI. */
	consistencySuggestions: ConsistencySuggestion[];
	/** Whether a consistency check is in progress. */
	checkingConsistency: boolean;
	/** Whether undo is available. */
	canUndo: boolean;
	/** Whether redo is available. */
	canRedo: boolean;
	/** Parent node ID during batch child generation. */
	batchParentNodeId: NodeId | null;
	/** Total number of children in batch generation. */
	batchTotalCount: number;
	/** Number of children completed in batch generation. */
	batchCompletedCount: number;
}>({
	selectedNodeId: null,
	selectedNode: null,
	selectedLevel: null,
	streamingNodeId: null,
	streamingText: '',
	streamingTokenCount: 0,
	generationContext: null,
	lastGenerationNodeId: null,
	generationError: null,
	aiStatus: null,
	diffusionStatus: null,
	consistencySuggestions: [],
	checkingConsistency: false,
	canUndo: false,
	canRedo: false,
	batchParentNodeId: null,
	batchTotalCount: 0,
	batchCompletedCount: 0,
});

/** Reset streaming state and begin a new generation for a single node. */
export function startGeneration(nodeId: string) {
	editorState.streamingNodeId = nodeId;
	editorState.streamingText = '';
	editorState.streamingTokenCount = 0;
	editorState.generationContext = null;
	editorState.generationError = null;
	editorState.batchParentNodeId = null;
}

/** Start batch generation tracking for all children of a parent node. */
export function startBatchGeneration(parentNodeId: string) {
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
export function setBatchTotalCount(count: number) {
	editorState.batchTotalCount = count;
	if (editorState.batchCompletedCount >= count && count > 0) {
		editorState.batchParentNodeId = null;
	}
}

/** Append a streaming token if it matches the active generation. */
export function appendStreamingToken(nodeId: string, token: string, count: number) {
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
export function completeGeneration(nodeId: string) {
	if (editorState.streamingNodeId === nodeId) {
		editorState.streamingNodeId = null;
		editorState.streamingText = '';
	}
	if (editorState.batchParentNodeId != null) {
		editorState.batchCompletedCount++;
		if (editorState.batchCompletedCount >= editorState.batchTotalCount && editorState.batchTotalCount > 0) {
			editorState.batchParentNodeId = null;
		}
	}
}

/** Store the AI generation context (formatted prompts). */
export function setGenerationContext(nodeId: string, system: string, user: string) {
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
export function setGenerationError(nodeId: string, error: string) {
	if (editorState.streamingNodeId === nodeId) {
		editorState.generationError = error;
		editorState.streamingNodeId = null;
		editorState.streamingText = '';
	}
	if (editorState.batchParentNodeId != null) {
		editorState.batchCompletedCount++;
		if (editorState.batchCompletedCount >= editorState.batchTotalCount && editorState.batchTotalCount > 0) {
			editorState.batchParentNodeId = null;
		}
	}
}

/** Add a consistency suggestion from the AI. */
export function addConsistencySuggestion(suggestion: ConsistencySuggestion) {
	editorState.consistencySuggestions = [
		...editorState.consistencySuggestions,
		suggestion,
	];
}

/** Remove a consistency suggestion by target node ID. */
export function removeConsistencySuggestion(targetNodeId: string) {
	editorState.consistencySuggestions = editorState.consistencySuggestions.filter(
		(s) => s.target_node_id !== targetNodeId
	);
}

/** Clear all consistency suggestions. */
export function clearConsistencySuggestions() {
	editorState.consistencySuggestions = [];
	editorState.checkingConsistency = false;
}
