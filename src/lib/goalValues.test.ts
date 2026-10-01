import { describe, expect, it } from 'vitest';
import { formatScaled, parseAtScale, parseGoalValues } from './goalValues';

describe('goal value helpers', () => {
  it('converts decimal text to one exact shared scale', () => {
    expect(parseGoalValues('12.5', '1.025')).toEqual({
      targetValue: '12500',
      currentValue: '1025',
      decimalScale: 3,
    });
  });
  it('round trips scales zero through three without floating point', () => {
    expect(formatScaled('12', 0)).toBe('12');
    expect(formatScaled('12500', 3)).toBe('12.5');
    expect(parseAtScale('12.500', 3)).toBe('12500');
  });
  it('rejects excess precision', () =>
    expect(() => parseGoalValues('1.0001', '0')).toThrow());
});
