import { describe, expect, it } from 'vitest';

import {
  embeddedViewportBoundsChanged,
  embeddedViewportBoundsFromRect,
} from './embeddedViewportBounds.js';

describe('embedded viewport bounds helpers', () => {
  it('normalizes valid client bounds for desktop viewport IPC', () => {
    expect(
      embeddedViewportBoundsFromRect(
        {
          left: -4,
          top: 12,
          width: 640,
          height: 360,
        },
        2,
      ),
    ).toEqual({
      x: 0,
      y: 12,
      width: 640,
      height: 360,
      scale_factor: 2,
    });
  });

  it('rejects invisible or non-finite bounds', () => {
    expect(
      embeddedViewportBoundsFromRect(
        {
          left: 0,
          top: 0,
          width: 0,
          height: 360,
        },
        1,
      ),
    ).toBeNull();
    expect(
      embeddedViewportBoundsFromRect(
        {
          left: 0,
          top: 0,
          width: Number.NaN,
          height: 360,
        },
        1,
      ),
    ).toBeNull();
  });

  it('ignores tiny resize jitter before sending viewport updates', () => {
    const previous = {
      x: 0,
      y: 0,
      width: 640,
      height: 360,
      scale_factor: 1,
    };

    expect(
      embeddedViewportBoundsChanged(previous, {
        ...previous,
        width: 640.25,
      }),
    ).toBe(false);
    expect(
      embeddedViewportBoundsChanged(previous, {
        ...previous,
        width: 641,
      }),
    ).toBe(true);
  });
});
