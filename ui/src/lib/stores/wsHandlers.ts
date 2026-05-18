import type { WsClient } from '$lib/ws.js';
import { timelineState } from './timeline.svelte.js';
import { storyState } from './story.svelte.js';
import {
  editorState,
  appendStreamingToken,
  completeGeneration,
  setGenerationContext,
  setGenerationError,
} from './editor.svelte.js';
import { getTimeline, listArcs, listEntities, getNodeContent } from '$lib/api.js';
import {
  MAIN_SCRIPT_DOCUMENT_ID,
  refreshScriptDocumentProjection,
} from './scriptDocumentProjection.svelte.js';
import { refreshTimelineRenderProjection } from './timelineRenderProjection.svelte.js';

/** Register WebSocket event handlers that update Svelte stores. */
export function setupWsHandlers(ws: WsClient) {
  const unsubscribers = [
    ws.on('timeline_changed', async () => {
      const timeline = await getTimeline();
      timelineState.timeline = timeline;
      await refreshTimelineRenderProjection().catch(() => {});
    }),

    ws.on('hierarchy_changed', async () => {
      const timeline = await getTimeline();
      timelineState.timeline = timeline;
      await refreshTimelineRenderProjection().catch(() => {});
    }),

    ws.on('story_changed', async () => {
      const arcs = await listArcs();
      const entities = await listEntities();
      storyState.arcs = arcs;
      storyState.entities = entities;
    }),

    ws.on('node_updated', async (data) => {
      const nodeId = data.node_id;
      const content = await getNodeContent(nodeId);
      if (editorState.selectedNodeId === nodeId && editorState.selectedNode) {
        editorState.selectedNode.content = content;
      }
      // Update timeline data so ScriptPanel reflects the change.
      if (timelineState.timeline) {
        const node = timelineState.timeline.nodes.find((n) => n.id === nodeId);
        if (node) {
          node.content = content;
        }
      }
    }),

    ws.on('generation_context', (data) => {
      setGenerationContext(data.node_id, data.system_prompt, data.user_prompt);
    }),

    ws.on('generation_progress', (data) => {
      appendStreamingToken(data.node_id, data.token, data.tokens_generated);
    }),

    ws.on('generation_complete', async (data) => {
      const nodeId = data.node_id;
      // Fetch and update content BEFORE clearing streaming to avoid flicker.
      const content = await getNodeContent(nodeId);
      if (editorState.selectedNodeId === nodeId && editorState.selectedNode) {
        editorState.selectedNode.content = content;
      }
      if (timelineState.timeline) {
        const node = timelineState.timeline.nodes.find((n) => n.id === nodeId);
        if (node) {
          node.content = content;
        }
      }
      completeGeneration(nodeId);
    }),

    ws.on('generation_error', (data) => {
      setGenerationError(data.node_id, data.error);
    }),

    ws.on('undo_redo_changed', (data) => {
      editorState.canUndo = data.can_undo;
      editorState.canRedo = data.can_redo;
    }),

    ws.on('project_mutated', async () => {
      const timeline = await getTimeline();
      timelineState.timeline = timeline;
      await refreshTimelineRenderProjection().catch(() => {});
      const arcs = await listArcs();
      const entities = await listEntities();
      storyState.arcs = arcs;
      storyState.entities = entities;
    }),

    ws.on('bible_changed', async () => {
      const entities = await listEntities();
      storyState.entities = entities;
    }),

    ws.on('script_changed', async () => {
      await refreshScriptDocumentProjection({ document_id: MAIN_SCRIPT_DOCUMENT_ID }).catch(
        () => {},
      );
    }),

    ws.on('entity_extraction_complete', (_data) => {
      listEntities().then((entities) => (storyState.entities = entities));
    }),
  ];

  return () => {
    for (const unsubscribe of unsubscribers) {
      unsubscribe();
    }
  };
}
