import { describe, expect, test } from 'vitest';
import { buildYearGrid, weekday } from './calendarGrid';
import type { HabitDay } from './ipc/habits';

function yearDays(year: number): HabitDay[] {
  const count =
    year % 4 === 0 && (year % 100 !== 0 || year % 400 === 0) ? 366 : 365;
  let month = 1,
    day = 1;
  const lengths = [
    31,
    count === 366 ? 29 : 28,
    31,
    30,
    31,
    30,
    31,
    31,
    30,
    31,
    30,
    31,
  ];
  return Array.from({ length: count }, () => {
    const date = `${year}-${String(month).padStart(2, '0')}-${String(day).padStart(2, '0')}`;
    day++;
    if (day > lengths[month - 1]) {
      month++;
      day = 1;
    }
    return {
      date,
      state: 'eligible',
      value: 0,
      notes: null,
      target: 1,
      completed: false,
      intensity: 0,
      progressRatio: 0,
    };
  });
}

describe('year calendar grid', () => {
  test('keeps every leap-year date and aligns Monday-first weeks', () => {
    const grid = buildYearGrid(yearDays(2024), 1);
    expect(
      grid.weeks.flatMap((w) => w.cells).filter((c) => c.day),
    ).toHaveLength(366);
    expect(grid.weeks[0].cells[0].day?.date).toBe('2024-01-01');
    expect(grid.monthLabels).toHaveLength(12);
  });
  test('adds Sunday-first leading placeholders without shifting dates', () => {
    const grid = buildYearGrid(yearDays(2024), 7);
    expect(grid.weeks[0].cells[0].day).toBeNull();
    expect(grid.weeks[0].cells[1].day?.date).toBe('2024-01-01');
    expect(
      grid.weeks
        .flatMap((w) => w.cells)
        .find((c) => c.day?.date === '2024-02-29'),
    ).toBeTruthy();
  });
  test('calculates weekdays without local timezone conversion', () => {
    expect(weekday('2024-01-01')).toBe(0);
    expect(weekday('2024-01-07')).toBe(6);
  });
});
