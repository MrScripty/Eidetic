import type { WsClient } from '$lib/ws.js';
import { timelineState } from './timeline.svelte.js';
import { storyState } from './story.svelte.js';
import {
	editorState,
	appendStreamingToken,
	completeGeneration,
	setGenerationError,
	addConsistencySuggestion,
} from './editor.svelte.js';
import { getTimeline, getScenes, listArcs, listEntities, getBeat } from '$lib/api.js';

/** Register WebSocket event handlers that update Svelte stores. */
export function setupWsHandlers(ws: WsClient) {
	ws.on('timeline_changed', async () => {
		const timeline = await getTimeline();
		timelineState.timeline = timeline;
	});

	ws.on('scenes_changed', async () => {
		const scenes = await getScenes();
		timelineState.scenes = scenes;
	});

	ws.on('story_changed', async () => {
		const arcs = await listArcs();
		const entities = await listEntities();
		storyState.arcs = arcs;
		storyState.entities = entities;
	});

	ws.on('beat_updated', async (data) => {
		const clipId = data.clip_id;
		if (editorState.selectedClipId === clipId && editorState.selectedClip) {
			const content = await getBeat(clipId);
			editorState.selectedClip.content = content;
		}
	});

	ws.on('generation_progress', (data) => {
		appendStreamingToken(data.clip_id, data.token, data.tokens_generated);
	});

	ws.on('generation_complete', async (data) => {
		const clipId = data.clip_id;
		completeGeneration(clipId);
		// Refresh clip content from server.
		if (editorState.selectedClipId === clipId && editorState.selectedClip) {
			const content = await getBeat(clipId);
			editorState.selectedClip.content = content;
		}
	});

	ws.on('generation_error', (data) => {
		setGenerationError(data.clip_id, data.error);
	});

	ws.on('consistency_suggestion', (data) => {
		addConsistencySuggestion({
			source_clip_id: data.source_clip_id,
			target_clip_id: data.target_clip_id,
			original_text: data.original_text,
			suggested_text: data.suggested_text,
			reason: data.reason,
		});
	});

	ws.on('consistency_complete', () => {
		editorState.checkingConsistency = false;
	});

	ws.on('undo_redo_changed', (data) => {
		editorState.canUndo = data.can_undo;
		editorState.canRedo = data.can_redo;
	});

	ws.on('project_mutated', async () => {
		const timeline = await getTimeline();
		timelineState.timeline = timeline;
		const scenes = await getScenes();
		timelineState.scenes = scenes;
		const arcs = await listArcs();
		const entities = await listEntities();
		storyState.arcs = arcs;
		storyState.entities = entities;
	});

	ws.on('bible_changed', async () => {
		const entities = await listEntities();
		storyState.entities = entities;
	});

	ws.on('entity_extraction_complete', (_data) => {
		// Refresh entities after extraction (new entities/snapshots may have been committed).
		listEntities().then(entities => storyState.entities = entities);
	});
}
