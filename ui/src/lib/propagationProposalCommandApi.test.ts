import { afterEach, describe, expect, it, vi } from 'vitest';

import {
  acceptPropagationProposal,
  createPropagationProposal,
  rejectPropagationProposal,
  updatePropagationProposal,
} from './commandApi.js';

afterEach(() => {
  vi.unstubAllGlobals();
  vi.restoreAllMocks();
});

describe('propagation proposal command api helpers', () => {
  it('uses desktop propagation proposal commands when Tauri transport is available', async () => {
    const response = {
      outcome: 'recorded',
      projection: {
        version: 4,
        payload: {
          proposals: [],
        },
      },
    };
    const invoke = vi.fn().mockResolvedValue(response);
    vi.stubGlobal('window', {
      __TAURI__: {
        core: { invoke },
      },
    });
    const fetchMock = vi.fn();
    vi.stubGlobal('fetch', fetchMock);

    await expect(
      createPropagationProposal(
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
        'command-propagation-create',
      ),
    ).resolves.toEqual(response);

    await expect(
      rejectPropagationProposal(
        {
          proposal_id: 'proposal.propagation.weather',
          reason: 'Wrong scope',
        },
        'command-propagation-reject',
      ),
    ).resolves.toEqual(response);

    await expect(
      updatePropagationProposal(
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
        'command-propagation-update',
      ),
    ).resolves.toEqual(response);

    await expect(
      acceptPropagationProposal(
        {
          proposal_id: 'proposal.propagation.weather',
        },
        'command-propagation-accept',
      ),
    ).resolves.toEqual(response);

    expect(invoke).toHaveBeenNthCalledWith(1, 'command_propagation_proposal_create', {
      command: {
        id: 'command-propagation-create',
        payload: {
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
      },
    });
    expect(invoke).toHaveBeenNthCalledWith(2, 'command_propagation_proposal_reject', {
      command: {
        id: 'command-propagation-reject',
        payload: {
          proposal_id: 'proposal.propagation.weather',
          reason: 'Wrong scope',
        },
      },
    });
    expect(invoke).toHaveBeenNthCalledWith(3, 'command_propagation_proposal_update', {
      command: {
        id: 'command-propagation-update',
        payload: {
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
      },
    });
    expect(invoke).toHaveBeenNthCalledWith(4, 'command_propagation_proposal_accept', {
      command: {
        id: 'command-propagation-accept',
        payload: {
          proposal_id: 'proposal.propagation.weather',
        },
      },
    });
    expect(fetchMock).not.toHaveBeenCalled();
  });
});
