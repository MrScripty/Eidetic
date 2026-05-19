import { afterEach, describe, expect, it, vi } from 'vitest';

import { getChangeReviewProjection } from './projectionApi.js';

afterEach(() => {
  vi.unstubAllGlobals();
  vi.restoreAllMocks();
});

describe('change review projection api helpers', () => {
  it('fetches change review projections without query params', async () => {
    const response = {
      version: 2,
      change_event_id: 'event-1',
      payload: {
        changes: [
          {
            event: {
              id: 'event-1',
              command_id: 'command-1',
              kind: 'ai_proposal_accepted',
              summary: 'accept bible reference Ada',
              created_at_ms: 100,
            },
            revisions: [
              {
                id: 'revision-1',
                object_kind: 'semantic_proposal',
                object_id: 'proposal.ada',
                change_event_id: 'event-1',
                base_revision_id: null,
                operation: 'update',
                fields: [
                  {
                    field_key: 'status',
                    old_value: { type: 'text', value: 'pending' },
                    new_value: { type: 'text', value: 'accepted' },
                    sort_order: 0,
                  },
                ],
              },
            ],
          },
        ],
      },
    };
    const fetchMock = vi.fn().mockResolvedValue(
      new Response(JSON.stringify(response), {
        status: 200,
        headers: { 'Content-Type': 'application/json' },
      }),
    );
    vi.stubGlobal('fetch', fetchMock);

    await expect(getChangeReviewProjection()).resolves.toEqual(response);

    expect(fetchMock).toHaveBeenCalledWith('/api/projections/history/changes', {
      method: 'GET',
      headers: { Accept: 'application/json' },
    });
  });
});
