import { z } from 'zod';
import { invokeCommand } from './client';
import { appErrorSchema } from './errors';

export const startupStatusSchema = z.object({
  state: z.enum(['ready', 'recovery']),
  databasePath: z.string(),
  environment: z.enum(['development', 'production', 'test']),
  error: appErrorSchema.optional(),
});

export const storageInfoSchema = z.object({
  databasePath: z.string(),
  backupDirectory: z.string(),
  environment: z.enum(['development', 'production', 'test']),
});

export type StartupStatus = z.infer<typeof startupStatusSchema>;
export type StorageInfo = z.infer<typeof storageInfoSchema>;

export function getStartupStatus() {
  return invokeCommand('get_startup_status', startupStatusSchema);
}

export function getStorageInfo() {
  return invokeCommand('get_storage_info', storageInfoSchema);
}
