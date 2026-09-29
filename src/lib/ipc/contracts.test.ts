import { describe, expect, test } from 'vitest';
import { appErrorSchema } from './errors';
import { appSettingsSchema } from './settings';
import { startupStatusSchema } from './startup';

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
});
