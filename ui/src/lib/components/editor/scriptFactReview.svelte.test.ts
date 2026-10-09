import { beforeEach, expect, it, vi } from 'vitest';
import { render } from 'svelte/server';
import ScriptFactReconciliation from './ScriptFactReconciliation.svelte';
import { createScriptFactReview } from './scriptFactReview.svelte.js';
import {
  requestScriptFactProposal,
  acceptPropagationProposal,
  rejectPropagationProposal,
} from '$lib/commandApi.js';
import {
  clearPropagationProposalListProjection,
  propagationProposalProjectionState,
  applyRequestScriptFactProposalCommand,
  applyScriptFactDecisionCommand,
} from '$lib/stores/propagationProposalProjection.svelte.js';
import {
  getSessionScriptBlockEditDraft,
  resetSessionScriptBlockEditDrafts,
} from '$lib/stores/scriptBlockEditSession.svelte.js';
import type { ScriptSegmentProjection } from '$lib/scriptTypes.js';
import type { ScriptFactEditEvidence } from '$lib/scriptFactTypes.js';
vi.mock('$lib/commandApi.js', () => ({
  requestScriptFactProposal: vi.fn(),
  acceptPropagationProposal: vi.fn(),
  rejectPropagationProposal: vi.fn(),
  requestScriptImpactProposal: vi.fn(),
  createPropagationProposal: vi.fn(),
  updatePropagationProposal: vi.fn(),
}));
vi.mock('$lib/projectionApi.js', () => ({ getPropagationProposalListProjection: vi.fn() }));

const evidence: ScriptFactEditEvidence = {
  document_id: 'main',
  segment_id: 'B',
  block_id: 'block.B',
  before_revision_event_id: 'before.B',
  revision_event_id: 'saved.B',
  segment_revision_event_id: 'segment.B',
  before_text: '  Before exact — 雨\n\n',
  text: '  After exact — 雪\n\n  ',
  start_ms: 1000,
  end_ms: 2000,
  generation_event_id: 'generation.B',
  facts: [
    {
      dependency_id: 'consumed.Mara',
      node_id: 'Mara',
      part_key: 'profile',
      field_key: 'tagline',
      field_id: 'Mara.tagline',
      consumed_revision_event_id: 'consumed.fact',
      consumed_text: 'Original fact',
      revision_event_id: 'current.fact',
      text: 'Current exact fact',
    },
  ],
};
const payload = {
  document_id: 'main',
  segment_id: 'B',
  block_id: 'block.B',
  expected_block_revision_event_id: 'saved.B',
  expected_field_revision_event_id: 'current.fact',
  generation_event_id: 'generation.B',
  dependency_id: 'consumed.Mara',
};
const empty = {
  outcome: 'recorded' as const,
  projection: { version: 2, payload: { proposals: [] } },
};
beforeEach(() => {
  vi.clearAllMocks();
  clearPropagationProposalListProjection();
  resetSessionScriptBlockEditDrafts();
});

it('shows actual consumed choices and exact source/fact before-after evidence without automatic analysis', () => {
  propagationProposalProjectionState.projection = {
    version: 1,
    payload: {
      proposals: [
        {
          id: 'fact.pending',
          action: 'set_bible_field',
          target: {
            kind: 'bible_field',
            node_id: 'Mara',
            part_key: 'profile',
            field_key: 'tagline',
            field_id: 'Mara.tagline',
          },
          status: 'pending',
          summary: 'Synthetic proposed fact',
          proposed_value: { type: 'text', value: 'Proposed fact — 雨\n\n' },
          rationale: 'Synthetic fixture; no model claim',
          created_at_ms: 1,
          script_fact_binding: {
            request: { ...payload, proposal_id: 'fact.pending' },
            edit: evidence,
            context_dependencies: [],
          },
        },
      ],
    },
  };
  const segment: ScriptSegmentProjection = {
    segment: {
      id: 'B',
      document_id: 'main',
      start_ms: 1000,
      end_ms: 2000,
      status: 'current',
      sort_order: 0,
    },
    blocks: [],
    impact: {
      generation_event_id: 'generation.B',
      lineage_available: true,
      needs_review: false,
      causes: [],
      fact_edit: evidence,
    },
  };
  const body = render(ScriptFactReconciliation, { props: { documentId: 'main', segment } }).body;
  for (const text of [
    'Consumed baseline fact',
    'Choose a consumed fact',
    'Mara · profile.tagline',
    'Before exact — 雨',
    'After exact — 雪',
    'Current exact fact',
    'Proposed fact — 雨',
    'Accept fact update',
    'Reject fact update',
    'before.B',
    'saved.B',
    'current.fact',
  ])
    expect(body).toContain(text);
  expect(body).not.toContain('Unconsumed Eli');
  expect(requestScriptFactProposal).not.toHaveBeenCalled();
});

it('preserves exact uncertain analysis identity; a new saved revision retires old intent', async () => {
  let owner = 'saved.B/current.fact';
  const review = createScriptFactReview(() => owner);
  vi.mocked(requestScriptFactProposal)
    .mockRejectedValueOnce(new Error('Lost acknowledgement'))
    .mockResolvedValue(empty);
  await review.analyze(payload);
  expect(review.state.retry).toBe(true);
  const first = vi.mocked(requestScriptFactProposal).mock.calls[0]!;
  await review.analyze(payload);
  expect(vi.mocked(requestScriptFactProposal).mock.calls[1]).toEqual(first);
  vi.mocked(requestScriptFactProposal).mockRejectedValueOnce(new Error('Lost acknowledgement'));
  await review.analyze(payload);
  const old = vi.mocked(requestScriptFactProposal).mock.calls[2]!;
  owner = 'later.B/current.fact';
  review.observe();
  expect(review.state.retry).toBe(false);
  await review.analyze({ ...payload, expected_block_revision_event_id: 'later.B' });
  const fresh = vi.mocked(requestScriptFactProposal).mock.calls[3]!;
  expect(fresh[0].proposal_id).not.toBe(old[0].proposal_id);
  expect(fresh[1]).not.toBe(old[1]);
});

it('keeps source and unrelated unsaved drafts during analysis, rejection, acceptance and failed retries', async () => {
  const draft = (id: string) => {
    const d = getSessionScriptBlockEditDraft('main', id);
    d.begin({
      block: { id, segment_id: id, block_kind: 'action', text: 'Saved ' + id, sort_order: 0 },
      revision_event_id: 'saved.' + id,
      spans: [],
      locks: [],
    });
    d.state.text = '  Draft ' + id + ' — 雨\n\n';
    return d;
  };
  const own = draft('B'),
    other = draft('F');
  let owner = 'owner';
  const review = createScriptFactReview(() => owner);
  vi.mocked(requestScriptFactProposal).mockResolvedValue(empty);
  vi.mocked(rejectPropagationProposal).mockResolvedValue(empty);
  vi.mocked(acceptPropagationProposal)
    .mockRejectedValueOnce(new Error('Lost acceptance acknowledgement'))
    .mockResolvedValue(empty);
  await review.analyze(payload);
  await review.decide('fact.pending', false);
  await review.decide('fact.fresh', true);
  owner = 'fact.changed.after.acceptance';
  await review.decide('fact.fresh', true);
  expect(vi.mocked(acceptPropagationProposal).mock.calls[1]).toEqual(
    vi.mocked(acceptPropagationProposal).mock.calls[0],
  );
  expect(own.state.text).toBe('  Draft B — 雨\n\n');
  expect(other.state.text).toBe('  Draft F — 雨\n\n');
});

it('refuses late analysis and decision results after the proposal owner is reset', async () => {
  for (const accept of [null, true, false]) {
    let resolve!: (value: typeof empty) => void;
    const promise = new Promise<typeof empty>((r) => (resolve = r));
    if (accept === null) vi.mocked(requestScriptFactProposal).mockReturnValueOnce(promise);
    else if (accept) vi.mocked(acceptPropagationProposal).mockReturnValueOnce(promise);
    else vi.mocked(rejectPropagationProposal).mockReturnValueOnce(promise);
    const pending =
      accept === null
        ? applyRequestScriptFactProposalCommand({ ...payload, proposal_id: 'old' }, 'old-command')
        : applyScriptFactDecisionCommand('old', accept, 'old-command');
    clearPropagationProposalListProjection();
    resolve(empty);
    await expect(pending).rejects.toThrow(/project changed/);
    expect(propagationProposalProjectionState.projection).toBeNull();
    expect(propagationProposalProjectionState.pending).toBe(false);
    expect(propagationProposalProjectionState.error).toBeUndefined();
  }
});

it('retired controls cannot submit analysis or a decision in a new project owner', async () => {
  const review = createScriptFactReview(() => 'old-source');
  clearPropagationProposalListProjection();
  await review.analyze(payload);
  await review.decide('old-proposal', true);
  expect(requestScriptFactProposal).not.toHaveBeenCalled();
  expect(acceptPropagationProposal).not.toHaveBeenCalled();
});
