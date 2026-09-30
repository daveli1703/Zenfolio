import type { HabitDay } from './ipc/habits';

export type CalendarCell = { key: string; day: HabitDay | null };
export type CalendarWeek = { key: string; cells: CalendarCell[] };
export type CalendarGrid = {
  weeks: CalendarWeek[];
  monthLabels: { month: number; weekIndex: number }[];
};

export function buildYearGrid(
  days: HabitDay[],
  firstWeekday: 1 | 7,
): CalendarGrid {
  if (!days.length) return { weeks: [], monthLabels: [] };
  const offset = (weekday(days[0].date) - (firstWeekday === 1 ? 0 : 6) + 7) % 7;
  const cells: CalendarCell[] = [];
  for (let index = 0; index < offset; index++)
    cells.push({ key: `before-${index}`, day: null });
  for (const day of days) cells.push({ key: day.date, day });
  while (cells.length % 7)
    cells.push({ key: `after-${cells.length}`, day: null });
  const weeks = [] as CalendarWeek[];
  for (let index = 0; index < cells.length; index += 7)
    weeks.push({
      key: `week-${index / 7}`,
      cells: cells.slice(index, index + 7),
    });
  const monthLabels: { month: number; weekIndex: number }[] = [];
  let last = 0;
  for (const [weekIndex, week] of weeks.entries()) {
    const month = week.cells
      .map((c) => (c.day ? Number(c.day.date.slice(5, 7)) : 0))
      .find((value) => value !== 0 && value !== last);
    if (month) {
      monthLabels.push({ month, weekIndex });
      last = month;
    }
  }
  return { weeks, monthLabels };
}

// Gregorian weekday with Monday=0. It never converts a date-only value to an instant.
export function weekday(value: string): number {
  let year = Number(value.slice(0, 4));
  const month = Number(value.slice(5, 7));
  const day = Number(value.slice(8, 10));
  const offsets = [0, 3, 2, 5, 0, 3, 5, 1, 4, 6, 2, 4];
  if (month < 3) year -= 1;
  const sunday =
    (year +
      Math.floor(year / 4) -
      Math.floor(year / 100) +
      Math.floor(year / 400) +
      offsets[month - 1] +
      day) %
    7;
  return (sunday + 6) % 7;
}
