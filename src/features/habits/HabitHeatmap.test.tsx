import { fireEvent, render, screen } from '@testing-library/react';
import { describe, expect, test, vi } from 'vitest';
import { HabitHeatmap } from './HabitHeatmap';
const habit = {
  id: 'h',
  name: 'Read',
  description: null,
  targetType: 'count' as const,
  unit: 'pages',
  color: '#17735a',
  startDate: '2024-01-01',
  archiveDate: null,
  createdAt: 'created',
  updatedAt: 'updated',
};
const settings = {
  currencyCode: 'VND',
  currencyExponent: 0,
  applicationTimezone: 'Asia/Ho_Chi_Minh',
  dateFormat: 'DD/MM/YYYY' as const,
  timeFormat: '24h' as const,
  firstWeekday: 1 as const,
  theme: 'system' as const,
  createdAt: 'created',
  updatedAt: 'updated',
};
const days = [
  {
    date: '2024-01-01',
    state: 'eligible' as const,
    value: 10,
    notes: null,
    target: 10,
    completed: true,
    intensity: 4 as const,
    progressRatio: 1,
  },
  {
    date: '2024-01-02',
    state: 'unscheduled' as const,
    value: 0,
    notes: null,
    target: 10,
    completed: false,
    intensity: 0 as const,
    progressRatio: 0,
  },
  {
    date: '2024-01-03',
    state: 'eligible' as const,
    value: 2,
    notes: null,
    target: 10,
    completed: false,
    intensity: 1 as const,
    progressRatio: 0.2,
  },
];
describe('Habit heatmap', () => {
  test('labels historical progress and activates eligible dates by keyboard', () => {
    const activate = vi.fn();
    render(
      <HabitHeatmap
        habit={habit}
        days={days}
        settings={settings}
        onActivate={activate}
      />,
    );
    const first = screen.getByRole('gridcell', {
      name: /01\/01\/2024.*completed/,
    });
    first.focus();
    fireEvent.keyDown(first, { key: 'ArrowDown' });
    expect(
      screen.getByRole('gridcell', { name: /02\/01\/2024.*unscheduled/ }),
    ).toHaveFocus();
    fireEvent.keyDown(screen.getByRole('gridcell', { name: /03\/01\/2024/ }), {
      key: 'Enter',
    });
    expect(activate).toHaveBeenCalledWith(days[2]);
  });
});
