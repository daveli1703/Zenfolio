import { describe, expect, test } from 'vitest';
import { formatDateOnly, formatWallTime } from './dates';

describe('date-only and wall-time formatting', () => {
  test('formats a stored date without converting it to an instant', () => {
    expect(formatDateOnly('2026-10-01', 'DD/MM/YYYY')).toBe('01/10/2026');
    expect(formatDateOnly('2026-10-01', 'MM/DD/YYYY')).toBe('10/01/2026');
    expect(formatDateOnly('2026-10-01', 'YYYY-MM-DD')).toBe('2026-10-01');
  });

  test('formats local wall time according to the preference', () => {
    expect(formatWallTime('09:30', '24h')).toBe('09:30');
    expect(formatWallTime('09:30', '12h')).toBe('9:30 AM');
    expect(formatWallTime('15:05', '12h')).toBe('3:05 PM');
  });
});
