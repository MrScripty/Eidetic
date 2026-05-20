import type { WsClient } from '$lib/ws.js';
import {
  appendStreamingToken,
  completeGeneration,
  setGenerationContext,
  setGenerationError,
} from './editor.svelte.js';
import {
  MAIN_SCRIPT_DOCUMENT_ID,
  refreshScriptDocumentProjection,
} from './scriptDocumentProjection.svelte.js';
import { refreshStoryArcListProjection } from './storyArcProjection.svelte.js';
import { refreshTimelineRenderProjection } from './timelineRenderProjection.svelte.js';
import { refreshBibleGraphNodeListProjection } from './bibleGraphNodeProjection.svelte.js';
import { refreshBibleRenderGraphProjection } from './bibleRenderGraphProjection.svelte.js';
import { refreshBibleReferenceProposalListProjection } from './semanticProposalProjection.svelte.js';
import { refreshPropagationProposalListProjection } from './propagationProposalProjection.svelte.js';
import { refreshChangeReviewProjection } from './changeReviewProjection.svelte.js';

/** Register WebSocket event handlers that update Svelte stores. */
export function setupWsHandlers(ws: WsClient) {
  const unsubscribers = [
    ws.on('timeline_changed', async () => {
      await refreshTimelineRenderProjection().catch(() => {});
    }),

    ws.on('hierarchy_changed', async () => {
      await refreshTimelineRenderProjection().catch(() => {});
    }),

    ws.on('story_changed', async () => {
      await refreshStoryArcListProjection().catch(() => {});
    }),

    ws.on('node_updated', async () => {
      await Promise.all([
        refreshTimelineRenderProjection().catch(() => {}),
        refreshScriptDocumentProjection({ document_id: MAIN_SCRIPT_DOCUMENT_ID }).catch(() => {}),
      ]);
    }),

    ws.on('generation_context', (data) => {
      setGenerationContext(data.node_id, data.system_prompt, data.user_prompt);
    }),

    ws.on('generation_progress', (data) => {
      appendStreamingToken(data.node_id, data.token, data.tokens_generated);
    }),

    ws.on('generation_complete', async (data) => {
      await Promise.all([
        refreshTimelineRenderProjection().catch(() => {}),
        refreshScriptDocumentProjection({ document_id: MAIN_SCRIPT_DOCUMENT_ID }).catch(() => {}),
      ]);
      completeGeneration(data.node_id);
    }),

    ws.on('generation_error', (data) => {
      setGenerationError(data.node_id, data.error);
    }),

    ws.on('bible_changed', async () => {
      await refreshBibleGraphNodeListProjection().catch(() => {});
      await refreshBibleRenderGraphProjection().catch(() => {});
      await refreshChangeReviewProjection().catch(() => {});
    }),

    ws.on('semantic_proposals_changed', async () => {
      await refreshBibleReferenceProposalListProjection().catch(() => {});
      await refreshPropagationProposalListProjection().catch(() => {});
      await refreshChangeReviewProjection().catch(() => {});
    }),

    ws.on('script_changed', async () => {
      await refreshScriptDocumentProjection({ document_id: MAIN_SCRIPT_DOCUMENT_ID }).catch(
        () => {},
      );
      await refreshChangeReviewProjection().catch(() => {});
    }),
  ];

  return () => {
    for (const unsubscribe of unsubscribers) {
      unsubscribe();
    }
  };
}
