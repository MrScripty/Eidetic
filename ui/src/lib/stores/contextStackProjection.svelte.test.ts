import { beforeEach, describe, expect, it, vi } from 'vitest';

import { getContextStackProjection } from '$lib/projectionApi.js';
import type { ContextStackProjection } from '$lib/contextInfluenceTypes.js';
import type { ProjectionEnvelope } from '$lib/projectionTypes.js';
import {
  clearContextStackProjection,
  contextStackProjectionState,
  getCachedContextStackProjection,
  refreshContextStackProjection,
} from './contextStackProjection.svelte.js';

vi.mock('$lib/projectionApi.js', () => ({
  getContextStackProjection: vi.fn(),
}));

const getContextStackProjectionMock = vi.mocked(getContextStackProjection);

function projection(version: number): ProjectionEnvelope<ContextStackProjection> {
  return {
    version,
    change_event_id: `event-${version}`,
    payload: {
      target_node_id: 'node.scene.beach',
      layers: [
        {
          node_id: 'node.act.one',
          level: 'Act',
          label: 'Act One',
          role: 'inherited',
          distilled_context: 'Opening pressure.',
          sort_order: 0,
        },
      ],
    },
  };
}

beforeEach(() => {
  clearContextStackProjection();
  getContextStackProjectionMock.mockReset();
});

describe('context stack projection store', () => {
  it('loads context stack projections through the desktop projection API', async () => {
    const response = projection(1);
    getContextStackProjectionMock.mockResolvedValue(response);

    await expect(refreshContextStackProjection('node.scene.beach')).resolves.toEqual(response);

    expect(getContextStackProjectionMock).toHaveBeenCalledWith({
      target_node_id: 'node.scene.beach',
    });
    expect(contextStackProjectionState.targetNodeId).toBe('node.scene.beach');
    expect(getCachedContextStackProjection()).toEqual(response);
  });

  it('keeps the latest cached projection when an older response arrives later', async () => {
    const newer = projection(2);
    const older = projection(1);
    getContextStackProjectionMock.mockResolvedValueOnce(newer);
    await refreshContextStackProjection('node.scene.beach');
    getContextStackProjectionMock.mockResolvedValueOnce(older);

    await expect(refreshContextStackProjection('node.scene.beach')).resolves.toEqual(older);

    expect(getCachedContextStackProjection()).toEqual(newer);
  });

  it('clears projection cache and pending state', () => {
    contextStackProjectionState.projection = projection(1);
    contextStackProjectionState.targetNodeId = 'node.scene.beach';
    contextStackProjectionState.pending = true;
    contextStackProjectionState.error = 'failed';

    clearContextStackProjection();

    expect(contextStackProjectionState).toMatchObject({
      targetNodeId: null,
      projection: null,
      pending: false,
      error: undefined,
    });
  });
});
