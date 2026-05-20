import { afterEach, beforeEach, describe, expect, it, vi } from 'vitest';

import { createDebouncedNodeNotesSave } from './debouncedNodeNotesSave.js';

beforeEach(() => {
  vi.useFakeTimers();
});

afterEach(() => {
  vi.useRealTimers();
});

describe('debounced node notes save', () => {
  it('saves the node id captured when the input was scheduled', async () => {
    const save = vi.fn().mockResolvedValue(undefined);
    const debounced = createDebouncedNodeNotesSave({ delayMs: 500, save });

    debounced.schedule('node.beat.first', 'first notes');

    await vi.advanceTimersByTimeAsync(500);

    expect(save).toHaveBeenCalledWith('node.beat.first', 'first notes');
  });

  it('cancels older pending notes when a newer edit is scheduled', async () => {
    const save = vi.fn().mockResolvedValue(undefined);
    const debounced = createDebouncedNodeNotesSave({ delayMs: 500, save });

    debounced.schedule('node.beat.first', 'first notes');
    debounced.schedule('node.beat.second', 'second notes');

    await vi.advanceTimersByTimeAsync(500);

    expect(save).toHaveBeenCalledTimes(1);
    expect(save).toHaveBeenCalledWith('node.beat.second', 'second notes');
  });

  it('does not flush pending notes after disposal', async () => {
    const save = vi.fn().mockResolvedValue(undefined);
    const debounced = createDebouncedNodeNotesSave({ delayMs: 500, save });

    debounced.schedule('node.beat.first', 'first notes');
    debounced.dispose();

    await vi.advanceTimersByTimeAsync(500);

    expect(save).not.toHaveBeenCalled();
  });
});
