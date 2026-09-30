import { useMemo, useRef, useState } from 'react';
import { buildYearGrid } from '../../lib/calendarGrid';
import { formatDateOnly } from '../../lib/dates';
import type { AppSettings } from '../../lib/ipc/settings';
import type { HabitDay, Habit } from '../../lib/ipc/habits';
const monthNames = [
  'Jan',
  'Feb',
  'Mar',
  'Apr',
  'May',
  'Jun',
  'Jul',
  'Aug',
  'Sep',
  'Oct',
  'Nov',
  'Dec',
];
export function HabitHeatmap({
  habit,
  days,
  settings,
  onActivate,
}: {
  habit: Habit;
  days: HabitDay[];
  settings: AppSettings;
  onActivate: (day: HabitDay) => void;
}) {
  const grid = useMemo(
    () => buildYearGrid(days, settings.firstWeekday),
    [days, settings.firstWeekday],
  );
  const eligible =
    days.find((d) => d.state === 'eligible')?.date ?? days[0]?.date ?? '';
  const [focusDate, setFocusDate] = useState(eligible);
  const refs = useRef(new Map<string, HTMLButtonElement>());
  const move = (date: string, delta: number) => {
    const index = days.findIndex((d) => d.date === date);
    const next = days[index + delta];
    if (next) {
      setFocusDate(next.date);
      refs.current.get(next.date)?.focus();
    }
  };
  return (
    <div className="heatmap-scroll">
      <div
        className="heatmap"
        role="grid"
        aria-label={`${habit.name} yearly activity`}
      >
        <div className="heatmap-months" aria-hidden="true">
          {grid.monthLabels.map((label) => (
            <span key={label.month} style={{ gridColumn: label.weekIndex + 1 }}>
              {monthNames[label.month - 1]}
            </span>
          ))}
        </div>
        <div className="heatmap-weeks">
          {grid.weeks.map((week) => (
            <div className="heatmap-week" role="row" key={week.key}>
              {week.cells.map((cell) =>
                cell.day ? (
                  <button
                    key={cell.key}
                    ref={(node) => {
                      if (node) refs.current.set(cell.day!.date, node);
                      else refs.current.delete(cell.day!.date);
                    }}
                    type="button"
                    role="gridcell"
                    tabIndex={cell.day.date === focusDate ? 0 : -1}
                    className={`heatmap-cell state-${cell.day.state} level-${cell.day.intensity}`}
                    aria-label={label(cell.day, habit, settings)}
                    aria-disabled={cell.day.state !== 'eligible'}
                    title={label(cell.day, habit, settings)}
                    onFocus={() => setFocusDate(cell.day!.date)}
                    onClick={() =>
                      cell.day!.state === 'eligible' && onActivate(cell.day!)
                    }
                    onKeyDown={(e) => {
                      const delta =
                        e.key === 'ArrowRight'
                          ? 7
                          : e.key === 'ArrowLeft'
                            ? -7
                            : e.key === 'ArrowDown'
                              ? 1
                              : e.key === 'ArrowUp'
                                ? -1
                                : 0;
                      if (delta) {
                        e.preventDefault();
                        move(cell.day!.date, delta);
                      } else if (
                        (e.key === 'Enter' || e.key === ' ') &&
                        cell.day!.state === 'eligible'
                      ) {
                        e.preventDefault();
                        onActivate(cell.day!);
                      }
                    }}
                  />
                ) : (
                  <span className="heatmap-cell placeholder" key={cell.key} />
                ),
              )}
            </div>
          ))}
        </div>
      </div>
    </div>
  );
}
function label(day: HabitDay, habit: Habit, settings: AppSettings) {
  const unit = habit.unit ?? (habit.targetType === 'duration' ? 'minutes' : '');
  const status =
    day.state === 'eligible'
      ? day.completed
        ? 'completed'
        : 'not completed'
      : day.state.replace('_', ' ');
  return `${formatDateOnly(day.date, settings.dateFormat)}: ${day.value}${unit ? ` ${unit}` : ''} of ${day.target ?? '—'}, ${status}`;
}
