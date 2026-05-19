import { afterEach, describe, expect, it, vi } from 'vitest';

import {
  acceptPropagationProposal,
  createPropagationProposal,
  rejectPropagationProposal,
} from './commandApi.js';

afterEach(() => {
  vi.unstubAllGlobals();
  vi.restoreAllMocks();
});

describe('propagation proposal command api helpers', () => {
  it('sends propagation proposal commands and returns proposal projections', async () => {
    const response = {
      outcome: 'recorded',
      projection: {
        version: 4,
        change_event_id: 'event-4',
        payload: {
          proposals: [
            {
              id: 'proposal.propagation.weather',
              action: 'set_bible_field',
              target: {
                kind: 'bible_field',
                node_id: 'node.location.harbor',
                part_key: 'environment',
                field_key: 'weather',
              },
              status: 'pending',
              summary: 'Set harbor weather to rainy',
              proposed_value: { type: 'text', value: 'rainy' },
              source_dependency_id: 'dependency.weather.scene',
              rationale: 'Manual edit introduced rainy weather',
              created_at_ms: 12_345,
            },
          ],
        },
      },
    };
    const fetchMock = vi.fn().mockImplementation(() =>
      Promise.resolve(
        new Response(JSON.stringify(response), {
          status: 200,
          headers: { 'Content-Type': 'application/json' },
        }),
      ),
    );
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
      acceptPropagationProposal(
        {
          proposal_id: 'proposal.propagation.weather',
        },
        'command-propagation-accept',
      ),
    ).resolves.toEqual(response);

    expect(fetchMock).toHaveBeenNthCalledWith(
      1,
      '/api/commands/semantic/propagation-proposal',
      expect.objectContaining({
        method: 'POST',
        headers: { 'Content-Type': 'application/json' },
        body: JSON.stringify({
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
        }),
      }),
    );
    expect(fetchMock).toHaveBeenNthCalledWith(
      2,
      '/api/commands/semantic/propagation-proposal/reject',
      expect.objectContaining({
        method: 'POST',
        headers: { 'Content-Type': 'application/json' },
        body: JSON.stringify({
          id: 'command-propagation-reject',
          payload: {
            proposal_id: 'proposal.propagation.weather',
            reason: 'Wrong scope',
          },
        }),
      }),
    );
    expect(fetchMock).toHaveBeenNthCalledWith(
      3,
      '/api/commands/semantic/propagation-proposal/accept',
      expect.objectContaining({
        method: 'POST',
        headers: { 'Content-Type': 'application/json' },
        body: JSON.stringify({
          id: 'command-propagation-accept',
          payload: {
            proposal_id: 'proposal.propagation.weather',
          },
        }),
      }),
    );
  });
});
