import type { ClipId, BeatClip, AiStatus, ConsistencySuggestion } from '../types.js';

/** Transient UI state for the beat editor panel. Frontend-owned. */
export const editorState = $state<{
	selectedClipId: ClipId | null;
	selectedClip: BeatClip | null;
	/** Clip ID currently streaming generation. */
	streamingClipId: ClipId | null;
	/** Accumulated streaming text during generation. */
	streamingText: string;
	/** Number of tokens received so far. */
	streamingTokenCount: number;
	/** Error from a failed generation. */
	generationError: string | null;
	/** Last-known AI backend status. */
	aiStatus: AiStatus | null;
	/** Pending consistency suggestions from the AI. */
	consistencySuggestions: ConsistencySuggestion[];
	/** Whether a consistency check is in progress. */
	checkingConsistency: boolean;
}>({
	selectedClipId: null,
	selectedClip: null,
	streamingClipId: null,
	streamingText: '',
	streamingTokenCount: 0,
	generationError: null,
	aiStatus: null,
	consistencySuggestions: [],
	checkingConsistency: false,
});

/** Reset streaming state and begin a new generation. */
export function startGeneration(clipId: string) {
	editorState.streamingClipId = clipId;
	editorState.streamingText = '';
	editorState.streamingTokenCount = 0;
	editorState.generationError = null;
}

/** Append a streaming token if it matches the active generation. */
export function appendStreamingToken(clipId: string, token: string, count: number) {
	if (editorState.streamingClipId === clipId) {
		editorState.streamingText += token;
		editorState.streamingTokenCount = count;
	}
}

/** Finalize a completed generation. */
export function completeGeneration(clipId: string) {
	if (editorState.streamingClipId === clipId) {
		editorState.streamingClipId = null;
	}
}

/** Record a generation error. */
export function setGenerationError(clipId: string, error: string) {
	if (editorState.streamingClipId === clipId) {
		editorState.generationError = error;
		editorState.streamingClipId = null;
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
