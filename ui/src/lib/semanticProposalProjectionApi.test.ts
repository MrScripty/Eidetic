import { afterEach, describe, expect, it, vi } from 'vitest';

import { getBibleReferenceProposalListProjection } from './projectionApi.js';

afterEach(() => {
  vi.unstubAllGlobals();
  vi.restoreAllMocks();
});

describe('semantic proposal projection api helpers', () => {
  it('fetches bible reference proposal projections without query params', async () => {
    const response = {
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
    };
    const fetchMock = vi.fn().mockResolvedValue(
      new Response(JSON.stringify(response), {
        status: 200,
        headers: { 'Content-Type': 'application/json' },
      }),
    );
    vi.stubGlobal('fetch', fetchMock);

    await expect(getBibleReferenceProposalListProjection()).resolves.toEqual(response);

    expect(fetchMock).toHaveBeenCalledWith('/api/projections/semantic/bible-reference-proposals', {
      method: 'GET',
      headers: { Accept: 'application/json' },
    });
  });

  it('uses desktop bible reference proposal projection command when Tauri transport is available', async () => {
    const response = {
      version: 4,
      change_event_id: 'event-4',
      payload: {
        proposals: [],
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

    await expect(getBibleReferenceProposalListProjection()).resolves.toEqual(response);

    expect(invoke).toHaveBeenCalledWith('projection_bible_reference_proposals', undefined);
    expect(fetchMock).not.toHaveBeenCalled();
  });
});
