import { describe, expect, it } from 'vitest';
import { formatProjectModified } from './projectModified.js';

describe('saved project modification time', () => {
  it('displays the backend Unix-second timestamp as the corresponding date', () => {
    expect(formatProjectModified('1700000000')).toBe(formatProjectModified('2023-11-14T22:13:20Z'));
    expect(formatProjectModified('0')).toBe(formatProjectModified('1970-01-01T00:00:00Z'));
  });

  it('shows a readable fallback when metadata is missing or invalid', () => {
    for (const value of ['', ' ', 'invalid', '999999999999999999999']) {
      expect(formatProjectModified(value)).toBe('Unknown date');
    }
  });
});
