import { invoke } from '@tauri-apps/api/core';
import { fireEvent, render, screen, waitFor } from '@testing-library/react';
import { beforeEach, describe, expect, test, vi } from 'vitest';
import { App } from '../../app/App';
import type { Goal } from '../../lib/ipc/goals';

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
const goal: Goal = {
  id: 'goal-1',
  title: 'Run',
  description: null,
  targetValue: '10000',
  currentValue: '2500',
  decimalScale: 2,
  unit: 'km',
  startDate: '2026-01-01',
  endDate: '2026-12-31',
  status: 'active',
  completedAt: null,
  progressBasisPoints: '2500',
  createdAt: 'created',
  updatedAt: 'updated',
};

describe('Goals page', () => {
  beforeEach(() => {
    window.location.hash = '#/goals';
    let goals: (typeof goal)[] = [];
    vi.mocked(invoke).mockImplementation(async (command, args) => {
      if (command === 'get_startup_status')
        return {
          state: 'ready',
          databasePath: 'db',
          environment: 'development',
        };
      if (command === 'get_settings') return settings;
      if (command === 'list_goals') return goals;
      if (command === 'create_goal') {
        const input = (args as { input: typeof goal }).input;
        const created = { ...goal, ...input };
        goals = [created];
        return created;
      }
      if (command === 'update_goal_progress') {
        const input = (args as { input: { id: string; currentValue: string } })
          .input;
        const updated: Goal = {
          ...goals[0],
          currentValue: input.currentValue,
          progressBasisPoints: '12500',
        };
        goals = [updated];
        return updated;
      }
      if (command === 'update_goal_status') {
        const input = (args as { input: { status: string } }).input;
        const updated = {
          ...goals[0],
          status: input.status as Goal['status'],
          completedAt: input.status === 'completed' ? 'done' : null,
        };
        goals = [updated];
        return updated;
      }
      if (command === 'delete_goal') {
        goals = [];
        return null;
      }
      throw new Error(`Unexpected command ${command}`);
    });
  });
  test('creates exact decimal values and supports progress and explicit completion', async () => {
    render(<App />);
    await screen.findByText('Set your first goal');
    fireEvent.click(screen.getByRole('button', { name: 'New goal' }));
    fireEvent.change(screen.getByLabelText('Title'), {
      target: { value: 'Run' },
    });
    fireEvent.change(screen.getByLabelText('Target'), {
      target: { value: '100.00' },
    });
    fireEvent.change(screen.getByLabelText('Current progress'), {
      target: { value: '25.00' },
    });
    fireEvent.change(screen.getByLabelText('Unit'), {
      target: { value: 'km' },
    });
    fireEvent.click(screen.getByRole('button', { name: 'Create goal' }));
    expect(
      await screen.findByRole('heading', { name: 'Run' }),
    ).toBeInTheDocument();
    expect(invoke).toHaveBeenCalledWith('create_goal', {
      input: expect.objectContaining({
        targetValue: '10000',
        currentValue: '2500',
        decimalScale: 2,
      }),
    });
    fireEvent.change(screen.getByLabelText('Progress for Run'), {
      target: { value: '125' },
    });
    fireEvent.click(screen.getByRole('button', { name: 'Update' }));
    await waitFor(() =>
      expect(invoke).toHaveBeenCalledWith('update_goal_progress', {
        input: { id: 'goal-1', currentValue: '12500' },
      }),
    );
    fireEvent.change(screen.getByLabelText('Status for Run'), {
      target: { value: 'completed' },
    });
    await waitFor(() =>
      expect(invoke).toHaveBeenCalledWith('update_goal_status', {
        input: { id: 'goal-1', status: 'completed' },
      }),
    );
  });
});
