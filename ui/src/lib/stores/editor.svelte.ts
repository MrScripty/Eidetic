import type { ClipId, BeatClip, AiStatus, ConsistencySuggestion } from '../types.js';

/**
 * Transient UI state for the beat editor panel. Frontend-owned.
 *
 * TODO: Split into focused stores (selectionState, generationState, aiState)
 * once all consumers have been migrated. Tracked in Sprint 2.
 */
export const editorState = $state<{
	selectedClipId: ClipId | null;
	selectedClip: BeatClip | null;
	/** Clip ID currently streaming generation. */
	streamingClipId: ClipId | null;
	/** Accumulated streaming text during generation. */
	streamingText: string;
	/** Number of tokens received so far. */
	streamingTokenCount: number;
	/** Formatted AI prompt context (system + user) for the active generation. */
	generationContext: { system: string; user: string } | null;
	/** Error from a failed generation. */
	generationError: string | null;
	/** Last-known AI backend status. */
	aiStatus: AiStatus | null;
	/** Pending consistency suggestions from the AI. */
	consistencySuggestions: ConsistencySuggestion[];
	/** Whether a consistency check is in progress. */
	checkingConsistency: boolean;
	/** Whether undo is available. */
	canUndo: boolean;
	/** Whether redo is available. */
	canRedo: boolean;
	/** Parent clip ID during batch beat generation. */
	batchParentClipId: ClipId | null;
	/** Total number of beats in batch generation. */
	batchTotalCount: number;
	/** Number of beats completed in batch generation. */
	batchCompletedCount: number;
}>({
	selectedClipId: null,
	selectedClip: null,
	streamingClipId: null,
	streamingText: '',
	streamingTokenCount: 0,
	generationContext: null,
	generationError: null,
	aiStatus: null,
	consistencySuggestions: [],
	checkingConsistency: false,
	canUndo: false,
	canRedo: false,
	batchParentClipId: null,
	batchTotalCount: 0,
	batchCompletedCount: 0,
});

/** Reset streaming state and begin a new generation for a single clip. */
export function startGeneration(clipId: string) {
	editorState.streamingClipId = clipId;
	editorState.streamingText = '';
	editorState.streamingTokenCount = 0;
	editorState.generationContext = null;
	editorState.generationError = null;
	// Clear any batch state when starting individual generation.
	editorState.batchParentClipId = null;
}

/** Start batch generation tracking for all sub-beats of a parent clip. */
export function startBatchGeneration(parentClipId: string) {
	editorState.batchParentClipId = parentClipId;
	editorState.batchTotalCount = 0;
	editorState.batchCompletedCount = 0;
	editorState.streamingClipId = null;
	editorState.streamingText = '';
	editorState.streamingTokenCount = 0;
	editorState.generationContext = null;
	editorState.generationError = null;
}

/** Update the expected batch total count (called after API response). */
export function setBatchTotalCount(count: number) {
	editorState.batchTotalCount = count;
	if (editorState.batchCompletedCount >= count && count > 0) {
		editorState.batchParentClipId = null;
	}
}

/** Append a streaming token if it matches the active generation. */
export function appendStreamingToken(clipId: string, token: string, count: number) {
	if (editorState.streamingClipId === clipId) {
		editorState.streamingText += token;
		editorState.streamingTokenCount = count;
	} else if (editorState.batchParentClipId != null) {
		// New sub-beat starting during batch — auto-switch streaming target.
		editorState.streamingClipId = clipId;
		editorState.streamingText = token;
		editorState.streamingTokenCount = count;
		editorState.generationContext = null;
	}
}

/** Finalize a completed generation. */
export function completeGeneration(clipId: string) {
	if (editorState.streamingClipId === clipId) {
		editorState.streamingClipId = null;
		editorState.streamingText = '';
	}
	if (editorState.batchParentClipId != null) {
		editorState.batchCompletedCount++;
		if (editorState.batchCompletedCount >= editorState.batchTotalCount && editorState.batchTotalCount > 0) {
			editorState.batchParentClipId = null;
		}
	}
}

/** Store the AI generation context (formatted prompts). */
export function setGenerationContext(clipId: string, system: string, user: string) {
	// Auto-switch for batch mode if a new sub-beat's context arrives.
	if (editorState.batchParentClipId != null && editorState.streamingClipId !== clipId) {
		editorState.streamingClipId = clipId;
		editorState.streamingText = '';
		editorState.streamingTokenCount = 0;
	}
	if (editorState.streamingClipId === clipId) {
		editorState.generationContext = { system, user };
	}
}

/** Record a generation error. */
export function setGenerationError(clipId: string, error: string) {
	if (editorState.streamingClipId === clipId) {
		editorState.generationError = error;
		editorState.streamingClipId = null;
		editorState.streamingText = '';
	}
	if (editorState.batchParentClipId != null) {
		editorState.batchCompletedCount++;
		if (editorState.batchCompletedCount >= editorState.batchTotalCount && editorState.batchTotalCount > 0) {
			editorState.batchParentClipId = null;
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

/** Remove a consistency suggestion by target clip ID. */
export function removeConsistencySuggestion(targetClipId: string) {
	editorState.consistencySuggestions = editorState.consistencySuggestions.filter(
		(s) => s.target_clip_id !== targetClipId
	);
}

/** Clear all consistency suggestions. */
export function clearConsistencySuggestions() {
	editorState.consistencySuggestions = [];
	editorState.checkingConsistency = false;
}
