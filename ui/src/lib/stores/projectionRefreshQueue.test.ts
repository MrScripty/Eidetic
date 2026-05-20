import { beforeEach, describe, expect, it, vi } from 'vitest';

import { clearProjectionRefreshQueue, requestProjectionRefresh } from './projectionRefreshQueue.js';

beforeEach(() => {
  clearProjectionRefreshQueue();
});

describe('projection refresh queue', () => {
  it('coalesces same-key refresh bursts into one task', async () => {
    const task = vi.fn().mockResolvedValue(undefined);

    const first = requestProjectionRefresh('timeline-render', task);
    const second = requestProjectionRefresh('timeline-render', task);

    await Promise.all([first, second]);

    expect(task).toHaveBeenCalledTimes(1);
  });

  it('runs one follow-up refresh when a request arrives during an in-flight refresh', async () => {
    let resolveFirst: () => void = () => {};
    const firstRun = new Promise<void>((resolve) => {
      resolveFirst = resolve;
    });
    const task = vi.fn().mockReturnValueOnce(firstRun).mockResolvedValueOnce(undefined);

    const first = requestProjectionRefresh('timeline-render', task);
    await vi.waitFor(() => {
      expect(task).toHaveBeenCalledTimes(1);
    });
    const second = requestProjectionRefresh('timeline-render', task);
    resolveFirst();
    await Promise.all([first, second]);

    expect(task).toHaveBeenCalledTimes(2);
  });

  it('resolves callers and swallows task errors so stores own visible error state', async () => {
    const task = vi.fn().mockRejectedValue(new Error('projection unavailable'));

    await expect(requestProjectionRefresh('timeline-render', task)).resolves.toBeUndefined();

    expect(task).toHaveBeenCalledTimes(1);
  });

  it('resolves queued callers when the queue is cleared during teardown', async () => {
    const task = vi.fn().mockResolvedValue(undefined);
    const queued = requestProjectionRefresh('timeline-render', task);

    clearProjectionRefreshQueue();

    await expect(queued).resolves.toBeUndefined();
    expect(task).not.toHaveBeenCalled();
  });
});
