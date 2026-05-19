import { afterEach, describe, expect, it, vi } from 'vitest';

import {
  acceptBibleReferenceProposal,
  createBibleReferenceProposal,
  rejectBibleReferenceProposal,
} from './commandApi.js';

afterEach(() => {
  vi.unstubAllGlobals();
  vi.restoreAllMocks();
});

describe('semantic proposal command api helpers', () => {
  it('sends bible reference proposal commands and returns proposal projections', async () => {
    const response = {
      outcome: 'recorded',
      projection: {
        version: 4,
        change_event_id: 'event-4',
        payload: {
          proposals: [
            {
              id: 'proposal-ada',
              source_node_id: 'node.scene.one',
              child_name: 'First encounter',
              reference_kind: 'character',
              reference_text: 'Ada',
              proposed_schema_key: 'character',
              status: 'pending',
              rationale: 'Named in the generated child plan',
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
      createBibleReferenceProposal(
        {
          proposal_id: 'proposal-ada',
          source_node_id: 'node.scene.one',
          child_name: 'First encounter',
          reference_kind: 'character',
          reference_text: 'Ada',
          rationale: 'Named in the generated child plan',
        },
        'command-proposal-create',
      ),
    ).resolves.toEqual(response);

    await expect(
      rejectBibleReferenceProposal(
        {
          proposal_id: 'proposal-ada',
          reason: 'Duplicate',
        },
        'command-proposal-reject',
      ),
    ).resolves.toEqual(response);

    await expect(
      acceptBibleReferenceProposal(
        {
          proposal_id: 'proposal-ada',
          node_id: 'bible.character.ada',
          parent_id: 'bible.root.characters',
          name: 'Ada',
          sort_order: 20,
        },
        'command-proposal-accept',
      ),
    ).resolves.toEqual(response);

    expect(fetchMock).toHaveBeenNthCalledWith(
      1,
      '/api/commands/semantic/bible-reference-proposal',
      expect.objectContaining({
        method: 'POST',
        headers: { 'Content-Type': 'application/json' },
        body: JSON.stringify({
          id: 'command-proposal-create',
          payload: {
            proposal_id: 'proposal-ada',
            source_node_id: 'node.scene.one',
            child_name: 'First encounter',
            reference_kind: 'character',
            reference_text: 'Ada',
            rationale: 'Named in the generated child plan',
          },
        }),
      }),
    );
    expect(fetchMock).toHaveBeenNthCalledWith(
      2,
      '/api/commands/semantic/bible-reference-proposal/reject',
      expect.objectContaining({
        method: 'POST',
        headers: { 'Content-Type': 'application/json' },
        body: JSON.stringify({
          id: 'command-proposal-reject',
          payload: {
            proposal_id: 'proposal-ada',
            reason: 'Duplicate',
          },
        }),
      }),
    );
    expect(fetchMock).toHaveBeenNthCalledWith(
      3,
      '/api/commands/semantic/bible-reference-proposal/accept',
      expect.objectContaining({
        method: 'POST',
        headers: { 'Content-Type': 'application/json' },
        body: JSON.stringify({
          id: 'command-proposal-accept',
          payload: {
            proposal_id: 'proposal-ada',
            node_id: 'bible.character.ada',
            parent_id: 'bible.root.characters',
            name: 'Ada',
            sort_order: 20,
          },
        }),
      }),
    );
  });
});
