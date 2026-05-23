import type { ServerEventClient } from '$lib/serverEventClient.js';
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
import {
  bibleRenderGraphRequestForTimelineSelection,
  refreshBibleRenderGraphProjection,
} from './bibleRenderGraphProjection.svelte.js';
import { refreshBibleReferenceProposalListProjection } from './semanticProposalProjection.svelte.js';
import { refreshPropagationProposalListProjection } from './propagationProposalProjection.svelte.js';
import { refreshChangeReviewProjection } from './changeReviewProjection.svelte.js';
import { clearProjectionRefreshQueue, requestProjectionRefresh } from './projectionRefreshQueue.js';

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
  return requestProjectionRefresh('bible-render-graph', () =>
    refreshBibleRenderGraphProjection(bibleRenderGraphRequestForTimelineSelection(null)),
  );
}

function refreshBibleRenderGraphForTimelineNode(nodeId: string) {
  return requestProjectionRefresh(`bible-render-graph:${nodeId}`, () =>
    refreshBibleRenderGraphProjection(bibleRenderGraphRequestForTimelineSelection(nodeId)),
  );
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

/** Register backend event handlers that update Svelte stores. */
export function setupServerEventHandlers(events: ServerEventClient) {
  const unsubscribers = [
    events.on('timeline_changed', async () => {
      await refreshTimelineRender();
    }),

    events.on('hierarchy_changed', async () => {
      await refreshTimelineRender();
    }),

    events.on('story_changed', async () => {
      await refreshStoryArcs();
    }),

    events.on('node_updated', async () => {
      await Promise.all([refreshTimelineRender(), refreshMainScriptDocument()]);
    }),

    events.on('generation_context', (data) => {
      setGenerationContext(data.node_id, data.system_prompt, data.user_prompt);
    }),

    events.on('generation_progress', (data) => {
      appendStreamingToken(data.node_id, data.token, data.tokens_generated);
    }),

    events.on('generation_complete', async (data) => {
      await Promise.all([refreshTimelineRender(), refreshMainScriptDocument()]);
      completeGeneration(data.node_id);
    }),

    events.on('generation_error', (data) => {
      setGenerationError(data.node_id, data.error);
    }),

    events.on('bible_changed', async () => {
      await Promise.all([refreshBibleNodeList(), refreshBibleRenderGraph(), refreshChangeReview()]);
    }),

    events.on('semantic_proposals_changed', async () => {
      await Promise.all([
        refreshSemanticProposals(),
        refreshPropagationProposals(),
        refreshChangeReview(),
      ]);
    }),

    events.on('context_influence_changed', async (data) => {
      await refreshBibleRenderGraphForTimelineNode(data.target_node_id);
    }),

    events.on('script_changed', async () => {
      await Promise.all([refreshMainScriptDocument(), refreshChangeReview()]);
    }),
  ];

  return () => {
    for (const unsubscribe of unsubscribers) {
      unsubscribe();
    }
    clearProjectionRefreshQueue();
  };
}
