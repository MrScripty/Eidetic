import { beforeEach, describe, expect, it, vi } from 'vitest';

import { setObjectField } from '$lib/commandApi.js';
import { getObjectFieldProjection } from '$lib/projectionApi.js';
import {
  applyObjectFieldCommand,
  clearObjectFieldProjection,
  getCachedObjectFieldProjection,
  getObjectFieldProjectionError,
  isObjectFieldProjectionPending,
  objectFieldProjectionState,
  refreshObjectFieldProjection,
} from './objectFieldProjection.svelte.js';

vi.mock('$lib/commandApi.js', () => ({
  setObjectField: vi.fn(),
}));

vi.mock('$lib/projectionApi.js', () => ({
  getObjectFieldProjection: vi.fn(),
}));

const setObjectFieldMock = vi.mocked(setObjectField);
const getObjectFieldProjectionMock = vi.mocked(getObjectFieldProjection);

const key = {
  object_kind: 'bible_part_field' as const,
  object_id: 'field/weather one',
};

const projection = {
  version: 2,
  change_event_id: 'event-1',
  payload: {
    object_kind: 'bible_part_field' as const,
    object_id: 'field/weather one',
    deleted: false,
    fields: {
      weather: { type: 'text' as const, value: 'rainy' },
    },
  },
};

const newerProjection = {
  ...projection,
  version: 4,
  change_event_id: 'event-4',
  payload: {
    ...projection.payload,
    fields: {
      weather: { type: 'text' as const, value: 'foggy' },
    },
  },
};

const olderProjection = {
  ...projection,
  version: 3,
  change_event_id: 'event-3',
  payload: {
    ...projection.payload,
    fields: {
      weather: { type: 'text' as const, value: 'sunny' },
    },
  },
};

function resetProjectionState(): void {
  for (const cacheKey of Object.keys(objectFieldProjectionState.projections)) {
    delete objectFieldProjectionState.projections[cacheKey];
  }
  for (const cacheKey of Object.keys(objectFieldProjectionState.pending)) {
    delete objectFieldProjectionState.pending[cacheKey];
  }
  for (const cacheKey of Object.keys(objectFieldProjectionState.errors)) {
    delete objectFieldProjectionState.errors[cacheKey];
  }
}

beforeEach(() => {
  resetProjectionState();
  setObjectFieldMock.mockReset();
  getObjectFieldProjectionMock.mockReset();
});

describe('object field projection store', () => {
  it('stores backend projection reads and clears pending state', async () => {
    getObjectFieldProjectionMock.mockResolvedValue(projection);

    await expect(refreshObjectFieldProjection(key)).resolves.toEqual(projection);

    expect(getObjectFieldProjectionMock).toHaveBeenCalledWith(key);
    expect(getCachedObjectFieldProjection(key)).toEqual(projection);
    expect(isObjectFieldProjectionPending(key)).toBe(false);
    expect(getObjectFieldProjectionError(key)).toBeUndefined();
  });

  it('records read errors without caching a projection', async () => {
    getObjectFieldProjectionMock.mockRejectedValue(new Error('projection unavailable'));

    await expect(refreshObjectFieldProjection(key)).rejects.toThrow('projection unavailable');

    expect(getCachedObjectFieldProjection(key)).toBeUndefined();
    expect(isObjectFieldProjectionPending(key)).toBe(false);
    expect(getObjectFieldProjectionError(key)).toBe('projection unavailable');
  });

  it('stores command response projections', async () => {
    setObjectFieldMock.mockResolvedValue({
      outcome: 'recorded',
      projection,
    });

    await expect(
      applyObjectFieldCommand(
        {
          object_kind: 'bible_part_field',
          object_id: 'field/weather one',
          field_key: 'weather',
          value: { type: 'text', value: 'rainy' },
        },
        'command-1',
      ),
    ).resolves.toEqual({
      outcome: 'recorded',
      projection,
    });

    expect(setObjectFieldMock).toHaveBeenCalledWith(
      {
        object_kind: 'bible_part_field',
        object_id: 'field/weather one',
        field_key: 'weather',
        value: { type: 'text', value: 'rainy' },
      },
      'command-1',
    );
    expect(getObjectFieldProjectionMock).not.toHaveBeenCalled();
    expect(getCachedObjectFieldProjection(key)).toEqual(projection);
  });

  it('records command errors and leaves cached projections unchanged', async () => {
    getObjectFieldProjectionMock.mockResolvedValue(projection);
    await refreshObjectFieldProjection(key);
    setObjectFieldMock.mockRejectedValue(new Error('command conflict'));

    await expect(
      applyObjectFieldCommand({
        object_kind: 'bible_part_field',
        object_id: 'field/weather one',
        field_key: 'weather',
        value: { type: 'text', value: 'stormy' },
      }),
    ).rejects.toThrow('command conflict');

    expect(getCachedObjectFieldProjection(key)).toEqual(projection);
    expect(isObjectFieldProjectionPending(key)).toBe(false);
    expect(getObjectFieldProjectionError(key)).toBe('command conflict');
  });

  it('does not replace cached object projections with stale refresh results', async () => {
    getObjectFieldProjectionMock.mockResolvedValueOnce(newerProjection);
    await refreshObjectFieldProjection(key);
    getObjectFieldProjectionMock.mockResolvedValueOnce(olderProjection);

    await expect(refreshObjectFieldProjection(key)).resolves.toEqual(olderProjection);

    expect(getCachedObjectFieldProjection(key)).toEqual(newerProjection);
    expect(isObjectFieldProjectionPending(key)).toBe(false);
    expect(getObjectFieldProjectionError(key)).toBeUndefined();
  });

  it('does not replace cached object projections with stale command responses', async () => {
    getObjectFieldProjectionMock.mockResolvedValueOnce(newerProjection);
    await refreshObjectFieldProjection(key);
    setObjectFieldMock.mockResolvedValue({
      outcome: 'recorded',
      projection: olderProjection,
    });

    await expect(
      applyObjectFieldCommand({
        object_kind: 'bible_part_field',
        object_id: 'field/weather one',
        field_key: 'weather',
        value: { type: 'text', value: 'sunny' },
      }),
    ).resolves.toEqual({
      outcome: 'recorded',
      projection: olderProjection,
    });

    expect(getCachedObjectFieldProjection(key)).toEqual(newerProjection);
    expect(isObjectFieldProjectionPending(key)).toBe(false);
    expect(getObjectFieldProjectionError(key)).toBeUndefined();
  });

  it('clears cached projection state for one object', async () => {
    getObjectFieldProjectionMock.mockResolvedValue(projection);
    await refreshObjectFieldProjection(key);

    clearObjectFieldProjection(key);

    expect(getCachedObjectFieldProjection(key)).toBeUndefined();
    expect(isObjectFieldProjectionPending(key)).toBe(false);
    expect(getObjectFieldProjectionError(key)).toBeUndefined();
  });
});
