import { describe, expect, test } from 'vitest';
import { appErrorSchema } from './errors';
import { appSettingsSchema } from './settings';
import { startupStatusSchema } from './startup';
import { projectSchema, tagSchema, taskInputSchema, taskSchema } from './tasks';
import { habitDetailSchema, habitYearSchema } from './habits';

describe('Tauri IPC contracts', () => {
  test('accepts the Rust settings payload shape', () => {
    expect(
      appSettingsSchema.parse({
        currencyCode: 'VND',
        currencyExponent: 0,
        applicationTimezone: 'Asia/Ho_Chi_Minh',
        dateFormat: 'DD/MM/YYYY',
        timeFormat: '24h',
        firstWeekday: 1,
        theme: 'system',
        createdAt: '2026-09-29T00:00:00.000Z',
        updatedAt: '2026-09-29T00:00:00.000Z',
      }).theme,
    ).toBe('system');
  });

  test('accepts recovery startup and field-error payloads', () => {
    const error = appErrorSchema.parse({
      code: 'VALIDATION_ERROR',
      message: 'Check the highlighted settings and try again.',
      fieldErrors: { applicationTimezone: 'Use a valid IANA timezone.' },
    });
    const status = startupStatusSchema.parse({
      state: 'recovery',
      databasePath: 'C:/Zenfolio/development/productivity.sqlite3',
      environment: 'development',
      error,
    });
    expect(status.error?.code).toBe('VALIDATION_ERROR');
  });

  test('accepts task, project, and tag payloads using camelCase fields', () => {
    const project = projectSchema.parse({
      id: 'project-1',
      name: 'Work',
      color: null,
      archived: false,
      createdAt: 'created',
      updatedAt: 'updated',
    });
    const tag = tagSchema.parse({
      id: 'tag-1',
      name: 'Focus',
      color: '#17735a',
      createdAt: 'created',
      updatedAt: 'updated',
    });
    const task = taskSchema.parse({
      id: 'task-1',
      title: 'Write report',
      notes: null,
      dueDate: '2026-10-01',
      dueTime: '09:30',
      priority: 'high',
      status: 'todo',
      project,
      tags: [tag],
      completedAt: null,
      createdAt: 'created',
      updatedAt: 'updated',
    });
    expect(task.project?.name).toBe('Work');
    expect(task.tags[0].name).toBe('Focus');
  });

  test('task input rejects a due time without a due date', () => {
    const parsed = taskInputSchema.safeParse({
      title: 'Write report',
      notes: null,
      dueDate: null,
      dueTime: '09:30',
      priority: 'medium',
      status: 'todo',
      projectId: null,
      tagIds: [],
    });
    expect(parsed.success).toBe(false);
  });

  test('accepts historical habit and heatmap payloads', () => {
    const habit = {
      id: 'habit-1',
      name: 'Read',
      description: null,
      targetType: 'count',
      unit: 'pages',
      color: '#17735a',
      startDate: '2024-01-01',
      archiveDate: null,
      createdAt: 'created',
      updatedAt: 'updated',
    };
    expect(
      habitDetailSchema.parse({
        habit,
        rules: [
          {
            id: 'rule-1',
            habitId: 'habit-1',
            effectiveDate: '2024-01-01',
            target: 10,
            weekdayMask: 127,
            createdAt: 'created',
            updatedAt: 'updated',
          },
        ],
        hasEntries: true,
      }).rules[0].target,
    ).toBe(10);
    expect(
      habitYearSchema.parse({
        habit,
        year: 2024,
        days: [
          {
            date: '2024-02-29',
            state: 'eligible',
            value: 10,
            notes: null,
            target: 10,
            completed: true,
            intensity: 4,
            progressRatio: 1,
          },
        ],
        statistics: {
          currentStreak: 1,
          longestStreak: 1,
          completedDays: 1,
          elapsedScheduledDays: 1,
          completionRate: 1,
        },
      }).days[0].date,
    ).toBe('2024-02-29');
  });
});
