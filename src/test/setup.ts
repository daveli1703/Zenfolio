import '@testing-library/jest-dom/vitest';
import { cleanup } from '@testing-library/react';
import { afterEach, vi } from 'vitest';

const defaultSettings = {
  currencyCode: 'VND',
  currencyExponent: 0,
  applicationTimezone: 'Asia/Ho_Chi_Minh',
  dateFormat: 'DD/MM/YYYY',
  timeFormat: '24h',
  firstWeekday: 1,
  theme: 'system',
  createdAt: '2026-09-29T00:00:00.000Z',
  updatedAt: '2026-09-29T00:00:00.000Z',
};

vi.mock('@tauri-apps/api/core', () => ({
  invoke: vi.fn(async (command: string, args?: Record<string, unknown>) => {
    if (command === 'get_startup_status') {
      return {
        state: 'ready',
        databasePath: 'C:/Zenfolio/development/productivity.sqlite3',
        environment: 'development',
      };
    }
    if (command === 'get_settings') return defaultSettings;
    if (command === 'update_settings') {
      const input = (args?.input ?? {}) as Record<string, unknown>;
      return { ...defaultSettings, ...input };
    }
    if (command === 'get_storage_info') {
      return {
        databasePath: 'C:/Zenfolio/development/productivity.sqlite3',
        backupDirectory: 'C:/Zenfolio/development/backups',
        environment: 'development',
      };
    }
    throw new Error(`Unexpected test command: ${command}`);
  }),
}));

vi.mock('@tauri-apps/plugin-dialog', () => ({
  open: vi.fn(async () => null),
  save: vi.fn(async () => null),
}));

Object.defineProperty(window, 'matchMedia', {
  writable: true,
  value: (query: string) => ({
    matches: false,
    media: query,
    onchange: null,
    addListener: () => undefined,
    removeListener: () => undefined,
    addEventListener: () => undefined,
    removeEventListener: () => undefined,
    dispatchEvent: () => false,
  }),
});

afterEach(cleanup);
