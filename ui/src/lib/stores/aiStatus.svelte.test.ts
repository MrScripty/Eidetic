import { afterEach, beforeEach, describe, expect, it, vi } from 'vitest';

import { editorState } from './editor.svelte.js';
import { refreshAiStatus, startAiStatusPolling } from './aiStatus.svelte.js';
import { getAiStatus } from '$lib/api.js';

vi.mock('$lib/api.js', () => ({
  getAiStatus: vi.fn(),
}));

const getAiStatusMock = vi.mocked(getAiStatus);

beforeEach(() => {
  editorState.aiStatus = null;
  vi.useFakeTimers();
  getAiStatusMock.mockReset();
});

afterEach(() => {
  vi.useRealTimers();
});

describe('ai status polling', () => {
  it('updates editor state when refresh succeeds', async () => {
    getAiStatusMock.mockResolvedValue({
      backend: 'ollama',
      connected: true,
      error: undefined,
      model: 'llama3',
    });

    await refreshAiStatus();

    expect(editorState.aiStatus).toMatchObject({
      backend: 'ollama',
      connected: true,
      model: 'llama3',
    });
  });

  it('marks the current backend disconnected when refresh fails', async () => {
    editorState.aiStatus = {
      backend: 'open_router',
      connected: true,
      error: undefined,
      model: 'gpt-4o-mini',
    };
    getAiStatusMock.mockRejectedValue(new Error('offline'));

    await refreshAiStatus();

    expect(editorState.aiStatus).toMatchObject({
      backend: 'open_router',
      connected: false,
      error: 'Failed to reach server',
    });
  });

  it('shares a single polling interval across multiple owners', async () => {
    getAiStatusMock.mockResolvedValue({
      backend: 'ollama',
      connected: true,
      error: undefined,
      model: 'llama3',
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
});
