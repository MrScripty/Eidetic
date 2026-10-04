import { beforeEach, describe, expect, it, vi } from 'vitest';
import { setScriptBlock, setScriptLock } from '$lib/commandApi.js';
import { getScriptDocumentProjection } from '$lib/projectionApi.js';
import type { ProjectionEnvelope } from '../projectionTypes.js';
import type { ScriptDocumentCommandResponse, ScriptDocumentProjection } from '../scriptTypes.js';
import {
  applyScriptBlockCommand,
  applyScriptLockCommand,
  clearScriptDocumentProjection,
  getCachedScriptDocumentProjection,
  getScriptDocumentProjectionError,
  isScriptDocumentProjectionPending,
  refreshScriptDocumentProjection,
} from './scriptDocumentProjection.svelte.js';

vi.mock('$lib/commandApi.js', () => ({ setScriptBlock: vi.fn(), setScriptLock: vi.fn() }));
vi.mock('$lib/projectionApi.js', () => ({ getScriptDocumentProjection: vi.fn() }));

const key = { document_id: 'script.document.main' };
const otherKey = { document_id: 'script.document.other' };
const readMock = vi.mocked(getScriptDocumentProjection);
const blockMock = vi.mocked(setScriptBlock);
const lockMock = vi.mocked(setScriptLock);

function envelope(version: number, title: string): ProjectionEnvelope<ScriptDocumentProjection> {
  return {
    version,
    change_event_id: `event-${title}`,
    payload: { document: { id: key.document_id, title, sort_order: 0 }, segments: [] },
  };
}

function deferred<T>() {
  let resolve!: (value: T) => void;
  let reject!: (error: unknown) => void;
  const promise = new Promise<T>((resolvePromise, rejectPromise) => {
    resolve = resolvePromise;
    reject = rejectPromise;
  });
  return { promise, resolve, reject };
}

const operations = [
  {
    name: 'refresh',
    start: (response: Promise<ScriptDocumentCommandResponse>) => {
      readMock.mockReturnValueOnce(response.then((result) => result.projection));
      return refreshScriptDocumentProjection(key);
    },
    result: (response: ScriptDocumentCommandResponse) => response.projection,
  },
  {
    name: 'block',
    start: (response: Promise<ScriptDocumentCommandResponse>) => {
      blockMock.mockReturnValueOnce(response);
      return applyScriptBlockCommand({
        document_id: key.document_id,
        document_title: 'Pilot',
        segment_id: 'segment',
        segment_start_ms: 0,
        segment_end_ms: 1000,
        segment_status: 'current',
        block_id: 'block',
        block_kind: 'action',
        text: 'Ada enters.',
      });
    },
    result: (response: ScriptDocumentCommandResponse) => response,
  },
  {
    name: 'lock',
    start: (response: Promise<ScriptDocumentCommandResponse>) => {
      lockMock.mockReturnValueOnce(response);
      return applyScriptLockCommand(
        { lock_id: 'lock', span_id: 'span', reason: 'Approved' },
        key.document_id,
      );
    },
    result: (response: ScriptDocumentCommandResponse) => response,
  },
] as const;

beforeEach(() => {
  clearScriptDocumentProjection(key);
  clearScriptDocumentProjection(otherKey);
  vi.resetAllMocks();
});

describe.each(operations)('$name cache lifetime', ({ start, result }) => {
  it('returns the original success after clear without repopulating the cache', async () => {
    const old = deferred<ScriptDocumentCommandResponse>();
    const request = start(old.promise);
    clearScriptDocumentProjection(key);
    const response: ScriptDocumentCommandResponse = {
      outcome: 'already_recorded',
      projection: envelope(99, 'old'),
    };
    old.resolve(response);
    await expect(request).resolves.toEqual(result(response));
    expect(getCachedScriptDocumentProjection(key)).toBeUndefined();
    expect(getScriptDocumentProjectionError(key)).toBeUndefined();
    expect(isScriptDocumentProjectionPending(key)).toBe(false);
  });

  it('cannot replace a lower-version new session or end its pending request', async () => {
    const old = deferred<ScriptDocumentCommandResponse>();
    const oldRequest = start(old.promise);
    clearScriptDocumentProjection(key);
    const current = envelope(1, 'current');
    readMock.mockResolvedValueOnce(current);
    await refreshScriptDocumentProjection(key);
    const pending = deferred<ProjectionEnvelope<ScriptDocumentProjection>>();
    readMock.mockReturnValueOnce(pending.promise);
    const newRequest = refreshScriptDocumentProjection(key);
    const response: ScriptDocumentCommandResponse = {
      outcome: 'recorded',
      projection: envelope(99, 'old'),
    };
    old.resolve(response);
    await expect(oldRequest).resolves.toEqual(result(response));
    expect(getCachedScriptDocumentProjection(key)).toEqual(current);
    expect(isScriptDocumentProjectionPending(key)).toBe(true);
    expect(getScriptDocumentProjectionError(key)).toBeUndefined();
    pending.resolve(envelope(2, 'new'));
    await newRequest;
    expect(isScriptDocumentProjectionPending(key)).toBe(false);
  });

  it('preserves caller rejection while leaving a new session error untouched', async () => {
    const old = deferred<ScriptDocumentCommandResponse>();
    const oldRequest = start(old.promise);
    const oldError = new Error('old session failure');
    const callerRejection = expect(oldRequest).rejects.toBe(oldError);
    clearScriptDocumentProjection(key);
    const current = envelope(1, 'current');
    readMock.mockResolvedValueOnce(current);
    await refreshScriptDocumentProjection(key);
    readMock.mockRejectedValueOnce(new Error('new session failure'));
    await expect(refreshScriptDocumentProjection(key)).rejects.toThrow('new session failure');
    old.reject(oldError);
    await callerRejection;
    expect(getCachedScriptDocumentProjection(key)).toEqual(current);
    expect(getScriptDocumentProjectionError(key)).toBe('new session failure');
    expect(isScriptDocumentProjectionPending(key)).toBe(false);
  });

  it('preserves caller rejection while a new session is pending', async () => {
    const old = deferred<ScriptDocumentCommandResponse>();
    const oldRequest = start(old.promise);
    const oldError = new Error('old session failure');
    const callerRejection = expect(oldRequest).rejects.toBe(oldError);
    clearScriptDocumentProjection(key);
    const pending = deferred<ProjectionEnvelope<ScriptDocumentProjection>>();
    readMock.mockReturnValueOnce(pending.promise);
    const newRequest = refreshScriptDocumentProjection(key);
    old.reject(oldError);
    await callerRejection;
    expect(getCachedScriptDocumentProjection(key)).toBeUndefined();
    expect(getScriptDocumentProjectionError(key)).toBeUndefined();
    expect(isScriptDocumentProjectionPending(key)).toBe(true);
    pending.resolve(envelope(1, 'new'));
    await newRequest;
    expect(isScriptDocumentProjectionPending(key)).toBe(false);
  });
});

describe('overlapping script requests', () => {
  it('counts mixed requests and orders projections by version rather than start or completion', async () => {
    const startPending = ({ start }: (typeof operations)[number]) => {
      const response = deferred<ScriptDocumentCommandResponse>();
      return { response, request: start(response.promise) };
    };
    const requests = [
      startPending(operations[0]),
      startPending(operations[1]),
      startPending(operations[2]),
    ] as const;
    requests[1].response.resolve({ outcome: 'recorded', projection: envelope(7, 'newest') });
    await requests[1].request;
    expect(isScriptDocumentProjectionPending(key)).toBe(true);
    requests[2].response.resolve({ outcome: 'already_recorded', projection: envelope(6, 'older') });
    await requests[2].request;
    expect(isScriptDocumentProjectionPending(key)).toBe(true);
    requests[0].response.resolve({ outcome: 'recorded', projection: envelope(5, 'oldest') });
    await requests[0].request;
    expect(getCachedScriptDocumentProjection(key)).toEqual(envelope(7, 'newest'));
    expect(isScriptDocumentProjectionPending(key)).toBe(false);
  });

  it('does not let an earlier failure overwrite the latest-started request error', async () => {
    const old = deferred<ScriptDocumentCommandResponse>();
    const oldRequest = operations[0].start(old.promise);
    const oldRejection = expect(oldRequest).rejects.toThrow('earlier failure');
    const latest = deferred<ScriptDocumentCommandResponse>();
    const latestRequest = operations[2].start(latest.promise);
    const latestRejection = expect(latestRequest).rejects.toThrow('latest failure');
    latest.reject(new Error('latest failure'));
    await latestRejection;
    expect(isScriptDocumentProjectionPending(key)).toBe(true);
    old.reject(new Error('earlier failure'));
    await oldRejection;
    expect(getScriptDocumentProjectionError(key)).toBe('latest failure');
    expect(isScriptDocumentProjectionPending(key)).toBe(false);
  });

  it('does not let an earlier failure introduce an error after the latest request succeeds', async () => {
    const old = deferred<ScriptDocumentCommandResponse>();
    const oldRequest = operations[1].start(old.promise);
    const oldRejection = expect(oldRequest).rejects.toThrow('earlier failure');
    readMock.mockResolvedValueOnce(envelope(2, 'current'));
    await refreshScriptDocumentProjection(key);
    expect(isScriptDocumentProjectionPending(key)).toBe(true);
    old.reject(new Error('earlier failure'));
    await oldRejection;
    expect(getScriptDocumentProjectionError(key)).toBeUndefined();
    expect(isScriptDocumentProjectionPending(key)).toBe(false);
  });

  it('clearing a different document does not invalidate this document request', async () => {
    const pending = deferred<ProjectionEnvelope<ScriptDocumentProjection>>();
    readMock.mockReturnValueOnce(pending.promise);
    const request = refreshScriptDocumentProjection(key);
    clearScriptDocumentProjection(otherKey);
    pending.resolve(envelope(1, 'current'));
    await request;
    expect(getCachedScriptDocumentProjection(key)).toEqual(envelope(1, 'current'));
    expect(isScriptDocumentProjectionPending(key)).toBe(false);
  });
});
