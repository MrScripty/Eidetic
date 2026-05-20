import { describe, expect, it } from 'vitest';

import { shouldReplaceProjection } from './projectionCacheGuards.js';

describe('projection cache guards', () => {
  it('accepts initial and same-or-newer projection versions', () => {
    const current = { version: 2, payload: { value: 'current' } };

    expect(shouldReplaceProjection(null, current)).toBe(true);
    expect(shouldReplaceProjection(current, { version: 2, payload: { value: 'same' } })).toBe(true);
    expect(shouldReplaceProjection(current, { version: 3, payload: { value: 'newer' } })).toBe(
      true,
    );
  });

  it('rejects older projection versions', () => {
    expect(
      shouldReplaceProjection(
        { version: 5, payload: { value: 'current' } },
        { version: 4, payload: { value: 'older' } },
      ),
    ).toBe(false);
  });
});
