import { beforeEach, describe, expect, it, vi } from 'vitest';

import {
  acceptAffectProposal,
  createAffectProposal,
  rejectAffectProposal,
} from '$lib/commandApi.js';
import { getAffectProposalListProjection } from '$lib/projectionApi.js';
import {
  affectProposalProjectionState,
  applyAcceptAffectProposalCommand,
  applyCreateAffectProposalCommand,
  applyRejectAffectProposalCommand,
  clearAffectProposalListProjection,
  getCachedAffectProposalListProjection,
  refreshAffectProposalListProjection,
} from './affectProposalProjection.svelte.js';

vi.mock('$lib/commandApi.js', () => ({
  acceptAffectProposal: vi.fn(),
  createAffectProposal: vi.fn(),
  rejectAffectProposal: vi.fn(),
}));

vi.mock('$lib/projectionApi.js', () => ({
  getAffectProposalListProjection: vi.fn(),
}));

const acceptAffectProposalMock = vi.mocked(acceptAffectProposal);
const createAffectProposalMock = vi.mocked(createAffectProposal);
const rejectAffectProposalMock = vi.mocked(rejectAffectProposal);
const getAffectProposalListProjectionMock = vi.mocked(getAffectProposalListProjection);

const proposalPayload = {
  proposal_id: 'proposal.affect.scene-weather',
  source: 'manual_script_edit' as const,
  proposed_value: {
    id: 'affect.scene-weather',
    target: { type: 'project' as const },
    valence: -100,
    arousal: 650,
    intensity: 700,
    confidence: 900,
    mood_labels: ['rainy'],
    provenance: 'script_edit_detected' as const,
    rationale: 'Manual edit changed the weather.',
  },
  summary: 'Detected rainy scene affect',
  rationale: 'Script edit introduced rain.',
  source_event_id: null,
};

const projection = {
  version: 4,
  change_event_id: 'event-4',
  payload: {
    proposals: [
      {
        id: proposalPayload.proposal_id,
        status: 'pending' as const,
        source: proposalPayload.source,
        proposed_value: proposalPayload.proposed_value,
        summary: proposalPayload.summary,
        rationale: proposalPayload.rationale,
        source_event_id: proposalPayload.source_event_id,
        created_at_ms: 12_345,
      },
    ],
  },
};

const olderProjection = {
  ...projection,
  version: 3,
  change_event_id: 'event-3',
};

beforeEach(() => {
  clearAffectProposalListProjection();
  acceptAffectProposalMock.mockReset();
  createAffectProposalMock.mockReset();
  rejectAffectProposalMock.mockReset();
  getAffectProposalListProjectionMock.mockReset();
});

describe('affect proposal projection store', () => {
  it('stores backend affect proposal projections', async () => {
    getAffectProposalListProjectionMock.mockResolvedValue(projection);

    await expect(refreshAffectProposalListProjection()).resolves.toEqual(projection);

    expect(getAffectProposalListProjectionMock).toHaveBeenCalledWith();
    expect(getCachedAffectProposalListProjection()).toEqual(projection);
    expect(affectProposalProjectionState.pending).toBe(false);
    expect(affectProposalProjectionState.error).toBeUndefined();
  });

  it('stores create, reject, and accept command response projections', async () => {
    createAffectProposalMock.mockResolvedValue(projection);
    rejectAffectProposalMock.mockResolvedValue(projection);
    acceptAffectProposalMock.mockResolvedValue(projection);

    await applyCreateAffectProposalCommand(proposalPayload, 'command-create');
    await applyRejectAffectProposalCommand(
      {
        proposal_id: proposalPayload.proposal_id,
        reason: 'Wrong scope',
      },
      'command-reject',
    );
    await applyAcceptAffectProposalCommand(
      {
        proposal_id: proposalPayload.proposal_id,
      },
      'command-accept',
    );

    expect(createAffectProposalMock).toHaveBeenCalledWith(proposalPayload, 'command-create');
    expect(rejectAffectProposalMock).toHaveBeenCalledWith(
      {
        proposal_id: proposalPayload.proposal_id,
        reason: 'Wrong scope',
      },
      'command-reject',
    );
    expect(acceptAffectProposalMock).toHaveBeenCalledWith(
      {
        proposal_id: proposalPayload.proposal_id,
      },
      'command-accept',
    );
    expect(getCachedAffectProposalListProjection()).toEqual(projection);
    expect(affectProposalProjectionState.pending).toBe(false);
    expect(affectProposalProjectionState.error).toBeUndefined();
  });

  it('keeps newer cached projections over older command responses', async () => {
    getAffectProposalListProjectionMock.mockResolvedValue(projection);
    rejectAffectProposalMock.mockResolvedValue(olderProjection);

    await refreshAffectProposalListProjection();
    await applyRejectAffectProposalCommand({
      proposal_id: proposalPayload.proposal_id,
    });

    expect(getCachedAffectProposalListProjection()).toEqual(projection);
  });

  it('records projection errors and clears pending state', async () => {
    const error = new Error('desktop unavailable');
    getAffectProposalListProjectionMock.mockRejectedValue(error);

    await expect(refreshAffectProposalListProjection()).rejects.toThrow(error);

    expect(affectProposalProjectionState.error).toBe('desktop unavailable');
    expect(affectProposalProjectionState.pending).toBe(false);
  });
});
