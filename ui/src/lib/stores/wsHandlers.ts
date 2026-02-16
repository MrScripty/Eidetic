import type { WsClient } from '$lib/ws.js';
import { timelineState } from './timeline.svelte.js';
import { storyState } from './story.svelte.js';
import {
	editorState,
	appendStreamingToken,
	completeGeneration,
	setGenerationContext,
	setGenerationError,
	addConsistencySuggestion,
} from './editor.svelte.js';
import { getTimeline, listArcs, listEntities, getNodeContent } from '$lib/api.js';

/** Register WebSocket event handlers that update Svelte stores. */
export function setupWsHandlers(ws: WsClient) {
	ws.on('timeline_changed', async () => {
		const timeline = await getTimeline();
		timelineState.timeline = timeline;
	});

	ws.on('hierarchy_changed', async () => {
		const timeline = await getTimeline();
		timelineState.timeline = timeline;
	});

	ws.on('story_changed', async () => {
		const arcs = await listArcs();
		const entities = await listEntities();
		storyState.arcs = arcs;
		storyState.entities = entities;
	});

	ws.on('node_updated', async (data) => {
		const nodeId = data.node_id;
		const content = await getNodeContent(nodeId);
		if (editorState.selectedNodeId === nodeId && editorState.selectedNode) {
			editorState.selectedNode.content = content;
		}
		// Update timeline data so ScriptPanel reflects the change.
		if (timelineState.timeline) {
			const node = timelineState.timeline.nodes.find(n => n.id === nodeId);
			if (node) {
				node.content = content;
			}
		}
	});

	ws.on('generation_context', (data) => {
		setGenerationContext(data.node_id, data.system_prompt, data.user_prompt);
	});

	ws.on('generation_progress', (data) => {
		appendStreamingToken(data.node_id, data.token, data.tokens_generated);
	});

	ws.on('generation_complete', async (data) => {
		const nodeId = data.node_id;
		// Fetch and update content BEFORE clearing streaming to avoid flicker.
		const content = await getNodeContent(nodeId);
		if (editorState.selectedNodeId === nodeId && editorState.selectedNode) {
			editorState.selectedNode.content = content;
		}
		if (timelineState.timeline) {
			const node = timelineState.timeline.nodes.find(n => n.id === nodeId);
			if (node) {
				node.content = content;
			}
		}
		completeGeneration(nodeId);
	});

	ws.on('generation_error', (data) => {
		setGenerationError(data.node_id, data.error);
	});

	ws.on('consistency_suggestion', (data) => {
		addConsistencySuggestion({
			source_node_id: data.source_node_id,
			target_node_id: data.target_node_id,
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
		listEntities().then(entities => storyState.entities = entities);
	});
}
