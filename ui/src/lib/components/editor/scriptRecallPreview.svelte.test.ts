import { beforeEach, expect, it, vi } from 'vitest';
import { packet } from './scriptRecallSelection.fixture.js';
import { createScriptRecallDraft } from './scriptRecallDraft.svelte.js';
import {
  getSessionScriptBlockEditDraft,
  resetSessionScriptBlockEditDrafts,
} from '$lib/stores/scriptBlockEditSession.svelte.js';
import {
  applyRequestScriptImpactProposalCommand,
  clearPropagationProposalListProjection,
  propagationProposalProjectionState,
} from '$lib/stores/propagationProposalProjection.svelte.js';
import { requestScriptImpactProposal } from '$lib/commandApi.js';

vi.mock('$lib/commandApi.js', () => ({
  requestScriptImpactProposal: vi.fn(),
  acceptPropagationProposal: vi.fn(),
  rejectPropagationProposal: vi.fn(),
  createPropagationProposal: vi.fn(),
  updatePropagationProposal: vi.fn(),
}));

beforeEach(() => {
  vi.clearAllMocks();
  resetSessionScriptBlockEditDrafts();
  clearPropagationProposalListProjection();
});

it('submits copied exact selectors through the existing command store and preserves independent manual drafts and pending proposals', async () => {
  const own = getSessionScriptBlockEditDraft('main', 'B');
  const other = getSessionScriptBlockEditDraft('main', 'F');
  const block = (id: string) => ({
    block: {
      id,
      segment_id: 'segment.' + id,
      block_kind: 'action' as const,
      text: 'Saved ' + id,
      sort_order: 0,
    },
    revision_event_id: 'saved.' + id,
    spans: [],
    locks: [],
  });
  own.begin(block('B'));
  own.state.text = 'Own unsaved draft — 雨\n\n';
  other.begin(block('F'));
  other.state.text = 'Unrelated unsaved draft — whistle\n\n';
  const prior = {
    id: 'previous',
    action: 'patch_script_block' as const,
    target: { kind: 'script_block' as const, block_id: 'B' },
    status: 'pending' as const,
    summary: 'Previous proposal',
    proposed_text: 'Previous exact text',
    created_at_ms: 1,
  };
  const projection = { version: 1, payload: { proposals: [prior] } };
  propagationProposalProjectionState.projection = projection;
  const selected = createScriptRecallDraft();
  const evidence = packet();
  selected.select('B/session1', evidence, 'house.0', true);
  vi.mocked(requestScriptImpactProposal).mockResolvedValue({
    outcome: 'recorded',
    projection: { version: 2, payload: { proposals: [prior] } },
  });
  const payload = {
    proposal_id: 'fresh',
    document_id: 'main',
    segment_id: 'segment.B',
    block_id: 'B',
    expected_block_revision_event_id: 'saved.B',
    generation_event_id: 'generation.B',
    dependency_id: 'cause.B',
    story_time_ms: null,
    recall_selection: selected.request('B/session1', evidence)!,
  };
  await applyRequestScriptImpactProposalCommand(payload, 'command.fresh');
  expect(requestScriptImpactProposal).toHaveBeenCalledWith(payload, 'command.fresh');
  expect(payload.recall_selection.facts[0]!.revision_event_id).toBe('revision.0');
  expect(payload.recall_selection.facts[0]).not.toHaveProperty('value');
  expect(own.state.text).toBe('Own unsaved draft — 雨\n\n');
  expect(other.state.text).toBe('Unrelated unsaved draft — whistle\n\n');
  expect(propagationProposalProjectionState.projection!.payload.proposals[0]).toEqual(prior);
  vi.mocked(requestScriptImpactProposal).mockRejectedValue(
    new Error('Selected recall evidence is stale'),
  );
  await expect(
    applyRequestScriptImpactProposalCommand({ ...payload, proposal_id: 'refused' }),
  ).rejects.toThrow(/stale/);
  expect(own.state.text).toBe('Own unsaved draft — 雨\n\n');
  expect(other.state.text).toBe('Unrelated unsaved draft — whistle\n\n');
  expect(propagationProposalProjectionState.projection!.payload.proposals[0]).toEqual(prior);
});
