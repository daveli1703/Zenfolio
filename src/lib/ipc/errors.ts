import { z } from 'zod';

export const appErrorSchema = z.object({
  code: z.string(),
  message: z.string(),
  fieldErrors: z.record(z.string(), z.string()).optional(),
});

export type AppErrorPayload = z.infer<typeof appErrorSchema>;

export class IpcError extends Error {
  readonly code: string;
  readonly fieldErrors?: Record<string, string>;

  constructor(payload: AppErrorPayload) {
    super(payload.message);
    this.name = 'IpcError';
    this.code = payload.code;
    this.fieldErrors = payload.fieldErrors;
  }
}

export function normalizeIpcError(error: unknown): IpcError {
  const parsed = appErrorSchema.safeParse(error);
  if (parsed.success) return new IpcError(parsed.data);
  if (error instanceof IpcError) return error;
  return new IpcError({
    code: 'INTERNAL_ERROR',
    message: 'Zenfolio could not complete the operation.',
  });
}
