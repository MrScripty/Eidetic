import { beforeEach, describe, expect, it, vi } from 'vitest';

import {
  acceptPropagationProposal,
  createPropagationProposal,
  rejectPropagationProposal,
  updatePropagationProposal,
} from '$lib/commandApi.js';
import { getPropagationProposalListProjection } from '$lib/projectionApi.js';
import {
  applyAcceptPropagationProposalCommand,
  applyCreatePropagationProposalCommand,
  applyRejectPropagationProposalCommand,
  applyUpdatePropagationProposalCommand,
  clearPropagationProposalListProjection,
  getCachedPropagationProposalListProjection,
  propagationProposalProjectionState,
  refreshPropagationProposalListProjection,
} from './propagationProposalProjection.svelte.js';

vi.mock('$lib/commandApi.js', () => ({
  acceptPropagationProposal: vi.fn(),
  createPropagationProposal: vi.fn(),
  rejectPropagationProposal: vi.fn(),
  updatePropagationProposal: vi.fn(),
}));

vi.mock('$lib/projectionApi.js', () => ({
  getPropagationProposalListProjection: vi.fn(),
}));

const acceptPropagationProposalMock = vi.mocked(acceptPropagationProposal);
const createPropagationProposalMock = vi.mocked(createPropagationProposal);
const rejectPropagationProposalMock = vi.mocked(rejectPropagationProposal);
const updatePropagationProposalMock = vi.mocked(updatePropagationProposal);
const getPropagationProposalListProjectionMock = vi.mocked(getPropagationProposalListProjection);

const projection = {
  version: 4,
  change_event_id: 'event-4',
  payload: {
    proposals: [
      {
        id: 'proposal.propagation.weather',
        action: 'set_bible_field' as const,
        target: {
          kind: 'bible_field' as const,
          node_id: 'node.location.harbor',
          part_key: 'environment',
          field_key: 'weather',
        },
        status: 'pending' as const,
        summary: 'Set harbor weather to rainy',
        proposed_value: { type: 'text' as const, value: 'rainy' },
        source_dependency_id: 'dependency.weather.scene',
        rationale: 'Manual edit introduced rainy weather',
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
        id: 'proposal.propagation.fog',
        action: 'set_bible_field' as const,
        target: {
          kind: 'bible_field' as const,
          node_id: 'node.location.harbor',
          part_key: 'environment',
          field_key: 'weather',
        },
        status: 'pending' as const,
        summary: 'Set harbor weather to foggy',
        proposed_value: { type: 'text' as const, value: 'foggy' },
        source_dependency_id: 'dependency.weather.scene',
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
        id: 'proposal.propagation.sun',
        action: 'set_bible_field' as const,
        target: {
          kind: 'bible_field' as const,
          node_id: 'node.location.harbor',
          part_key: 'environment',
          field_key: 'weather',
        },
        status: 'pending' as const,
        summary: 'Set harbor weather to sunny',
        proposed_value: { type: 'text' as const, value: 'sunny' },
        source_dependency_id: 'dependency.weather.scene',
        rationale: 'Older proposal',
        created_at_ms: 56_789,
      },
    ],
  },
};

beforeEach(() => {
  clearPropagationProposalListProjection();
  acceptPropagationProposalMock.mockReset();
  createPropagationProposalMock.mockReset();
  rejectPropagationProposalMock.mockReset();
  updatePropagationProposalMock.mockReset();
  getPropagationProposalListProjectionMock.mockReset();
});

describe('propagation proposal projection store', () => {
  it('stores backend propagation proposal projections', async () => {
    getPropagationProposalListProjectionMock.mockResolvedValue(projection);

    await expect(refreshPropagationProposalListProjection()).resolves.toEqual(projection);

    expect(getPropagationProposalListProjectionMock).toHaveBeenCalledWith();
    expect(getCachedPropagationProposalListProjection()).toEqual(projection);
    expect(propagationProposalProjectionState.pending).toBe(false);
    expect(propagationProposalProjectionState.error).toBeUndefined();
  });

  it('stores create, reject, update, and accept command response projections', async () => {
    createPropagationProposalMock.mockResolvedValue({
      outcome: 'recorded',
      projection,
    });
    rejectPropagationProposalMock.mockResolvedValue({
      outcome: 'recorded',
      projection,
    });
    updatePropagationProposalMock.mockResolvedValue({
      outcome: 'recorded',
      projection,
    });
    acceptPropagationProposalMock.mockResolvedValue({
      outcome: 'recorded',
      projection,
    });

    await applyCreatePropagationProposalCommand(
      {
        proposal_id: 'proposal.propagation.weather',
        action: 'set_bible_field',
        target: {
          kind: 'bible_field',
          node_id: 'node.location.harbor',
          part_key: 'environment',
          field_key: 'weather',
        },
        summary: 'Set harbor weather to rainy',
        proposed_value: { type: 'text', value: 'rainy' },
        source_dependency_id: 'dependency.weather.scene',
        rationale: 'Manual edit introduced rainy weather',
      },
      'command-create',
    );
    await applyRejectPropagationProposalCommand(
      {
        proposal_id: 'proposal.propagation.weather',
        reason: 'Wrong scope',
      },
      'command-reject',
    );
    await applyUpdatePropagationProposalCommand(
      {
        proposal_id: 'proposal.propagation.weather',
        action: 'set_bible_field',
        target: {
          kind: 'bible_field',
          node_id: 'node.location.harbor',
          part_key: 'environment',
          field_key: 'weather',
        },
        summary: 'Set harbor weather to foggy',
        proposed_value: { type: 'text', value: 'foggy' },
        source_dependency_id: 'dependency.weather.scene',
        rationale: 'Reviewer corrected propagation',
      },
      'command-update',
    );
    await applyAcceptPropagationProposalCommand(
      {
        proposal_id: 'proposal.propagation.weather',
      },
      'command-accept',
    );

    expect(createPropagationProposalMock).toHaveBeenCalledWith(
      {
        proposal_id: 'proposal.propagation.weather',
        action: 'set_bible_field',
        target: {
          kind: 'bible_field',
          node_id: 'node.location.harbor',
          part_key: 'environment',
          field_key: 'weather',
        },
        summary: 'Set harbor weather to rainy',
        proposed_value: { type: 'text', value: 'rainy' },
        source_dependency_id: 'dependency.weather.scene',
        rationale: 'Manual edit introduced rainy weather',
      },
      'command-create',
    );
    expect(rejectPropagationProposalMock).toHaveBeenCalledWith(
      {
        proposal_id: 'proposal.propagation.weather',
        reason: 'Wrong scope',
      },
      'command-reject',
    );
    expect(updatePropagationProposalMock).toHaveBeenCalledWith(
      {
        proposal_id: 'proposal.propagation.weather',
        action: 'set_bible_field',
        target: {
          kind: 'bible_field',
          node_id: 'node.location.harbor',
          part_key: 'environment',
          field_key: 'weather',
        },
        summary: 'Set harbor weather to foggy',
        proposed_value: { type: 'text', value: 'foggy' },
        source_dependency_id: 'dependency.weather.scene',
        rationale: 'Reviewer corrected propagation',
      },
      'command-update',
    );
    expect(acceptPropagationProposalMock).toHaveBeenCalledWith(
      {
        proposal_id: 'proposal.propagation.weather',
      },
      'command-accept',
    );
    expect(getCachedPropagationProposalListProjection()).toEqual(projection);
  });

  it('records errors without replacing cached projections', async () => {
    getPropagationProposalListProjectionMock.mockResolvedValue(projection);
    await refreshPropagationProposalListProjection();
    getPropagationProposalListProjectionMock.mockRejectedValue(
      new Error('propagation proposals unavailable'),
    );

    await expect(refreshPropagationProposalListProjection()).rejects.toThrow(
      'propagation proposals unavailable',
    );

    expect(getCachedPropagationProposalListProjection()).toEqual(projection);
    expect(propagationProposalProjectionState.pending).toBe(false);
    expect(propagationProposalProjectionState.error).toBe('propagation proposals unavailable');
  });

  it('does not replace cached propagation projections with stale refresh results', async () => {
    getPropagationProposalListProjectionMock.mockResolvedValueOnce(newerProjection);
    await refreshPropagationProposalListProjection();
    getPropagationProposalListProjectionMock.mockResolvedValueOnce(olderProjection);

    await expect(refreshPropagationProposalListProjection()).resolves.toEqual(olderProjection);

    expect(getCachedPropagationProposalListProjection()).toEqual(newerProjection);
    expect(propagationProposalProjectionState.pending).toBe(false);
    expect(propagationProposalProjectionState.error).toBeUndefined();
  });

  it('does not replace cached propagation projections with stale command responses', async () => {
    getPropagationProposalListProjectionMock.mockResolvedValueOnce(newerProjection);
    await refreshPropagationProposalListProjection();
    createPropagationProposalMock.mockResolvedValue({
      outcome: 'recorded',
      projection: olderProjection,
    });

    await expect(
      applyCreatePropagationProposalCommand(
        {
          proposal_id: 'proposal.propagation.sun',
          action: 'set_bible_field',
          target: {
            kind: 'bible_field',
            node_id: 'node.location.harbor',
            part_key: 'environment',
            field_key: 'weather',
          },
          summary: 'Set harbor weather to sunny',
          proposed_value: { type: 'text', value: 'sunny' },
          source_dependency_id: 'dependency.weather.scene',
          rationale: 'Older command response',
        },
        'command-create-stale',
      ),
    ).resolves.toEqual({
      outcome: 'recorded',
      projection: olderProjection,
    });

    expect(getCachedPropagationProposalListProjection()).toEqual(newerProjection);
    expect(propagationProposalProjectionState.pending).toBe(false);
    expect(propagationProposalProjectionState.error).toBeUndefined();
  });
});
