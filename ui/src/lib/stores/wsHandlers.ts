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
import { requestProjectionRefresh } from './projectionRefreshQueue.js';

const SCRIPT_DOCUMENT_KEY = `script-document:${MAIN_SCRIPT_DOCUMENT_ID}`;

function refreshTimelineRender() {
  return requestProjectionRefresh('timeline-render', refreshTimelineRenderProjection);
}

function refreshMainScriptDocument() {
  return requestProjectionRefresh(SCRIPT_DOCUMENT_KEY, () =>
    refreshScriptDocumentProjection({ document_id: MAIN_SCRIPT_DOCUMENT_ID }),
  );
}

function refreshStoryArcs() {
  return requestProjectionRefresh('story-arcs', refreshStoryArcListProjection);
}

function refreshBibleNodeList() {
  return requestProjectionRefresh('bible-node-list', refreshBibleGraphNodeListProjection);
}

function refreshBibleRenderGraph() {
  return requestProjectionRefresh('bible-render-graph', refreshBibleRenderGraphProjection);
}

function refreshSemanticProposals() {
  return requestProjectionRefresh(
    'semantic-bible-reference-proposals',
    refreshBibleReferenceProposalListProjection,
  );
}

function refreshPropagationProposals() {
  return requestProjectionRefresh(
    'semantic-propagation-proposals',
    refreshPropagationProposalListProjection,
  );
}

function refreshChangeReview() {
  return requestProjectionRefresh('change-review', refreshChangeReviewProjection);
}

/** Register WebSocket event handlers that update Svelte stores. */
export function setupWsHandlers(ws: WsClient) {
  const unsubscribers = [
    ws.on('timeline_changed', async () => {
      await refreshTimelineRender();
    }),

    ws.on('hierarchy_changed', async () => {
      await refreshTimelineRender();
    }),

    ws.on('story_changed', async () => {
      await refreshStoryArcs();
    }),

    ws.on('node_updated', async () => {
      await Promise.all([refreshTimelineRender(), refreshMainScriptDocument()]);
    }),

    ws.on('generation_context', (data) => {
      setGenerationContext(data.node_id, data.system_prompt, data.user_prompt);
    }),

    ws.on('generation_progress', (data) => {
      appendStreamingToken(data.node_id, data.token, data.tokens_generated);
    }),

    ws.on('generation_complete', async (data) => {
      await Promise.all([refreshTimelineRender(), refreshMainScriptDocument()]);
      completeGeneration(data.node_id);
    }),

    ws.on('generation_error', (data) => {
      setGenerationError(data.node_id, data.error);
    }),

    ws.on('bible_changed', async () => {
      await Promise.all([refreshBibleNodeList(), refreshBibleRenderGraph(), refreshChangeReview()]);
    }),

    ws.on('semantic_proposals_changed', async () => {
      await Promise.all([
        refreshSemanticProposals(),
        refreshPropagationProposals(),
        refreshChangeReview(),
      ]);
    }),

    ws.on('script_changed', async () => {
      await Promise.all([refreshMainScriptDocument(), refreshChangeReview()]);
    }),
  ];

  return () => {
    for (const unsubscribe of unsubscribers) {
      unsubscribe();
    }
  };
}
