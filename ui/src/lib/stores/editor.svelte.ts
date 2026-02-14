import type { ClipId, BeatClip } from '../types.js';

/** Transient UI state for the beat editor panel. Frontend-owned. */
export const editorState = $state<{
	selectedClipId: ClipId | null;
	selectedClip: BeatClip | null;
}>({
	selectedClipId: null,
	selectedClip: null,
});
