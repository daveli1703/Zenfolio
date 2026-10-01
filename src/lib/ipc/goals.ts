import { z } from 'zod';
import { invokeCommand } from './client';

const integer = z.string().regex(/^\d+$/);
export const goalStatusSchema = z.enum([
  'active',
  'completed',
  'paused',
  'abandoned',
]);
export const goalSchema = z.object({
  id: z.string().min(1),
  title: z.string().min(1),
  description: z.string().nullable(),
  targetValue: integer,
  currentValue: integer,
  decimalScale: z.number().int().min(0).max(3),
  unit: z.string().min(1),
  startDate: z.string(),
  endDate: z.string().nullable(),
  status: goalStatusSchema,
  completedAt: z.string().nullable(),
  progressBasisPoints: integer,
  createdAt: z.string(),
  updatedAt: z.string(),
});
export const goalInputSchema = z
  .object({
    title: z.string().trim().min(1, 'Goal title is required.'),
    description: z.string().trim().min(1).nullable(),
    targetValue: integer.refine(
      (v) => BigInt(v) > 0n,
      'Target must be greater than zero.',
    ),
    currentValue: integer,
    decimalScale: z.number().int().min(0).max(3),
    unit: z.string().trim().min(1, 'Unit is required.'),
    startDate: z.string().regex(/^\d{4}-\d{2}-\d{2}$/),
    endDate: z
      .string()
      .regex(/^\d{4}-\d{2}-\d{2}$/)
      .nullable(),
    status: goalStatusSchema,
  })
  .refine((v) => !v.endDate || v.endDate >= v.startDate, {
    path: ['endDate'],
    message: 'End date cannot be before start date.',
  });
export const goalFiltersSchema = z.object({
  status: goalStatusSchema.nullable().optional(),
  sortField: z
    .enum(['endDate', 'createdAt', 'progress', 'title'])
    .default('endDate'),
  sortDirection: z.enum(['asc', 'desc']).default('asc'),
});
export type Goal = z.infer<typeof goalSchema>;
export type GoalInput = z.infer<typeof goalInputSchema>;
export type GoalFilters = z.infer<typeof goalFiltersSchema>;
export type GoalStatus = z.infer<typeof goalStatusSchema>;
export const listGoals = (filters: GoalFilters) =>
  invokeCommand('list_goals', z.array(goalSchema), { filters });
export const createGoal = (input: GoalInput) =>
  invokeCommand('create_goal', goalSchema, { input });
export const updateGoal = (id: string, input: GoalInput) =>
  invokeCommand('update_goal', goalSchema, { id, input });
export const updateGoalProgress = (id: string, currentValue: string) =>
  invokeCommand('update_goal_progress', goalSchema, {
    input: { id, currentValue },
  });
export const updateGoalStatus = (id: string, status: GoalStatus) =>
  invokeCommand('update_goal_status', goalSchema, { input: { id, status } });
export const deleteGoal = (id: string) =>
  invokeCommand('delete_goal', z.null(), { input: { id } });
