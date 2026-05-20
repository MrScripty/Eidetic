import { afterEach, describe, expect, it, vi } from 'vitest';

import { getPropagationProposalListProjection } from './projectionApi.js';

afterEach(() => {
  vi.unstubAllGlobals();
  vi.restoreAllMocks();
});

describe('propagation proposal projection api helpers', () => {
  it('fetches propagation proposal projections without query params', async () => {
    const response = {
      version: 4,
      change_event_id: 'event-4',
      payload: {
        proposals: [
          {
            id: 'proposal.propagation.script-block',
            action: 'patch_script_block',
            target: {
              kind: 'script_block',
              block_id: 'script.block.action-1',
            },
            status: 'pending',
            summary: 'Patch generated script block',
            proposed_text: 'Ada enters with a rain-black umbrella.',
            source_dependency_id: 'dependency.weather.scene',
            rationale: 'Manual script edit requires propagation',
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

    await expect(getPropagationProposalListProjection()).resolves.toEqual(response);

    expect(fetchMock).toHaveBeenCalledWith('/api/projections/semantic/propagation-proposals', {
      method: 'GET',
      headers: { Accept: 'application/json' },
    });
  });

  it('uses desktop propagation proposal projection command when Tauri transport is available', async () => {
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

    await expect(getPropagationProposalListProjection()).resolves.toEqual(response);

    expect(invoke).toHaveBeenCalledWith('projection_propagation_proposals', undefined);
    expect(fetchMock).not.toHaveBeenCalled();
  });
});
