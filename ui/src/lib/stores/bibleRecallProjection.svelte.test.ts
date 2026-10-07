import { beforeEach, expect, it, vi } from 'vitest';
import { getBibleRecallProjection } from '$lib/projectionApi.js';
import type { BibleRecallRequest, BibleRecallProjection } from '$lib/bibleRecallTypes.js';
import type { ProjectionEnvelope } from '$lib/projectionTypes.js';
import {
  bibleRecallState,
  clearBibleRecall,
  invalidateBibleRecall,
  recallBibleFacts,
  selectBibleRecallAnchor,
} from './bibleRecallProjection.svelte.js';
import { editorState, resetEditorState } from './editor.svelte.js';
import { getSessionScriptBlockEditDraft } from './scriptBlockEditSession.svelte.js';
import { propagationProposalProjectionState } from './propagationProposalProjection.svelte.js';
import { selectBibleGraphNode } from './bible.svelte.js';

vi.mock('$lib/projectionApi.js', () => ({ getBibleRecallProjection: vi.fn() }));
const read = vi.mocked(getBibleRecallProjection);
const query: BibleRecallRequest = {
  anchor_node_id: 'Mara',
  story_time_ms: null,
  direction: 'both',
  edge_kinds: [],
  neighbor_limit: 8,
};
function evidence(q = query, version = 8): ProjectionEnvelope<BibleRecallProjection> {
  return {
    version,
    change_event_id: `event-${version}`,
    payload: {
      request: { ...q },
      nodes: [
        {
          node_id: q.anchor_node_id,
          name: 'Mara',
          schema_key: 'character',
          name_revision_event_id: 'name-write',
          fields: [
            {
              part_key: 'profile',
              field_key: 'tagline',
              value: { type: 'text', value: 'blue' },
              source: {
                kind: 'baseline',
                field_id: 'Mara.tagline',
                revision_event_id: 'fact-write',
              },
            },
          ],
          unresolved_timed_fields: [],
          omitted_fields: 0,
        },
      ],
      paths: [],
      omitted_neighbors: 0,
      omitted_edges: 0,
      relationships_untimed: true,
    },
  };
}
function deferred() {
  let resolve!: (value: ProjectionEnvelope<BibleRecallProjection>) => void;
  let reject!: (value: Error) => void;
  const promise = new Promise<ProjectionEnvelope<BibleRecallProjection>>((yes, no) => {
    resolve = yes;
    reject = no;
  });
  return { promise, resolve, reject };
}
beforeEach(() => {
  read.mockReset();
  clearBibleRecall();
  resetEditorState();
});

it('requires an explicit read and displays only the requested evidence without changing generation context', async () => {
  editorState.generationContext = {
    system: 'Exact current system',
    user: 'Current generation input',
  };
  const context = { ...editorState.generationContext };
  selectBibleRecallAnchor('Mara');
  expect(read).not.toHaveBeenCalled();
  read.mockResolvedValue(evidence());
  await recallBibleFacts(query);
  expect(read).toHaveBeenCalledExactlyOnceWith(query);
  expect(bibleRecallState.projection).toEqual(evidence());
  expect(editorState.generationContext).toEqual(context);
});

it.each(['mutation', 'anchor', 'dispose', 'session'] as const)(
  'withholds late success and errors after %s',
  async (reason) => {
    for (const failure of [false, true]) {
      clearBibleRecall();
      const pending = deferred();
      read.mockReturnValueOnce(pending.promise);
      const work = recallBibleFacts(query);
      if (reason === 'mutation') invalidateBibleRecall();
      if (reason === 'anchor') selectBibleRecallAnchor('BeachHouse');
      if (reason === 'dispose') clearBibleRecall();
      if (reason === 'session') resetEditorState();
      if (failure) pending.reject(new Error('obsolete error'));
      else pending.resolve(evidence());
      await work;
      expect(bibleRecallState.projection).toBeNull();
      expect(bibleRecallState.error).toBeNull();
    }
  },
);

it('keeps the newest explicit time/query result when requests resolve in reverse order', async () => {
  const old = deferred();
  const current = deferred();
  read.mockReturnValueOnce(old.promise).mockReturnValueOnce(current.promise);
  const one = recallBibleFacts(query);
  const timed = { ...query, story_time_ms: 1000, direction: 'outgoing' as const };
  const two = recallBibleFacts(timed);
  current.resolve(evidence(timed, 9));
  await two;
  old.resolve(evidence(query, 8));
  await one;
  expect(bibleRecallState.projection).toEqual(evidence(timed, 9));
});

it('invalidates evidence on writes and refuses an older revision or mismatched source query', async () => {
  read.mockResolvedValueOnce(evidence(query, 10));
  await recallBibleFacts(query);
  invalidateBibleRecall(11);
  expect(bibleRecallState.projection).toBeNull();
  expect(bibleRecallState.invalidated).toBe(true);
  read.mockResolvedValueOnce(evidence(query, 10));
  await recallBibleFacts(query);
  expect(bibleRecallState.projection).toBeNull();
  expect(bibleRecallState.error).toContain('stale');
  read.mockResolvedValueOnce(evidence({ ...query, story_time_ms: 1000 }, 12));
  await recallBibleFacts(query);
  expect(bibleRecallState.projection).toBeNull();
  expect(bibleRecallState.error).toContain('stale');
  read.mockResolvedValueOnce(evidence(query, 12));
  await recallBibleFacts(query);
  expect(bibleRecallState.projection?.version).toBe(12);
  expect(bibleRecallState.invalidated).toBe(false);
});

it('reports an active missing/deleted anchor without fabricating a projection', async () => {
  read.mockRejectedValueOnce(new Error('Recall anchor no longer exists'));
  await recallBibleFacts(query);
  expect(bibleRecallState.error).toBe('Recall anchor no longer exists');
  expect(bibleRecallState.pending).toBe(false);
  expect(bibleRecallState.projection).toBeNull();
});

it('selection revokes a pending read immediately and leaves a manual edit and pending proposal intact', async () => {
  const draft = getSessionScriptBlockEditDraft('recall-document', 'recall-block');
  draft.begin({
    block: {
      id: 'recall-block',
      segment_id: 'segment.F',
      block_kind: 'action',
      text: 'Saved F',
      sort_order: 0,
    },
    revision_event_id: 'saved-F-write',
    spans: [],
    locks: [],
  });
  draft.state.text = '  Unsaved F — 雨\n\n';
  propagationProposalProjectionState.projection = {
    version: 2,
    payload: {
      proposals: [
        {
          id: 'pending-review',
          action: 'patch_script_block',
          target: { kind: 'script_block', block_id: 'recall-block' },
          status: 'pending',
          summary: 'Explicit pending review',
          proposed_text: 'Proposed F',
          created_at_ms: 1,
        },
      ],
    },
  };
  const proposal = JSON.stringify(propagationProposalProjectionState.projection);
  const pending = deferred();
  read.mockReturnValueOnce(pending.promise);
  const work = recallBibleFacts(query);
  selectBibleGraphNode('BeachHouse');
  pending.resolve(evidence());
  await work;
  expect(bibleRecallState.anchor).toBe('BeachHouse');
  expect(bibleRecallState.projection).toBeNull();
  invalidateBibleRecall();
  expect(getSessionScriptBlockEditDraft('recall-document', 'recall-block')).toBe(draft);
  expect(draft.state.text).toBe('  Unsaved F — 雨\n\n');
  expect(draft.state.baseRevision).toBe('saved-F-write');
  expect(draft.state.editing).toBe(true);
  expect(JSON.stringify(propagationProposalProjectionState.projection)).toBe(proposal);
});
