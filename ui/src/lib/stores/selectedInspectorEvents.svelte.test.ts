import { afterEach, beforeEach, describe, expect, it, vi } from 'vitest';
import {
  getPropagationProposalListProjection,
  getSelectedNodeEditorProjection,
} from '$lib/projectionApi.js';
import type { ProjectionEnvelope } from '$lib/projectionTypes.js';
import type { SelectedNodeEditorProjection } from '$lib/selectedNodeEditorTypes.js';
import type { ServerMessage } from '$lib/serverEventTypes.js';
import type { ScriptBlockProjection } from '$lib/scriptTypes.js';
import {
  invalidateScriptContext,
  refreshScriptDocumentProjection,
} from './scriptDocumentProjection.svelte.js';
import { refreshCurrentContextStackProjection } from './contextStackProjection.svelte.js';
import { beatContentStatusLabel } from '$lib/components/editor/beatEditorStatus.js';
import { setupServerEventHandlers } from './serverEventHandlers.js';
import { editorState, resetEditorState, startGeneration } from './editor.svelte.js';
import {
  clearSelectedNodeEditorProjection,
  refreshSelectedNodeEditorProjection,
  selectedNodeEditorProjectionState,
} from './selectedNodeEditorProjection.svelte.js';
import {
  getSessionScriptBlockEditDraft,
  resetSessionScriptBlockEditDrafts,
} from './scriptBlockEditSession.svelte.js';
import { getSessionTimelinePlacementDraft } from './timelinePlacementSession.svelte.js';
import {
  clearPropagationProposalListProjection,
  getCachedPropagationProposalListProjection,
  refreshPropagationProposalListProjection,
} from './propagationProposalProjection.svelte.js';

vi.mock('$lib/projectionApi.js', () => ({
  getSelectedNodeEditorProjection: vi.fn(),
  getPropagationProposalListProjection: vi.fn(),
}));
vi.mock('./timelineRenderProjection.svelte.js', () => ({
  refreshTimelineRenderProjection: vi.fn().mockResolvedValue(undefined),
  applyTimelineNodeRangeCommand: vi.fn(),
}));
vi.mock('./scriptDocumentProjection.svelte.js', () => ({
  MAIN_SCRIPT_DOCUMENT_ID: 'script.document.main',
  invalidateScriptContext: vi.fn(),
  refreshScriptDocumentProjection: vi.fn().mockResolvedValue(undefined),
  applyScriptBlockEditCommand: vi.fn(),
}));
vi.mock('./contextStackProjection.svelte.js', () => ({
  refreshCurrentContextStackProjection: vi.fn().mockResolvedValue(undefined),
}));
vi.mock('./storyArcProjection.svelte.js', () => ({ refreshStoryArcListProjection: vi.fn() }));
vi.mock('./bibleGraphNodeProjection.svelte.js', () => ({
  refreshBibleGraphNodeListProjection: vi.fn(),
}));
vi.mock('./bibleRenderGraphProjection.svelte.js', () => ({
  getActiveBibleRenderGraphProjectionRequest: vi.fn(),
  refreshBibleRenderGraphProjection: vi.fn(),
}));
vi.mock('./semanticProposalProjection.svelte.js', () => ({
  refreshBibleReferenceProposalListProjection: vi.fn(),
}));
vi.mock('./changeReviewProjection.svelte.js', () => ({ refreshChangeReviewProjection: vi.fn() }));
vi.mock('./graphRendererCommands.js', () => ({ applyGraphRendererCommand: vi.fn() }));

class Events {
  handlers = new Map<ServerMessage['type'], (message: ServerMessage) => unknown>();
  on<T extends ServerMessage['type']>(
    type: T,
    handler: (message: Extract<ServerMessage, { type: T }>) => unknown,
  ) {
    this.handlers.set(type, handler as (message: ServerMessage) => unknown);
    return () => this.handlers.delete(type);
  }
  async emit(message: ServerMessage) {
    await this.handlers.get(message.type)?.(message);
  }
}
const read = vi.mocked(getSelectedNodeEditorProjection);
const initial: ProjectionEnvelope<SelectedNodeEditorProjection> = {
  version: 1,
  payload: {
    node: {
      node_id: 'B',
      level: 'Scene',
      sort_order: 0,
      name: 'Scene B',
      notes: 'Mara waits.',
      content_status: 'NotesOnly',
      locked: false,
      start_ms: 540000,
      end_ms: 570000,
      range_read: { start_ms: 540000, end_ms: 570000, node_revision_event_id: 'range-1' },
    },
    has_children: false,
    siblings: [],
    children: [],
    adjacent_parents: {},
  },
};
const updated: ProjectionEnvelope<SelectedNodeEditorProjection> = {
  version: 2,
  payload: {
    ...initial.payload,
    node: {
      ...initial.payload.node!,
      content_status: 'HasContent',
      start_ms: 180000,
      end_ms: 210000,
      range_read: { start_ms: 180000, end_ms: 210000, node_revision_event_id: 'range-2' },
    },
  },
};
let events: Events;
let teardown: () => void;
beforeEach(async () => {
  resetEditorState();
  resetSessionScriptBlockEditDrafts();
  clearSelectedNodeEditorProjection();
  clearPropagationProposalListProjection();
  editorState.selectedNodeId = 'B';
  read.mockReset().mockResolvedValue(initial);
  await refreshSelectedNodeEditorProjection('B');
  read.mockClear().mockResolvedValue(updated);
  events = new Events();
  teardown = setupServerEventHandlers(events as never);
});
afterEach(() => teardown());

async function retainedWork() {
  const draft = getSessionScriptBlockEditDraft('script.document.main', 'F');
  draft.begin({
    block: { id: 'F', text: 'Saved platform scene.' },
    revision_event_id: 'F-revision',
  } as ScriptBlockProjection);
  draft.state.text = 'Unrelated draft: Eli pockets a brass whistle.';
  vi.mocked(getPropagationProposalListProjection).mockResolvedValue({
    version: 1,
    payload: {
      proposals: [
        {
          id: 'pending-B',
          action: 'patch_script_block',
          target: { kind: 'script_block', block_id: 'B' },
          status: 'pending',
          summary: 'Amber umbrella',
          proposed_text: 'Synthetic preview: Mara carries an amber umbrella.',
          created_at_ms: 1,
        },
      ],
    },
  });
  await refreshPropagationProposalListProjection();
  const proposal = getCachedPropagationProposalListProjection();
  return () => {
    expect(getSessionScriptBlockEditDraft('script.document.main', 'F')).toBe(draft);
    expect(draft.state.editing).toBe(true);
    expect(draft.state.text).toBe('Unrelated draft: Eli pockets a brass whistle.');
    expect(draft.state.baseRevision).toBe('F-revision');
    expect(getCachedPropagationProposalListProjection()).toBe(proposal);
    expect(proposal?.payload.proposals[0]?.status).toBe('pending');
  };
}

describe('canonical selected inspector event freshness', () => {
  it('rereads screenplay impact after an arc event without rebasing retained work', async () => {
    const assertRetained = await retainedWork();
    const draft = getSessionTimelinePlacementDraft(initial.payload.node!);
    draft.initialize(initial.payload.node!.range_read);
    draft.state.start = '600';
    draft.state.end = '630';
    vi.mocked(invalidateScriptContext).mockClear();
    vi.mocked(refreshScriptDocumentProjection).mockClear();
    vi.mocked(refreshCurrentContextStackProjection).mockClear();
    await events.emit({ type: 'story_changed' });
    expect(invalidateScriptContext).toHaveBeenCalledTimes(1);
    expect(refreshScriptDocumentProjection).toHaveBeenCalledWith({
      document_id: 'script.document.main',
    });
    expect(refreshCurrentContextStackProjection).toHaveBeenCalledTimes(1);
    expect(draft.state.base?.node_revision_event_id).toBe('range-1');
    expect([draft.state.start, draft.state.end]).toEqual(['600', '630']);
    expect(editorState.selectedNodeId).toBe('B');
    assertRetained();
  });

  it.each(['selection', 'session', 'teardown'] as const)(
    'completes generation while inspector is stalled, then discards late success after %s changes',
    async (change) => {
      const assertRetained = await retainedWork();
      let resolve!: (projection: typeof updated) => void;
      read.mockReturnValueOnce(
        new Promise((done) => {
          resolve = done;
        }),
      );
      startGeneration('B');
      const completion = events.emit({ type: 'generation_complete', node_id: 'B' });
      try {
        await vi.waitFor(() => expect(read).toHaveBeenCalledTimes(1));
        await vi.waitFor(() => expect(editorState.streamingNodeId).toBeNull());
        expect(selectedNodeEditorProjectionState.pending).toBe(true);
        assertRetained();
        if (change === 'selection') editorState.selectedNodeId = 'F';
        if (change === 'session') {
          resetEditorState();
          editorState.selectedNodeId = 'B';
          startGeneration('B');
        }
        if (change === 'teardown') teardown();
      } finally {
        resolve(updated);
      }
      await completion;
      await vi.waitFor(() => expect(selectedNodeEditorProjectionState.pending).toBe(false));
      expect(selectedNodeEditorProjectionState.projection).toEqual(initial);
      if (change === 'session') expect(editorState.streamingNodeId).toBe('B');
    },
  );

  it.each(['selection', 'session', 'teardown'] as const)(
    'completes generation while inspector is stalled, then discards late failure after %s changes',
    async (change) => {
      let reject!: (error: Error) => void;
      read.mockReturnValueOnce(
        new Promise((_, fail) => {
          reject = fail;
        }),
      );
      startGeneration('B');
      const completion = events.emit({ type: 'generation_complete', node_id: 'B' });
      try {
        await vi.waitFor(() => expect(read).toHaveBeenCalledTimes(1));
        await vi.waitFor(() => expect(editorState.streamingNodeId).toBeNull());
        if (change === 'selection') editorState.selectedNodeId = 'F';
        if (change === 'session') {
          resetEditorState();
          editorState.selectedNodeId = 'B';
          startGeneration('B');
        }
        if (change === 'teardown') teardown();
      } finally {
        reject(new Error('Old selected read failed'));
      }
      await completion;
      await vi.waitFor(() => expect(selectedNodeEditorProjectionState.pending).toBe(false));
      expect(selectedNodeEditorProjectionState.error).toBeUndefined();
      expect(selectedNodeEditorProjectionState.projection).toEqual(initial);
      if (change === 'session') expect(editorState.streamingNodeId).toBe('B');
    },
  );

  it('reports a current inspector failure after generation has completed', async () => {
    let reject!: (error: Error) => void;
    read.mockReturnValueOnce(
      new Promise((_, fail) => {
        reject = fail;
      }),
    );
    startGeneration('B');
    const completion = events.emit({ type: 'generation_complete', node_id: 'B' });
    try {
      await vi.waitFor(() => expect(read).toHaveBeenCalledTimes(1));
      await vi.waitFor(() => expect(editorState.streamingNodeId).toBeNull());
    } finally {
      reject(new Error('Canonical selected read failed'));
    }
    await completion;
    await vi.waitFor(() =>
      expect(selectedNodeEditorProjectionState.error).toBe('Canonical selected read failed'),
    );
    expect(selectedNodeEditorProjectionState.pending).toBe(false);
  });

  it.each(['generation_complete', 'node_updated'] as const)(
    'refreshes the selected caption on %s while retaining an unrelated draft and pending proposal',
    async (type) => {
      const assertRetained = await retainedWork();
      startGeneration('B');
      await events.emit({ type, node_id: 'B' });
      expect(read).toHaveBeenCalledWith({ node_id: 'B' });
      expect(
        beatContentStatusLabel(
          selectedNodeEditorProjectionState.projection!.payload.node!.content_status,
        ),
      ).toBe('Has content');
      assertRetained();
      if (type === 'generation_complete') expect(editorState.streamingNodeId).toBeNull();
    },
  );

  it.each(['timeline_changed', 'hierarchy_changed'] as const)(
    'refreshes canonical placement on %s without rebasing the placement intent',
    async (type) => {
      const assertRetained = await retainedWork();
      const draft = getSessionTimelinePlacementDraft(initial.payload.node!);
      draft.initialize(initial.payload.node!.range_read);
      draft.state.start = '600';
      draft.state.end = '630';
      await events.emit({ type });
      const node = selectedNodeEditorProjectionState.projection!.payload.node!;
      expect([node.start_ms, node.end_ms]).toEqual([180000, 210000]);
      expect(getSessionTimelinePlacementDraft(node)).toBe(draft);
      expect([draft.state.start, draft.state.end]).toEqual(['600', '630']);
      expect(draft.state.base?.node_revision_event_id).toBe('range-1');
      assertRetained();
    },
  );

  it('does not refresh or retarget the inspector for an unrelated node update', async () => {
    await events.emit({ type: 'node_updated', node_id: 'F' });
    expect(read).not.toHaveBeenCalled();
    expect(editorState.selectedNodeId).toBe('B');
    expect(selectedNodeEditorProjectionState.projection).toEqual(initial);
  });

  it('abandons queued reads after selection changes', async () => {
    const refresh = events.emit({ type: 'generation_complete', node_id: 'B' });
    editorState.selectedNodeId = 'F';
    await refresh;
    expect(read).not.toHaveBeenCalled();
    expect(editorState.selectedNodeId).toBe('F');
  });

  it.each(['selection', 'session', 'teardown'] as const)(
    'ignores an in-flight response after %s changes',
    async (change) => {
      let resolve!: (projection: typeof updated) => void;
      read.mockReturnValueOnce(
        new Promise((done) => {
          resolve = done;
        }),
      );
      const refresh = events.emit({ type: 'generation_complete', node_id: 'B' });
      await vi.waitFor(() => expect(read).toHaveBeenCalledTimes(1));
      if (change === 'selection') editorState.selectedNodeId = 'F';
      if (change === 'session') {
        resetEditorState();
        editorState.selectedNodeId = 'B';
        startGeneration('B');
      }
      if (change === 'teardown') teardown();
      resolve(updated);
      await refresh;
      expect(selectedNodeEditorProjectionState.projection).toEqual(initial);
      expect(selectedNodeEditorProjectionState.pending).toBe(false);
      if (change === 'session') expect(editorState.streamingNodeId).toBe('B');
    },
  );

  it('coalesces generation and node events into one canonical selected read', async () => {
    await Promise.all([
      events.emit({ type: 'generation_complete', node_id: 'B' }),
      events.emit({ type: 'node_updated', node_id: 'B' }),
      events.emit({ type: 'timeline_changed' }),
    ]);
    expect(read).toHaveBeenCalledTimes(1);
    expect(selectedNodeEditorProjectionState.projection).toEqual(updated);
  });

  it('retains the newer selected projection when the latest event read is older', async () => {
    await refreshSelectedNodeEditorProjection('B');
    read.mockResolvedValueOnce(initial);
    await events.emit({ type: 'node_updated', node_id: 'B' });
    expect(selectedNodeEditorProjectionState.projection).toEqual(updated);
  });

  it('keeps the newly selected scene when the previous scene read settles late', async () => {
    let resolve!: (projection: typeof updated) => void;
    read.mockReturnValueOnce(
      new Promise((done) => {
        resolve = done;
      }),
    );
    const oldRead = events.emit({ type: 'node_updated', node_id: 'B' });
    await vi.waitFor(() => expect(read).toHaveBeenCalledTimes(1));
    editorState.selectedNodeId = 'F';
    const selectedF = {
      ...updated,
      version: 3,
      payload: {
        ...updated.payload,
        node: { ...updated.payload.node!, node_id: 'F', name: 'Scene F' },
      },
    };
    read.mockResolvedValueOnce(selectedF);
    await refreshSelectedNodeEditorProjection('F');
    resolve(updated);
    await oldRead;
    expect(selectedNodeEditorProjectionState.selectedNodeId).toBe('F');
    expect(selectedNodeEditorProjectionState.projection).toEqual(selectedF);
  });

  it('does not publish an event read error after selection changes', async () => {
    let reject!: (error: Error) => void;
    read.mockReturnValueOnce(
      new Promise((_, fail) => {
        reject = fail;
      }),
    );
    const oldRead = events.emit({ type: 'node_updated', node_id: 'B' });
    await vi.waitFor(() => expect(read).toHaveBeenCalledTimes(1));
    editorState.selectedNodeId = 'F';
    reject(new Error('Old selected read failed'));
    await oldRead;
    expect(selectedNodeEditorProjectionState.error).toBeUndefined();
    expect(selectedNodeEditorProjectionState.pending).toBe(false);
  });
});
