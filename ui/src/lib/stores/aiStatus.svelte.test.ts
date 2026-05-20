import { afterEach, beforeEach, describe, expect, it, vi } from 'vitest';

import {
  aiStatusState,
  refreshAiStatus,
  resetAiStatusPollingForTests,
  startAiStatusPolling,
} from './aiStatus.svelte.js';
import { getAiStatus } from '$lib/api.js';

vi.mock('$lib/api.js', () => ({
  getAiStatus: vi.fn(),
}));

const getAiStatusMock = vi.mocked(getAiStatus);

beforeEach(() => {
  resetAiStatusPollingForTests();
  aiStatusState.status = null;
  vi.useFakeTimers();
  getAiStatusMock.mockReset();
});

afterEach(() => {
  resetAiStatusPollingForTests();
  vi.useRealTimers();
});

describe('ai status polling', () => {
  it('updates the ai status cache when refresh succeeds', async () => {
    getAiStatusMock.mockResolvedValue({
      backend: 'llama_cpp',
      connected: true,
      error: undefined,
      model: 'served-model',
    });

    await refreshAiStatus();

    expect(aiStatusState.status).toMatchObject({
      backend: 'llama_cpp',
      connected: true,
      model: 'served-model',
    });
  });

  it('marks the current backend disconnected when refresh fails', async () => {
    aiStatusState.status = {
      backend: 'open_router',
      connected: true,
      error: undefined,
      model: 'gpt-4o-mini',
    };
    getAiStatusMock.mockRejectedValue(new Error('offline'));

    await refreshAiStatus();

    expect(aiStatusState.status).toMatchObject({
      backend: 'open_router',
      connected: false,
      error: 'Failed to reach server',
    });
  });

  it('shares a single polling interval across multiple owners', async () => {
    getAiStatusMock.mockResolvedValue({
      backend: 'llama_cpp',
      connected: true,
      error: undefined,
      model: 'served-model',
    });

    const stopFirst = startAiStatusPolling();
    const stopSecond = startAiStatusPolling();

    await Promise.resolve();
    expect(getAiStatusMock).toHaveBeenCalledTimes(1);

    await vi.advanceTimersByTimeAsync(30_000);
    expect(getAiStatusMock).toHaveBeenCalledTimes(2);

    stopFirst();
    await vi.advanceTimersByTimeAsync(30_000);
    expect(getAiStatusMock).toHaveBeenCalledTimes(3);

    stopSecond();
    await vi.advanceTimersByTimeAsync(30_000);
    expect(getAiStatusMock).toHaveBeenCalledTimes(3);
  });

  it('ignores stale status responses from older overlapping refreshes', async () => {
    let resolveFirst: (status: Awaited<ReturnType<typeof getAiStatus>>) => void = () => {};
    const firstRefresh = new Promise<Awaited<ReturnType<typeof getAiStatus>>>((resolve) => {
      resolveFirst = resolve;
    });
    getAiStatusMock.mockReturnValueOnce(firstRefresh).mockResolvedValueOnce({
      backend: 'llama_cpp',
      connected: true,
      error: undefined,
      model: 'newer-model',
    });

    const first = refreshAiStatus();
    await refreshAiStatus();
    resolveFirst({
      backend: 'llama_cpp',
      connected: true,
      error: undefined,
      model: 'stale-model',
    });
    await first;

    expect(aiStatusState.status).toMatchObject({
      backend: 'llama_cpp',
      connected: true,
      model: 'newer-model',
    });
  });
});
