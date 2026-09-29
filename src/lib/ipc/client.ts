import { invoke } from '@tauri-apps/api/core';
import { type ZodType } from 'zod';
import { normalizeIpcError } from './errors';

export async function invokeCommand<T>(
  command: string,
  schema: ZodType<T>,
  args?: Record<string, unknown>,
): Promise<T> {
  try {
    return schema.parse(await invoke(command, args));
  } catch (error) {
    if (error instanceof Error && error.name === 'ZodError') {
      throw new Error(`Invalid response from ${command}`, { cause: error });
    }
    throw normalizeIpcError(error);
  }
}
