import type { WsClient } from '$lib/ws.js';
import type { Timeline, InferredScene, StoryArc, Character, BeatContent } from '$lib/types.js';
import { timelineState } from './timeline.svelte.js';
import { storyState } from './story.svelte.js';
import {
	editorState,
	appendStreamingToken,
	completeGeneration,
	setGenerationError,
	addConsistencySuggestion,
} from './editor.svelte.js';
import { getTimeline, getScenes, listArcs, listCharacters, getBeat } from '$lib/api.js';

/** Register WebSocket event handlers that update Svelte stores. */
export function setupWsHandlers(ws: WsClient) {
	ws.on('timeline_changed', async () => {
		const timeline = (await getTimeline()) as Timeline;
		timelineState.timeline = timeline;
	});

	ws.on('scenes_changed', async () => {
		const scenes = (await getScenes()) as InferredScene[];
		timelineState.scenes = scenes;
	});

	ws.on('story_changed', async () => {
		const arcs = (await listArcs()) as StoryArc[];
		const characters = (await listCharacters()) as Character[];
		storyState.arcs = arcs;
		storyState.characters = characters;
	});

	ws.on('beat_updated', async (data) => {
		const clipId = data.clip_id as string;
		if (editorState.selectedClipId === clipId && editorState.selectedClip) {
			const content = (await getBeat(clipId)) as BeatContent;
			editorState.selectedClip.content = content;
		}
	});

	ws.on('generation_progress', (data) => {
		const clipId = data.clip_id as string;
		const token = data.token as string;
		const tokensGenerated = data.tokens_generated as number;
		appendStreamingToken(clipId, token, tokensGenerated);
	});

	ws.on('generation_complete', async (data) => {
		const clipId = data.clip_id as string;
		completeGeneration(clipId);
		// Refresh clip content from server.
		if (editorState.selectedClipId === clipId && editorState.selectedClip) {
			const content = (await getBeat(clipId)) as BeatContent;
			editorState.selectedClip.content = content;
		}
	});

	ws.on('generation_error', (data) => {
		const clipId = data.clip_id as string;
		const error = data.error as string;
		setGenerationError(clipId, error);
	});

	ws.on('consistency_suggestion', (data) => {
		addConsistencySuggestion({
			source_clip_id: data.source_clip_id as string,
			target_clip_id: data.target_clip_id as string,
			original_text: data.original_text as string,
			suggested_text: data.suggested_text as string,
			reason: data.reason as string,
		});
	});

	ws.on('consistency_complete', (data) => {
		editorState.checkingConsistency = false;
		const count = data.suggestion_count as number;
		if (count === 0) {
			// No suggestions — nothing to show.
		}
	});
}
