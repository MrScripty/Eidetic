import { beforeEach, describe, expect, it, vi } from 'vitest';

import { getBibleGraphSchemaListProjection } from '$lib/projectionApi.js';
import type { BibleGraphSchemaListProjection } from '$lib/bibleGraphSchemaTypes.js';
import type { ProjectionEnvelope } from '$lib/projectionTypes.js';
import {
  bibleGraphSchemaProjectionState,
  clearBibleGraphSchemaListProjection,
  getCachedBibleGraphSchemaListProjection,
  refreshBibleGraphSchemaListProjection,
} from './bibleGraphSchemaProjection.svelte.js';

vi.mock('$lib/projectionApi.js', () => ({
  getBibleGraphSchemaListProjection: vi.fn(),
}));

const getBibleGraphSchemaListProjectionMock = vi.mocked(getBibleGraphSchemaListProjection);

const projection: ProjectionEnvelope<BibleGraphSchemaListProjection> = {
  version: 1,
  payload: {
    schemas: [
      {
        schema_key: 'character',
        category: 'character',
        display_name: 'Character',
        default_node_name: 'New Character',
        canonical_parent_id: 'canonical.characters',
        canonical_root_schema_key: 'canonical.root.characters',
        parts: [
          {
            part_key: 'profile',
            name: 'Profile',
            sort_order: 10,
            fields: [
              {
                field_key: 'summary',
                sort_order: 10,
              },
              {
                field_key: 'tagline',
                sort_order: 20,
              },
            ],
          },
        ],
      },
    ],
  },
};

const newerProjection: ProjectionEnvelope<BibleGraphSchemaListProjection> = {
  ...projection,
  version: 3,
  payload: {
    schemas: [
      {
        schema_key: 'location',
        category: 'location',
        display_name: 'Location',
        default_node_name: 'New Location',
        canonical_parent_id: 'canonical.places',
        canonical_root_schema_key: 'canonical.root.places',
        parts: [
          {
            part_key: 'environment',
            name: 'Environment',
            sort_order: 10,
            fields: [
              {
                field_key: 'weather',
                sort_order: 10,
              },
            ],
          },
        ],
      },
    ],
  },
};

const olderProjection: ProjectionEnvelope<BibleGraphSchemaListProjection> = {
  ...projection,
  version: 2,
  payload: {
    schemas: [
      {
        schema_key: 'prop',
        category: 'prop',
        display_name: 'Prop',
        default_node_name: 'New Prop',
        canonical_parent_id: 'canonical.objects',
        canonical_root_schema_key: 'canonical.root.objects',
        parts: [],
      },
    ],
  },
};

function resetProjectionState(): void {
  bibleGraphSchemaProjectionState.projection = null;
  bibleGraphSchemaProjectionState.pending = false;
  bibleGraphSchemaProjectionState.error = undefined;
}

beforeEach(() => {
  resetProjectionState();
  getBibleGraphSchemaListProjectionMock.mockReset();
});

describe('bible graph schema projection store', () => {
  it('stores backend schema projection reads and clears pending state', async () => {
    getBibleGraphSchemaListProjectionMock.mockResolvedValue(projection);

    await expect(refreshBibleGraphSchemaListProjection()).resolves.toEqual(projection);

    expect(getBibleGraphSchemaListProjectionMock).toHaveBeenCalledWith();
    expect(getCachedBibleGraphSchemaListProjection()).toEqual(projection);
    expect(bibleGraphSchemaProjectionState.pending).toBe(false);
    expect(bibleGraphSchemaProjectionState.error).toBeUndefined();
  });

  it('records read errors without replacing a cached projection', async () => {
    getBibleGraphSchemaListProjectionMock.mockResolvedValue(projection);
    await refreshBibleGraphSchemaListProjection();
    getBibleGraphSchemaListProjectionMock.mockRejectedValue(new Error('schemas unavailable'));

    await expect(refreshBibleGraphSchemaListProjection()).rejects.toThrow('schemas unavailable');

    expect(getCachedBibleGraphSchemaListProjection()).toEqual(projection);
    expect(bibleGraphSchemaProjectionState.pending).toBe(false);
    expect(bibleGraphSchemaProjectionState.error).toBe('schemas unavailable');
  });

  it('does not replace cached schema projections with stale refresh results', async () => {
    getBibleGraphSchemaListProjectionMock.mockResolvedValueOnce(newerProjection);
    await refreshBibleGraphSchemaListProjection();
    getBibleGraphSchemaListProjectionMock.mockResolvedValueOnce(olderProjection);

    await expect(refreshBibleGraphSchemaListProjection()).resolves.toEqual(olderProjection);

    expect(getCachedBibleGraphSchemaListProjection()).toEqual(newerProjection);
    expect(bibleGraphSchemaProjectionState.pending).toBe(false);
    expect(bibleGraphSchemaProjectionState.error).toBeUndefined();
  });

  it('clears cached schema projection state', async () => {
    getBibleGraphSchemaListProjectionMock.mockResolvedValue(projection);
    await refreshBibleGraphSchemaListProjection();

    clearBibleGraphSchemaListProjection();

    expect(getCachedBibleGraphSchemaListProjection()).toBeNull();
    expect(bibleGraphSchemaProjectionState.pending).toBe(false);
    expect(bibleGraphSchemaProjectionState.error).toBeUndefined();
  });
});
