import { z } from 'zod';
import { invokeCommand } from './client';

export const themePreferenceSchema = z.enum(['system', 'light', 'dark']);
export const dateFormatSchema = z.enum([
  'DD/MM/YYYY',
  'MM/DD/YYYY',
  'YYYY-MM-DD',
]);
export const timeFormatSchema = z.enum(['12h', '24h']);

export const appSettingsSchema = z.object({
  currencyCode: z.string().regex(/^[A-Z]{3}$/),
  currencyExponent: z.number().int().min(0).max(3),
  applicationTimezone: z.string().min(1),
  dateFormat: dateFormatSchema,
  timeFormat: timeFormatSchema,
  firstWeekday: z.union([z.literal(1), z.literal(7)]),
  theme: themePreferenceSchema,
  createdAt: z.string(),
  updatedAt: z.string(),
});

export const updateSettingsInputSchema = appSettingsSchema.omit({
  createdAt: true,
  updatedAt: true,
});

export type AppSettings = z.infer<typeof appSettingsSchema>;
export type UpdateSettingsInput = z.infer<typeof updateSettingsInputSchema>;

export function getSettings() {
  return invokeCommand('get_settings', appSettingsSchema);
}

export function updateSettings(input: UpdateSettingsInput) {
  return invokeCommand('update_settings', appSettingsSchema, { input });
}
