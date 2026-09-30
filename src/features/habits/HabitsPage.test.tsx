import { invoke } from '@tauri-apps/api/core';
import { fireEvent, render, screen, waitFor } from '@testing-library/react';
import { beforeEach, describe, expect, test, vi } from 'vitest';
import { App } from '../../app/App';

const settings = {
  currencyCode: 'VND',
  currencyExponent: 0,
  applicationTimezone: 'Asia/Ho_Chi_Minh',
  dateFormat: 'DD/MM/YYYY',
  timeFormat: '24h',
  firstWeekday: 1,
  theme: 'system',
  createdAt: 'created',
  updatedAt: 'updated',
};
const habit = {
  id: 'habit-1',
  name: 'Read',
  description: null,
  targetType: 'boolean',
  unit: null,
  color: '#17735a',
  startDate: '2026-09-28',
  archiveDate: null,
  createdAt: 'created',
  updatedAt: 'updated',
};
const rule = {
  id: 'rule-1',
  habitId: habit.id,
  effectiveDate: habit.startDate,
  target: 1,
  weekdayMask: 127,
  createdAt: 'created',
  updatedAt: 'updated',
};
const day = {
  date: '2026-09-30',
  state: 'eligible',
  value: 0,
  notes: null,
  target: 1,
  completed: false,
  intensity: 0,
  progressRatio: 0,
};
function handler(command: string, args?: Record<string, unknown>) {
  if (command === 'get_startup_status')
    return { state: 'ready', databasePath: 'db', environment: 'development' };
  if (command === 'get_settings') return settings;
  if (command === 'list_habits') return [habit];
  if (command === 'list_today_habits') return [{ habit, day }];
  if (command === 'get_habit_detail')
    return { habit, rules: [rule], hasEntries: false };
  if (command === 'get_habit_year')
    return {
      habit,
      year: args?.year as number,
      days: [day],
      statistics: {
        currentStreak: 0,
        longestStreak: 0,
        completedDays: 0,
        elapsedScheduledDays: 0,
        completionRate: null,
      },
    };
  if (command === 'save_habit_entry')
    return {
      id: 'entry',
      ...(args?.input as object),
      createdAt: 'created',
      updatedAt: 'updated',
    };
  if (command === 'schedule_habit_rule_change')
    return { habit, rules: [rule], hasEntries: false };
  if (command === 'create_habit')
    return {
      habit: { ...habit, ...(args?.input as object), id: 'new' },
      rules: [rule],
      hasEntries: false,
    };
  throw new Error(`Unexpected command ${command}`);
}
describe('Habits page', () => {
  beforeEach(() => {
    window.location.hash = '#/habits';
    vi.mocked(invoke).mockImplementation(async (command, args) =>
      handler(command, args as Record<string, unknown>),
    );
  });
  test('records today and schedules a rule change through typed commands', async () => {
    render(<App />);
    fireEvent.click(
      (await screen.findAllByRole('button', { name: /Read/ }))[0],
    );
    expect(
      await screen.findByRole('dialog', { name: 'Log Read' }),
    ).toBeInTheDocument();
    fireEvent.click(screen.getByLabelText('Completed'));
    fireEvent.click(screen.getByRole('button', { name: 'Save entry' }));
    await waitFor(() =>
      expect(invoke).toHaveBeenCalledWith('save_habit_entry', {
        input: expect.objectContaining({ habitId: 'habit-1', value: 1 }),
      }),
    );
    fireEvent.click(
      await screen.findByRole('button', { name: 'Target & schedule' }),
    );
    fireEvent.click(screen.getByLabelText('Sunday'));
    fireEvent.click(screen.getByRole('button', { name: 'Save for tomorrow' }));
    await waitFor(() =>
      expect(invoke).toHaveBeenCalledWith('schedule_habit_rule_change', {
        id: 'habit-1',
        input: { target: 1, weekdayMask: 63 },
      }),
    );
  });
  test('creates a count habit with a selected-weekday schedule', async () => {
    render(<App />);
    const createButton = await screen.findByRole('button', {
      name: 'New habit',
    });
    await waitFor(() => expect(createButton).not.toBeDisabled());
    fireEvent.click(createButton);
    fireEvent.change(screen.getByLabelText('Name'), {
      target: { value: 'Drink water' },
    });
    fireEvent.change(screen.getByLabelText('Type'), {
      target: { value: 'count' },
    });
    fireEvent.change(screen.getByLabelText('Unit'), {
      target: { value: 'glasses' },
    });
    fireEvent.change(screen.getByLabelText('Target'), {
      target: { value: '8' },
    });
    fireEvent.click(screen.getByLabelText('Sunday'));
    fireEvent.click(screen.getByRole('button', { name: 'Create habit' }));
    await waitFor(() =>
      expect(invoke).toHaveBeenCalledWith('create_habit', {
        input: expect.objectContaining({
          name: 'Drink water',
          targetType: 'count',
          unit: 'glasses',
          target: 8,
          weekdayMask: 63,
        }),
      }),
    );
  });
});
