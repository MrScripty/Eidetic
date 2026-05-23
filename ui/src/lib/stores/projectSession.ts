import type { Project } from '$lib/projectTypes.js';
import {
  MAIN_SCRIPT_DOCUMENT_ID,
  clearScriptDocumentProjection,
  refreshScriptDocumentProjection,
} from './scriptDocumentProjection.svelte.js';
import {
  clearBibleGraphNodeListProjection,
  refreshBibleGraphNodeListProjection,
} from './bibleGraphNodeProjection.svelte.js';
import {
  clearBibleGraphSchemaListProjection,
  refreshBibleGraphSchemaListProjection,
} from './bibleGraphSchemaProjection.svelte.js';
import {
  bibleRenderGraphRequestForTimelineSelection,
  clearBibleRenderGraphProjection,
  refreshBibleRenderGraphProjection,
} from './bibleRenderGraphProjection.svelte.js';
import { clearContextStackProjection } from './contextStackProjection.svelte.js';
import { selectBibleGraphNode } from './bible.svelte.js';
import {
  clearChangeReviewProjection,
  refreshChangeReviewProjection,
} from './changeReviewProjection.svelte.js';
import { resetEditorState } from './editor.svelte.js';
import { clearProjectionRefreshQueue } from './projectionRefreshQueue.js';
import {
  clearPropagationProposalListProjection,
  refreshPropagationProposalListProjection,
} from './propagationProposalProjection.svelte.js';
import { projectState } from './project.svelte.js';
import {
  clearBibleReferenceProposalListProjection,
  refreshBibleReferenceProposalListProjection,
} from './semanticProposalProjection.svelte.js';
import {
  clearStoryArcListProjection,
  refreshStoryArcListProjection,
} from './storyArcProjection.svelte.js';
import {
  clearTimelineRenderProjection,
  refreshTimelineRenderProjection,
} from './timelineRenderProjection.svelte.js';
import { clearSelectedNodeEditorProjection } from './selectedNodeEditorProjection.svelte.js';

export interface ProjectSessionLifecycle {
  clearProjectionRefreshQueue: () => void;
  resetEditorState: () => void;
  clearBibleSelection: () => void;
  clearProjectionCaches: () => void;
  setActiveProject: (project: Project) => void;
  refreshProjections: () => Promise<unknown>;
}

const mainScriptDocumentKey = { document_id: MAIN_SCRIPT_DOCUMENT_ID };

const defaultProjectSessionLifecycle: ProjectSessionLifecycle = {
  clearProjectionRefreshQueue,
  resetEditorState,
  clearBibleSelection: () => selectBibleGraphNode(null),
  clearProjectionCaches() {
    clearTimelineRenderProjection();
    clearStoryArcListProjection();
    clearSelectedNodeEditorProjection();
    clearScriptDocumentProjection(mainScriptDocumentKey);
    clearBibleGraphNodeListProjection();
    clearBibleGraphSchemaListProjection();
    clearBibleRenderGraphProjection();
    clearContextStackProjection();
    clearBibleReferenceProposalListProjection();
    clearPropagationProposalListProjection();
    clearChangeReviewProjection();
  },
  setActiveProject(project) {
    projectState.current = { name: project.name };
  },
  refreshProjections() {
    return Promise.all([
      refreshTimelineRenderProjection(),
      refreshStoryArcListProjection(),
      refreshScriptDocumentProjection(mainScriptDocumentKey),
      refreshBibleGraphNodeListProjection(),
      refreshBibleGraphSchemaListProjection(),
      refreshBibleRenderGraphProjection(bibleRenderGraphRequestForTimelineSelection(null)),
      refreshBibleReferenceProposalListProjection(),
      refreshPropagationProposalListProjection(),
      refreshChangeReviewProjection(),
    ]);
  },
};

export async function activateProjectSession(
  project: Project,
  lifecycle: ProjectSessionLifecycle = defaultProjectSessionLifecycle,
): Promise<void> {
  lifecycle.clearProjectionRefreshQueue();
  lifecycle.resetEditorState();
  lifecycle.clearBibleSelection();
  lifecycle.clearProjectionCaches();
  lifecycle.setActiveProject(project);
  await lifecycle.refreshProjections();
}
