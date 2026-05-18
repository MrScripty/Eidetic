import { beforeEach, describe, expect, it, vi } from 'vitest';

import { setScriptBlock } from '$lib/commandApi.js';
import { getScriptDocumentProjection } from '$lib/projectionApi.js';
import {
  applyScriptBlockCommand,
  clearScriptDocumentProjection,
  getCachedScriptDocumentProjection,
  getScriptDocumentProjectionError,
  isScriptDocumentProjectionPending,
  refreshScriptDocumentProjection,
  scriptDocumentProjectionState,
} from './scriptDocumentProjection.svelte.js';

vi.mock('$lib/commandApi.js', () => ({
  setScriptBlock: vi.fn(),
}));

vi.mock('$lib/projectionApi.js', () => ({
  getScriptDocumentProjection: vi.fn(),
}));

const setScriptBlockMock = vi.mocked(setScriptBlock);
const getScriptDocumentProjectionMock = vi.mocked(getScriptDocumentProjection);

const key = {
  document_id: 'script.document/main one',
};

const projection = {
  version: 5,
  change_event_id: 'event-script-1',
  payload: {
    document: {
      id: 'script.document/main one',
      title: 'Pilot',
      sort_order: 0,
    },
    segments: [
      {
        segment: {
          id: 'script.segment.beat-1',
          document_id: 'script.document/main one',
          source_node_id: 'node.beat.opening',
          start_ms: 1000,
          end_ms: 5000,
          status: 'current' as const,
          sort_order: 1,
        },
        blocks: [
          {
            block: {
              id: 'script.block.action-1',
              segment_id: 'script.segment.beat-1',
              block_kind: 'action' as const,
              text: 'Ada enters with a wet umbrella.',
              sort_order: 2,
            },
            spans: [],
            locks: [],
          },
        ],
      },
    ],
  },
};

function resetProjectionState(): void {
  for (const cacheKey of Object.keys(scriptDocumentProjectionState.projections)) {
    delete scriptDocumentProjectionState.projections[cacheKey];
  }
  for (const cacheKey of Object.keys(scriptDocumentProjectionState.pending)) {
    delete scriptDocumentProjectionState.pending[cacheKey];
  }
  for (const cacheKey of Object.keys(scriptDocumentProjectionState.errors)) {
    delete scriptDocumentProjectionState.errors[cacheKey];
  }
}

beforeEach(() => {
  resetProjectionState();
  setScriptBlockMock.mockReset();
  getScriptDocumentProjectionMock.mockReset();
});

describe('script document projection store', () => {
  it('stores backend projection reads and clears pending state', async () => {
    getScriptDocumentProjectionMock.mockResolvedValue(projection);

    await expect(refreshScriptDocumentProjection(key)).resolves.toEqual(projection);

    expect(getScriptDocumentProjectionMock).toHaveBeenCalledWith(key);
    expect(getCachedScriptDocumentProjection(key)).toEqual(projection);
    expect(isScriptDocumentProjectionPending(key)).toBe(false);
    expect(getScriptDocumentProjectionError(key)).toBeUndefined();
  });

  it('records read errors without caching a projection', async () => {
    getScriptDocumentProjectionMock.mockRejectedValue(new Error('script document not found'));

    await expect(refreshScriptDocumentProjection(key)).rejects.toThrow('script document not found');

    expect(getCachedScriptDocumentProjection(key)).toBeUndefined();
    expect(isScriptDocumentProjectionPending(key)).toBe(false);
    expect(getScriptDocumentProjectionError(key)).toBe('script document not found');
  });

  it('stores command response projections without reading legacy script state', async () => {
    setScriptBlockMock.mockResolvedValue({
      outcome: 'recorded',
      projection,
    });

    await expect(
      applyScriptBlockCommand(
        {
          document_id: 'script.document/main one',
          document_title: 'Pilot',
          document_sort_order: 0,
          segment_id: 'script.segment.beat-1',
          source_node_id: 'node.beat.opening',
          segment_start_ms: 1000,
          segment_end_ms: 5000,
          segment_status: 'current',
          segment_sort_order: 1,
          block_id: 'script.block.action-1',
          block_kind: 'action',
          text: 'Ada enters with a wet umbrella.',
          sort_order: 2,
        },
        'command-script-1',
      ),
    ).resolves.toEqual({
      outcome: 'recorded',
      projection,
    });

    expect(setScriptBlockMock).toHaveBeenCalledWith(
      {
        document_id: 'script.document/main one',
        document_title: 'Pilot',
        document_sort_order: 0,
        segment_id: 'script.segment.beat-1',
        source_node_id: 'node.beat.opening',
        segment_start_ms: 1000,
        segment_end_ms: 5000,
        segment_status: 'current',
        segment_sort_order: 1,
        block_id: 'script.block.action-1',
        block_kind: 'action',
        text: 'Ada enters with a wet umbrella.',
        sort_order: 2,
      },
      'command-script-1',
    );
    expect(getScriptDocumentProjectionMock).not.toHaveBeenCalled();
    expect(getCachedScriptDocumentProjection(key)).toEqual(projection);
  });

  it('records command errors and leaves cached projections unchanged', async () => {
    getScriptDocumentProjectionMock.mockResolvedValue(projection);
    await refreshScriptDocumentProjection(key);
    setScriptBlockMock.mockRejectedValue(new Error('command conflict'));

    await expect(
      applyScriptBlockCommand({
        document_id: 'script.document/main one',
        document_title: 'Pilot',
        segment_id: 'script.segment.beat-1',
        segment_start_ms: 1000,
        segment_end_ms: 5000,
        segment_status: 'current',
        block_id: 'script.block.action-1',
        block_kind: 'action',
        text: 'Ada exits.',
      }),
    ).rejects.toThrow('command conflict');

    expect(getCachedScriptDocumentProjection(key)).toEqual(projection);
    expect(isScriptDocumentProjectionPending(key)).toBe(false);
    expect(getScriptDocumentProjectionError(key)).toBe('command conflict');
  });

  it('clears cached projection state for one script document', async () => {
    getScriptDocumentProjectionMock.mockResolvedValue(projection);
    await refreshScriptDocumentProjection(key);

    clearScriptDocumentProjection(key);

    expect(getCachedScriptDocumentProjection(key)).toBeUndefined();
    expect(isScriptDocumentProjectionPending(key)).toBe(false);
    expect(getScriptDocumentProjectionError(key)).toBeUndefined();
  });
});
