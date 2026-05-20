import { afterEach, beforeEach, describe, expect, it, vi } from 'vitest';

import { createDebouncedStoryArcMetadataSave } from './debouncedStoryArcMetadataSave.js';

beforeEach(() => {
  vi.useFakeTimers();
});

afterEach(() => {
  vi.useRealTimers();
});

describe('debounced story arc metadata save', () => {
  it('saves the arc id and field captured when scheduled', async () => {
    const save = vi.fn().mockResolvedValue(undefined);
    const debounced = createDebouncedStoryArcMetadataSave({ delayMs: 500, save });

    debounced.schedule('arc.mystery', 'name', 'Mystery revised');

    await vi.advanceTimersByTimeAsync(500);

    expect(save).toHaveBeenCalledWith({
      arc_id: 'arc.mystery',
      name: 'Mystery revised',
    });
  });

  it('cancels older pending metadata when a newer edit is scheduled', async () => {
    const save = vi.fn().mockResolvedValue(undefined);
    const debounced = createDebouncedStoryArcMetadataSave({ delayMs: 500, save });

    debounced.schedule('arc.mystery', 'name', 'Mystery revised');
    debounced.schedule('arc.romance', 'description', 'Newer description');

    await vi.advanceTimersByTimeAsync(500);

    expect(save).toHaveBeenCalledTimes(1);
    expect(save).toHaveBeenCalledWith({
      arc_id: 'arc.romance',
      description: 'Newer description',
    });
  });

  it('does not flush pending metadata after disposal', async () => {
    const save = vi.fn().mockResolvedValue(undefined);
    const debounced = createDebouncedStoryArcMetadataSave({ delayMs: 500, save });

    debounced.schedule('arc.mystery', 'name', 'Mystery revised');
    debounced.dispose();

    await vi.advanceTimersByTimeAsync(500);

    expect(save).not.toHaveBeenCalled();
  });
});
