import type { AppSettings } from './ipc/settings';

export function formatDateOnly(
  value: string,
  format: AppSettings['dateFormat'],
): string {
  const [year, month, day] = value.split('-');
  if (!year || !month || !day) return value;
  if (format === 'MM/DD/YYYY') return `${month}/${day}/${year}`;
  if (format === 'YYYY-MM-DD') return value;
  return `${day}/${month}/${year}`;
}

export function formatWallTime(
  value: string,
  format: AppSettings['timeFormat'],
): string {
  if (format === '24h') return value;
  const [hourText, minute] = value.split(':');
  const hour = Number(hourText);
  if (!minute || !Number.isInteger(hour) || hour < 0 || hour > 23) return value;
  const suffix = hour >= 12 ? 'PM' : 'AM';
  const displayHour = hour % 12 || 12;
  return `${displayHour}:${minute} ${suffix}`;
}

export function currentDateInTimezone(
  timezone: string,
  instant = new Date(),
): string {
  const parts = new Intl.DateTimeFormat('en-CA', {
    timeZone: timezone,
    year: 'numeric',
    month: '2-digit',
    day: '2-digit',
  }).formatToParts(instant);
  const value = (type: string) =>
    parts.find((part) => part.type === type)?.value;
  return `${value('year')}-${value('month')}-${value('day')}`;
}
