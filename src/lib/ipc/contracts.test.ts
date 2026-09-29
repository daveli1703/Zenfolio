import { describe, expect, test } from 'vitest';
import { appErrorSchema } from './errors';
import { appSettingsSchema } from './settings';
import { startupStatusSchema } from './startup';
import { projectSchema, tagSchema, taskInputSchema, taskSchema } from './tasks';

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
});
