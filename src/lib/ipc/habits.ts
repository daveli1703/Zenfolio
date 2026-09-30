import { z } from 'zod';
import { invokeCommand } from './client';

export const targetTypeSchema = z.enum(['boolean', 'count', 'duration']);
export const habitSchema = z.object({
  id: z.string(),
  name: z.string(),
  description: z.string().nullable(),
  targetType: targetTypeSchema,
  unit: z.string().nullable(),
  color: z.string(),
  startDate: z.string(),
  archiveDate: z.string().nullable(),
  createdAt: z.string(),
  updatedAt: z.string(),
});
export const habitRuleSchema = z.object({
  id: z.string(),
  habitId: z.string(),
  effectiveDate: z.string(),
  target: z.number().int().positive(),
  weekdayMask: z.number().int().min(1).max(127),
  createdAt: z.string(),
  updatedAt: z.string(),
});
export const habitEntrySchema = z.object({
  id: z.string(),
  habitId: z.string(),
  date: z.string(),
  value: z.number().int().nonnegative(),
  notes: z.string().nullable(),
  createdAt: z.string(),
  updatedAt: z.string(),
});
export const habitDetailSchema = z.object({
  habit: habitSchema,
  rules: z.array(habitRuleSchema),
  hasEntries: z.boolean(),
});
const commonInput = {
  name: z.string().trim().min(1, 'Habit name is required.'),
  description: z.string().nullable(),
  targetType: targetTypeSchema,
  unit: z.string().nullable(),
  color: z.string().regex(/^#[0-9a-fA-F]{6}$/),
  startDate: z.string().regex(/^\d{4}-\d{2}-\d{2}$/),
};
export const createHabitInputSchema = z
  .object({
    ...commonInput,
    target: z.number().int().positive(),
    weekdayMask: z.number().int().min(1).max(127),
  })
  .superRefine((value, ctx) => {
    if (value.targetType === 'boolean' && value.target !== 1)
      ctx.addIssue({
        code: 'custom',
        path: ['target'],
        message: 'Boolean habits target one completion.',
      });
    if (value.targetType === 'count' && !value.unit?.trim())
      ctx.addIssue({
        code: 'custom',
        path: ['unit'],
        message: 'Count habits require a unit.',
      });
    if (value.targetType !== 'count' && value.unit)
      ctx.addIssue({
        code: 'custom',
        path: ['unit'],
        message: 'Only count habits use a custom unit.',
      });
  });
export const habitProfileInputSchema = z
  .object(commonInput)
  .superRefine((value, ctx) => {
    if (value.targetType === 'count' && !value.unit?.trim())
      ctx.addIssue({
        code: 'custom',
        path: ['unit'],
        message: 'Count habits require a unit.',
      });
    if (value.targetType !== 'count' && value.unit)
      ctx.addIssue({
        code: 'custom',
        path: ['unit'],
        message: 'Only count habits use a custom unit.',
      });
  });
export const habitRuleInputSchema = z.object({
  target: z.number().int().positive(),
  weekdayMask: z.number().int().min(1).max(127),
});
export const habitEntryInputSchema = z.object({
  habitId: z.string().min(1),
  date: z.string().regex(/^\d{4}-\d{2}-\d{2}$/),
  value: z.number().int().nonnegative(),
  notes: z.string().nullable(),
});
export const habitDaySchema = z.object({
  date: z.string(),
  state: z.enum([
    'eligible',
    'unscheduled',
    'future',
    'pre_start',
    'post_archive',
  ]),
  value: z.number().int().nonnegative(),
  notes: z.string().nullable(),
  target: z.number().int().positive().nullable(),
  completed: z.boolean(),
  intensity: z.union([
    z.literal(0),
    z.literal(1),
    z.literal(2),
    z.literal(3),
    z.literal(4),
  ]),
  progressRatio: z.number().nonnegative().nullable(),
});
export const habitTodaySchema = z.object({
  habit: habitSchema,
  day: habitDaySchema,
});
export const habitStatisticsSchema = z.object({
  currentStreak: z.number().int().nonnegative(),
  longestStreak: z.number().int().nonnegative(),
  completedDays: z.number().int().nonnegative(),
  elapsedScheduledDays: z.number().int().nonnegative(),
  completionRate: z.number().nonnegative().nullable(),
});
export const habitYearSchema = z.object({
  habit: habitSchema,
  year: z.number().int(),
  days: z.array(habitDaySchema),
  statistics: habitStatisticsSchema,
});
export const habitDeleteImpactSchema = z.object({
  ruleCount: z.number().int().nonnegative(),
  entryCount: z.number().int().nonnegative(),
});

export type Habit = z.infer<typeof habitSchema>;
export type HabitDetail = z.infer<typeof habitDetailSchema>;
export type CreateHabitInput = z.infer<typeof createHabitInputSchema>;
export type HabitProfileInput = z.infer<typeof habitProfileInputSchema>;
export type HabitRuleInput = z.infer<typeof habitRuleInputSchema>;
export type HabitEntryInput = z.infer<typeof habitEntryInputSchema>;
export type HabitDay = z.infer<typeof habitDaySchema>;
export type HabitYear = z.infer<typeof habitYearSchema>;

export const listHabits = () =>
  invokeCommand('list_habits', z.array(habitSchema));
export const getHabitDetail = (id: string) =>
  invokeCommand('get_habit_detail', habitDetailSchema, { id });
export const createHabit = (input: CreateHabitInput) =>
  invokeCommand('create_habit', habitDetailSchema, { input });
export const updateHabitProfile = (id: string, input: HabitProfileInput) =>
  invokeCommand('update_habit_profile', habitDetailSchema, { id, input });
export const scheduleHabitRuleChange = (id: string, input: HabitRuleInput) =>
  invokeCommand('schedule_habit_rule_change', habitDetailSchema, { id, input });
export const archiveHabit = (id: string) =>
  invokeCommand('archive_habit', habitDetailSchema, { input: { id } });
export const saveHabitEntry = (input: HabitEntryInput) =>
  invokeCommand('save_habit_entry', habitEntrySchema, { input });
export const listTodayHabits = () =>
  invokeCommand('list_today_habits', z.array(habitTodaySchema));
export const getHabitYear = (id: string, year: number) =>
  invokeCommand('get_habit_year', habitYearSchema, { id, year });
export const getHabitDeleteImpact = (id: string) =>
  invokeCommand('get_habit_delete_impact', habitDeleteImpactSchema, { id });
export const deleteHabit = (id: string) =>
  invokeCommand('delete_habit', z.null(), { input: { id } });
