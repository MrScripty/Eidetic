import type { WsClient } from '$lib/ws.js';
import type { Timeline, InferredScene, StoryArc, Character, BeatContent } from '$lib/types.js';
import { timelineState } from './timeline.svelte.js';
import { storyState } from './story.svelte.js';
import { editorState } from './editor.svelte.js';
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
}
