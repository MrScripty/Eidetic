import type { ServerEventClient } from '$lib/serverEventClient.js';
import { invalidateScriptContext } from './scriptDocumentProjection.svelte.js';
import {
  appendStreamingToken,
  completeGeneration,
  editorState,
  getEditorSessionGeneration,
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
  getActiveBibleRenderGraphProjectionRequest,
  refreshBibleRenderGraphProjection,
} from './bibleRenderGraphProjection.svelte.js';
import { refreshBibleReferenceProposalListProjection } from './semanticProposalProjection.svelte.js';
import { refreshPropagationProposalListProjection } from './propagationProposalProjection.svelte.js';
import { refreshChangeReviewProjection } from './changeReviewProjection.svelte.js';
import { clearProjectionRefreshQueue, requestProjectionRefresh } from './projectionRefreshQueue.js';
import { applyGraphRendererCommand } from './graphRendererCommands.js';
import { timelineState } from './timeline.svelte.js';
import { refreshCurrentContextStackProjection } from './contextStackProjection.svelte.js';
import { refreshSelectedNodeEditorProjection } from './selectedNodeEditorProjection.svelte.js';

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
    refreshBibleRenderGraphProjection(getActiveBibleRenderGraphProjectionRequest()),
  );
}

function refreshActiveBibleRenderGraphForContextInfluence() {
  return requestProjectionRefresh('bible-render-graph:active-context-influence', () =>
    refreshBibleRenderGraphProjection(getActiveBibleRenderGraphProjectionRequest()),
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

function refreshContextStack() {
  return requestProjectionRefresh('context-stack', refreshCurrentContextStackProjection);
}

/** Register backend event handlers that update Svelte stores. */
export function setupServerEventHandlers(events: ServerEventClient): () => void {
  let active = true;

  function refreshSelectedInspector(affectedNodeId?: string) {
    const nodeId = editorState.selectedNodeId;
    if (!nodeId || (affectedNodeId !== undefined && affectedNodeId !== nodeId)) {
      return Promise.resolve();
    }
    const session = getEditorSessionGeneration();
    const isCurrent = () =>
      active && session === getEditorSessionGeneration() && nodeId === editorState.selectedNodeId;
    return requestProjectionRefresh('selected-node-editor', async () => {
      // Events invalidate the canonical read; they never own selection or draft receipts.
      if (isCurrent()) await refreshSelectedNodeEditorProjection(nodeId, isCurrent);
    });
  }

  const unsubscribers = [
    events.on('timeline_changed', async () => {
      await Promise.all([
        refreshTimelineRender(),
        refreshContextStack(),
        refreshSelectedInspector(),
      ]);
    }),

    events.on('hierarchy_changed', async () => {
      await Promise.all([
        refreshTimelineRender(),
        refreshContextStack(),
        refreshSelectedInspector(),
      ]);
    }),

    events.on('story_changed', async () => {
      await refreshStoryArcs();
    }),

    events.on('node_updated', async (data) => {
      await Promise.all([
        refreshTimelineRender(),
        refreshMainScriptDocument(),
        refreshContextStack(),
        refreshSelectedInspector(data.node_id),
      ]);
    }),

    events.on('generation_context', (data) => {
      setGenerationContext(data.node_id, data.system_prompt, data.user_prompt);
    }),

    events.on('generation_progress', (data) => {
      appendStreamingToken(data.node_id, data.token, data.tokens_generated);
    }),

    events.on('generation_complete', async (data) => {
      const session = getEditorSessionGeneration();
      // Inspector freshness has its own ownership/error state. Its read must not
      // hold a completed generation in the streaming state.
      void refreshSelectedInspector(data.node_id);
      await Promise.all([
        refreshTimelineRender(),
        refreshMainScriptDocument(),
        refreshContextStack(),
      ]);
      if (active && session === getEditorSessionGeneration()) completeGeneration(data.node_id);
    }),

    events.on('generation_error', (data) => {
      setGenerationError(data.node_id, data.error);
    }),

    events.on('bible_changed', async () => {
      invalidateScriptContext();
      await Promise.all([
        refreshBibleNodeList(),
        refreshBibleRenderGraph(),
        refreshChangeReview(),
        refreshMainScriptDocument(),
      ]);
    }),

    events.on('semantic_proposals_changed', async () => {
      await Promise.all([
        refreshSemanticProposals(),
        refreshPropagationProposals(),
        refreshChangeReview(),
      ]);
    }),

    events.on('context_influence_changed', async () => {
      invalidateScriptContext();
      await Promise.all([
        refreshActiveBibleRenderGraphForContextInfluence(),
        refreshMainScriptDocument(),
        refreshChangeReview(),
        refreshContextStack(),
      ]);
    }),

    events.on('script_changed', async () => {
      invalidateScriptContext();
      await Promise.all([
        refreshMainScriptDocument(),
        refreshChangeReview(),
        refreshContextStack(),
      ]);
    }),

    events.on('timeline_selection_changed', async (command) => {
      editorState.selectedNodeId = command.node_id;
      await Promise.all([
        refreshTimelineRender(),
        refreshActiveBibleRenderGraphForContextInfluence(),
      ]);
    }),

    events.on('timeline_playhead_changed', async (command) => {
      timelineState.playheadMs = Math.max(0, Math.trunc(command.position_ms));
      await refreshActiveBibleRenderGraphForContextInfluence();
    }),

    events.on('select_node', (command) => {
      applyGraphRendererCommand(command);
    }),

    events.on('select_edge', (command) => {
      applyGraphRendererCommand(command);
    }),

    events.on('select_influence', (command) => {
      applyGraphRendererCommand(command);
    }),

    events.on('inspect_node', (command) => {
      applyGraphRendererCommand(command);
    }),

    events.on('focus_node', (command) => {
      applyGraphRendererCommand(command);
    }),

    events.on('navigate_to_node', (command) => {
      applyGraphRendererCommand(command);
    }),
  ];

  return () => {
    active = false;
    for (const unsubscribe of unsubscribers) {
      unsubscribe();
    }
    clearProjectionRefreshQueue();
  };
}
