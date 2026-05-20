import { beforeEach, describe, expect, it, vi } from 'vitest';

import {
  acceptBibleReferenceProposal,
  createBibleReferenceProposal,
  rejectBibleReferenceProposal,
} from '$lib/commandApi.js';
import { getBibleReferenceProposalListProjection } from '$lib/projectionApi.js';
import {
  applyAcceptBibleReferenceProposalCommand,
  applyCreateBibleReferenceProposalCommand,
  applyRejectBibleReferenceProposalCommand,
  clearBibleReferenceProposalListProjection,
  getCachedBibleReferenceProposalListProjection,
  refreshBibleReferenceProposalListProjection,
  semanticProposalProjectionState,
} from './semanticProposalProjection.svelte.js';

vi.mock('$lib/commandApi.js', () => ({
  acceptBibleReferenceProposal: vi.fn(),
  createBibleReferenceProposal: vi.fn(),
  rejectBibleReferenceProposal: vi.fn(),
}));

vi.mock('$lib/projectionApi.js', () => ({
  getBibleReferenceProposalListProjection: vi.fn(),
}));

const acceptBibleReferenceProposalMock = vi.mocked(acceptBibleReferenceProposal);
const createBibleReferenceProposalMock = vi.mocked(createBibleReferenceProposal);
const rejectBibleReferenceProposalMock = vi.mocked(rejectBibleReferenceProposal);
const getBibleReferenceProposalListProjectionMock = vi.mocked(
  getBibleReferenceProposalListProjection,
);

const projection = {
  version: 4,
  change_event_id: 'event-4',
  payload: {
    proposals: [
      {
        id: 'proposal-ada',
        source_node_id: 'node.scene.one',
        child_name: 'First encounter',
        reference_kind: 'character' as const,
        reference_text: 'Ada',
        proposed_schema_key: 'character',
        status: 'pending' as const,
        rationale: 'Named in the generated child plan',
        created_at_ms: 12_345,
      },
    ],
  },
};

const newerProjection = {
  ...projection,
  version: 6,
  change_event_id: 'event-6',
  payload: {
    proposals: [
      {
        id: 'proposal-bea',
        source_node_id: 'node.scene.two',
        child_name: 'Second encounter',
        reference_kind: 'character' as const,
        reference_text: 'Bea',
        proposed_schema_key: 'character',
        status: 'pending' as const,
        rationale: 'Newer proposal',
        created_at_ms: 67_890,
      },
    ],
  },
};

const olderProjection = {
  ...projection,
  version: 5,
  change_event_id: 'event-5',
  payload: {
    proposals: [
      {
        id: 'proposal-cal',
        source_node_id: 'node.scene.three',
        child_name: 'Third encounter',
        reference_kind: 'character' as const,
        reference_text: 'Cal',
        proposed_schema_key: 'character',
        status: 'pending' as const,
        rationale: 'Older proposal',
        created_at_ms: 56_789,
      },
    ],
  },
};

beforeEach(() => {
  clearBibleReferenceProposalListProjection();
  acceptBibleReferenceProposalMock.mockReset();
  createBibleReferenceProposalMock.mockReset();
  rejectBibleReferenceProposalMock.mockReset();
  getBibleReferenceProposalListProjectionMock.mockReset();
});

describe('semantic proposal projection store', () => {
  it('stores backend bible reference proposal projections', async () => {
    getBibleReferenceProposalListProjectionMock.mockResolvedValue(projection);

    await expect(refreshBibleReferenceProposalListProjection()).resolves.toEqual(projection);

    expect(getBibleReferenceProposalListProjectionMock).toHaveBeenCalledWith();
    expect(getCachedBibleReferenceProposalListProjection()).toEqual(projection);
    expect(semanticProposalProjectionState.pending).toBe(false);
    expect(semanticProposalProjectionState.error).toBeUndefined();
  });

  it('stores create, reject, and accept command response projections', async () => {
    createBibleReferenceProposalMock.mockResolvedValue({
      outcome: 'recorded',
      projection,
    });
    rejectBibleReferenceProposalMock.mockResolvedValue({
      outcome: 'recorded',
      projection,
    });
    acceptBibleReferenceProposalMock.mockResolvedValue({
      outcome: 'recorded',
      projection,
    });

    await applyCreateBibleReferenceProposalCommand(
      {
        proposal_id: 'proposal-ada',
        source_node_id: 'node.scene.one',
        child_name: 'First encounter',
        reference_kind: 'character',
        reference_text: 'Ada',
        rationale: 'Named in the generated child plan',
      },
      'command-create',
    );
    await applyRejectBibleReferenceProposalCommand(
      {
        proposal_id: 'proposal-ada',
        reason: 'Duplicate',
      },
      'command-reject',
    );
    await applyAcceptBibleReferenceProposalCommand(
      {
        proposal_id: 'proposal-ada',
        node_id: 'bible.character.ada',
        parent_id: 'bible.root.characters',
        name: 'Ada',
        sort_order: 20,
      },
      'command-accept',
    );

    expect(createBibleReferenceProposalMock).toHaveBeenCalledWith(
      {
        proposal_id: 'proposal-ada',
        source_node_id: 'node.scene.one',
        child_name: 'First encounter',
        reference_kind: 'character',
        reference_text: 'Ada',
        rationale: 'Named in the generated child plan',
      },
      'command-create',
    );
    expect(rejectBibleReferenceProposalMock).toHaveBeenCalledWith(
      {
        proposal_id: 'proposal-ada',
        reason: 'Duplicate',
      },
      'command-reject',
    );
    expect(acceptBibleReferenceProposalMock).toHaveBeenCalledWith(
      {
        proposal_id: 'proposal-ada',
        node_id: 'bible.character.ada',
        parent_id: 'bible.root.characters',
        name: 'Ada',
        sort_order: 20,
      },
      'command-accept',
    );
    expect(getCachedBibleReferenceProposalListProjection()).toEqual(projection);
  });

  it('records errors without replacing cached projections', async () => {
    getBibleReferenceProposalListProjectionMock.mockResolvedValue(projection);
    await refreshBibleReferenceProposalListProjection();
    getBibleReferenceProposalListProjectionMock.mockRejectedValue(
      new Error('proposals unavailable'),
    );

    await expect(refreshBibleReferenceProposalListProjection()).rejects.toThrow(
      'proposals unavailable',
    );

    expect(getCachedBibleReferenceProposalListProjection()).toEqual(projection);
    expect(semanticProposalProjectionState.pending).toBe(false);
    expect(semanticProposalProjectionState.error).toBe('proposals unavailable');
  });

  it('does not replace cached proposal projections with stale refresh results', async () => {
    getBibleReferenceProposalListProjectionMock.mockResolvedValueOnce(newerProjection);
    await refreshBibleReferenceProposalListProjection();
    getBibleReferenceProposalListProjectionMock.mockResolvedValueOnce(olderProjection);

    await expect(refreshBibleReferenceProposalListProjection()).resolves.toEqual(olderProjection);

    expect(getCachedBibleReferenceProposalListProjection()).toEqual(newerProjection);
    expect(semanticProposalProjectionState.pending).toBe(false);
    expect(semanticProposalProjectionState.error).toBeUndefined();
  });

  it('does not replace cached proposal projections with stale command responses', async () => {
    getBibleReferenceProposalListProjectionMock.mockResolvedValueOnce(newerProjection);
    await refreshBibleReferenceProposalListProjection();
    createBibleReferenceProposalMock.mockResolvedValue({
      outcome: 'recorded',
      projection: olderProjection,
    });

    await expect(
      applyCreateBibleReferenceProposalCommand(
        {
          proposal_id: 'proposal-cal',
          source_node_id: 'node.scene.one',
          child_name: 'First encounter',
          reference_kind: 'character',
          reference_text: 'Cal',
          rationale: 'Older command response',
        },
        'command-create-stale',
      ),
    ).resolves.toEqual({
      outcome: 'recorded',
      projection: olderProjection,
    });

    expect(getCachedBibleReferenceProposalListProjection()).toEqual(newerProjection);
    expect(semanticProposalProjectionState.pending).toBe(false);
    expect(semanticProposalProjectionState.error).toBeUndefined();
  });
});
