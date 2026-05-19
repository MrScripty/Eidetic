import type { WsClient } from '$lib/ws.js';
import { timelineState } from './timeline.svelte.js';
import {
  editorState,
  appendStreamingToken,
  completeGeneration,
  setGenerationContext,
  setGenerationError,
} from './editor.svelte.js';
import { getTimeline, getNodeContent } from '$lib/api.js';
import {
  MAIN_SCRIPT_DOCUMENT_ID,
  refreshScriptDocumentProjection,
} from './scriptDocumentProjection.svelte.js';
import { refreshStoryArcListProjection } from './storyArcProjection.svelte.js';
import { refreshTimelineRenderProjection } from './timelineRenderProjection.svelte.js';
import { refreshBibleGraphNodeListProjection } from './bibleGraphNodeProjection.svelte.js';
import { refreshBibleReferenceProposalListProjection } from './semanticProposalProjection.svelte.js';

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
      await refreshStoryArcListProjection().catch(() => {});
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

    ws.on('bible_changed', async () => {
      await refreshBibleGraphNodeListProjection().catch(() => {});
    }),

    ws.on('semantic_proposals_changed', async () => {
      await refreshBibleReferenceProposalListProjection().catch(() => {});
    }),

    ws.on('script_changed', async () => {
      await refreshScriptDocumentProjection({ document_id: MAIN_SCRIPT_DOCUMENT_ID }).catch(
        () => {},
      );
    }),
  ];

  return () => {
    for (const unsubscribe of unsubscribers) {
      unsubscribe();
    }
  };
}
