import { z } from 'zod';
import { invokeCommand } from './client';

export const backupResultSchema = z.object({
  path: z.string(),
  schemaVersion: z.number().int().nonnegative(),
});

export type BackupResult = z.infer<typeof backupResultSchema>;

export function createManualBackup(path: string) {
  return invokeCommand('create_manual_backup', backupResultSchema, {
    input: { path },
  });
}

export function validateBackup(path: string) {
  return invokeCommand('validate_backup', backupResultSchema, {
    input: { path },
  });
}
