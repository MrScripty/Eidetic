import { beforeEach, describe, expect, it, vi } from 'vitest';

import { editScriptBlock, setScriptBlock, setScriptLock } from '$lib/commandApi.js';
import { getScriptDocumentProjection } from '$lib/projectionApi.js';
import type { ScriptImpactProjection } from '$lib/scriptTypes.js';
import {
  applyScriptBlockCommand,
  applyScriptBlockEditCommand,
  applyScriptLockCommand,
  clearScriptDocumentProjection,
  getCachedScriptDocumentProjection,
  getScriptDocumentProjectionError,
  isScriptDocumentProjectionPending,
  refreshScriptDocumentProjection,
  scriptDocumentProjectionState,
} from './scriptDocumentProjection.svelte.js';

vi.mock('$lib/commandApi.js', () => ({
  editScriptBlock: vi.fn(),
  setScriptBlock: vi.fn(),
  setScriptLock: vi.fn(),
}));

vi.mock('$lib/projectionApi.js', () => ({
  getScriptDocumentProjection: vi.fn(),
}));

const setScriptBlockMock = vi.mocked(setScriptBlock);
const setScriptLockMock = vi.mocked(setScriptLock);
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

const newerProjection = {
  ...projection,
  version: 7,
  change_event_id: 'event-script-7',
  payload: {
    ...projection.payload,
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
              id: 'script.block.action-2',
              segment_id: 'script.segment.beat-1',
              block_kind: 'action' as const,
              text: 'Ada enters through fog.',
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

const olderProjection = {
  ...projection,
  version: 6,
  change_event_id: 'event-script-6',
  payload: {
    ...projection.payload,
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
              id: 'script.block.action-3',
              segment_id: 'script.segment.beat-1',
              block_kind: 'action' as const,
              text: 'Ada enters in sunlight.',
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
  setScriptLockMock.mockReset();
  getScriptDocumentProjectionMock.mockReset();
});

describe('script document projection store', () => {
  it('refreshes review impact without changing authored output and refuses stale clean results', async () => {
    const impact: ScriptImpactProjection = {
      generation_event_id: 'generation-B',
      lineage_available: true,
      needs_review: true,
      causes: [
        {
          dependency_id: 'generation-B.input-A',
          input: { kind: 'script_block', block_id: 'source-A' },
          consumed_revision_event_id: 'A-before',
          current_revision_event_id: null,
          reason: 'deleted',
          input_excerpt: '  Consumed A — 雨\n\n',
        },
      ],
    };
    const reviewed = {
      ...newerProjection,
      payload: {
        ...newerProjection.payload,
        segments: newerProjection.payload.segments.map((segment) => ({ ...segment, impact })),
      },
    };
    getScriptDocumentProjectionMock.mockResolvedValueOnce(reviewed);
    await refreshScriptDocumentProjection(key);
    getScriptDocumentProjectionMock.mockResolvedValueOnce(olderProjection);
    await refreshScriptDocumentProjection(key);
    const cached = getCachedScriptDocumentProjection(key);
    expect(cached?.payload.segments[0]?.impact).toEqual(impact);
    expect(cached?.payload.segments[0]?.blocks).toEqual(
      newerProjection.payload.segments[0]?.blocks,
    );
  });
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

  it('does not replace cached script projections with stale refresh results', async () => {
    getScriptDocumentProjectionMock.mockResolvedValueOnce(newerProjection);
    await refreshScriptDocumentProjection(key);
    getScriptDocumentProjectionMock.mockResolvedValueOnce(olderProjection);

    await expect(refreshScriptDocumentProjection(key)).resolves.toEqual(olderProjection);

    expect(getCachedScriptDocumentProjection(key)).toEqual(newerProjection);
    expect(isScriptDocumentProjectionPending(key)).toBe(false);
    expect(getScriptDocumentProjectionError(key)).toBeUndefined();
  });

  it('does not replace cached script projections with stale command responses', async () => {
    getScriptDocumentProjectionMock.mockResolvedValueOnce(newerProjection);
    await refreshScriptDocumentProjection(key);
    setScriptBlockMock.mockResolvedValue({
      outcome: 'recorded',
      projection: olderProjection,
    });

    await expect(
      applyScriptBlockCommand({
        document_id: 'script.document/main one',
        document_title: 'Pilot',
        segment_id: 'script.segment.beat-1',
        segment_start_ms: 1000,
        segment_end_ms: 5000,
        segment_status: 'current',
        block_id: 'script.block.action-3',
        block_kind: 'action',
        text: 'Ada enters in sunlight.',
      }),
    ).resolves.toEqual({
      outcome: 'recorded',
      projection: olderProjection,
    });

    expect(getCachedScriptDocumentProjection(key)).toEqual(newerProjection);
    expect(isScriptDocumentProjectionPending(key)).toBe(false);
    expect(getScriptDocumentProjectionError(key)).toBeUndefined();
  });

  it('stores lock command response projections by explicit document id', async () => {
    setScriptLockMock.mockResolvedValue({
      outcome: 'recorded',
      projection,
    });

    await expect(
      applyScriptLockCommand(
        {
          lock_id: 'script.lock.action-1',
          span_id: 'script.block.action-1.span.main',
          reason: 'User approved wording.',
        },
        'script.document/main one',
        'command-lock-1',
      ),
    ).resolves.toEqual({
      outcome: 'recorded',
      projection,
    });

    expect(setScriptLockMock).toHaveBeenCalledWith(
      {
        lock_id: 'script.lock.action-1',
        span_id: 'script.block.action-1.span.main',
        reason: 'User approved wording.',
      },
      'command-lock-1',
    );
    expect(getCachedScriptDocumentProjection(key)).toEqual(projection);
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

it('saves only text and its expected revision, publishes projection and invalidates prompt context', async () => {
  const payload = {
    document_id: key.document_id,
    block_id: 'script.block.action-1',
    expected_revision_event_id: 'event-script-1',
    text: '  Ada leaves under clear skies.\n\nBEN\nWait.  ',
  };
  vi.mocked(editScriptBlock).mockResolvedValue({
    outcome: 'recorded',
    projection: newerProjection,
  });
  const revision = scriptDocumentProjectionState.contextRevision;
  await applyScriptBlockEditCommand(payload, 'manual-edit-1');
  expect(editScriptBlock).toHaveBeenCalledWith(payload, 'manual-edit-1');
  expect(getCachedScriptDocumentProjection(key)).toEqual(newerProjection);
  expect(scriptDocumentProjectionState.contextRevision).toBe(revision + 1);
});

it('keeps the last committed projection and context revision when an edit is refused', async () => {
  getScriptDocumentProjectionMock.mockResolvedValue(projection);
  await refreshScriptDocumentProjection(key);
  vi.mocked(editScriptBlock).mockRejectedValue(
    new Error('script block changed; reload before saving'),
  );
  const revision = scriptDocumentProjectionState.contextRevision;
  await expect(
    applyScriptBlockEditCommand({
      document_id: key.document_id,
      block_id: 'script.block.action-1',
      expected_revision_event_id: 'old-event',
      text: 'Unsaved draft',
    }),
  ).rejects.toThrow('changed');
  expect(getCachedScriptDocumentProjection(key)).toEqual(projection);
  expect(scriptDocumentProjectionState.contextRevision).toBe(revision);
  expect(getScriptDocumentProjectionError(key)).toContain('changed');
});
