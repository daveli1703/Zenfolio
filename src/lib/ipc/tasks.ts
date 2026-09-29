import { z } from 'zod';
import { invokeCommand } from './client';

export const taskPrioritySchema = z.enum(['low', 'medium', 'high']);
export const taskStatusSchema = z.enum(['todo', 'in_progress', 'done']);
export const taskSortFieldSchema = z.enum(['dueDate', 'createdAt', 'priority']);
export const sortDirectionSchema = z.enum(['asc', 'desc']);

export const projectSchema = z.object({
  id: z.string().min(1),
  name: z.string().min(1),
  color: z.string().nullable(),
  archived: z.boolean(),
  createdAt: z.string(),
  updatedAt: z.string(),
});

export const tagSchema = z.object({
  id: z.string().min(1),
  name: z.string().min(1),
  color: z.string().nullable(),
  createdAt: z.string(),
  updatedAt: z.string(),
});

export const taskSchema = z.object({
  id: z.string().min(1),
  title: z.string().min(1),
  notes: z.string().nullable(),
  dueDate: z.string().nullable(),
  dueTime: z.string().nullable(),
  priority: taskPrioritySchema,
  status: taskStatusSchema,
  project: projectSchema.nullable(),
  tags: z.array(tagSchema),
  completedAt: z.string().nullable(),
  createdAt: z.string(),
  updatedAt: z.string(),
});

const optionalText = z.string().trim().min(1).nullable();
export const taskInputSchema = z
  .object({
    title: z.string().trim().min(1, 'Task title is required.'),
    notes: optionalText,
    dueDate: z
      .string()
      .regex(/^\d{4}-\d{2}-\d{2}$/)
      .nullable(),
    dueTime: z
      .string()
      .regex(/^([01]\d|2[0-3]):[0-5]\d$/)
      .nullable(),
    priority: taskPrioritySchema,
    status: taskStatusSchema,
    projectId: optionalText,
    tagIds: z.array(z.string().min(1)),
  })
  .refine((value) => value.dueTime === null || value.dueDate !== null, {
    path: ['dueTime'],
    message: 'Choose a due date before adding a due time.',
  });

export const projectInputSchema = z.object({
  name: z.string().trim().min(1, 'Project name is required.'),
  color: z
    .string()
    .regex(/^#[0-9a-fA-F]{6}$/)
    .nullable(),
});

export const tagInputSchema = z.object({
  name: z.string().trim().min(1, 'Tag name is required.'),
  color: z
    .string()
    .regex(/^#[0-9a-fA-F]{6}$/)
    .nullable(),
});

export const taskFiltersSchema = z.object({
  search: optionalText.optional(),
  status: taskStatusSchema.nullable().optional(),
  priority: taskPrioritySchema.nullable().optional(),
  projectId: optionalText.optional(),
  tagId: optionalText.optional(),
  sortField: taskSortFieldSchema.default('dueDate'),
  sortDirection: sortDirectionSchema.default('asc'),
});

export type Project = z.infer<typeof projectSchema>;
export type ProjectInput = z.infer<typeof projectInputSchema>;
export type Tag = z.infer<typeof tagSchema>;
export type TagInput = z.infer<typeof tagInputSchema>;
export type Task = z.infer<typeof taskSchema>;
export type TaskInput = z.infer<typeof taskInputSchema>;
export type TaskFilters = z.infer<typeof taskFiltersSchema>;
export type TaskStatus = z.infer<typeof taskStatusSchema>;

export function listProjects() {
  return invokeCommand('list_projects', z.array(projectSchema));
}

export function createProject(input: ProjectInput) {
  return invokeCommand('create_project', projectSchema, { input });
}

export function updateProject(id: string, input: ProjectInput) {
  return invokeCommand('update_project', projectSchema, { id, input });
}

export function setProjectArchived(id: string, archived: boolean) {
  return invokeCommand('set_project_archived', projectSchema, {
    input: { id, archived },
  });
}

export function deleteProject(id: string) {
  return invokeCommand('delete_project', z.null(), { input: { id } });
}

export function listTags() {
  return invokeCommand('list_tags', z.array(tagSchema));
}

export function createTag(input: TagInput) {
  return invokeCommand('create_tag', tagSchema, { input });
}

export function updateTag(id: string, input: TagInput) {
  return invokeCommand('update_tag', tagSchema, { id, input });
}

export function deleteTag(id: string) {
  return invokeCommand('delete_tag', z.null(), { input: { id } });
}

export function listTasks(filters: TaskFilters) {
  return invokeCommand('list_tasks', z.array(taskSchema), { filters });
}

export function getTask(id: string) {
  return invokeCommand('get_task', taskSchema, { id });
}

export function createTask(input: TaskInput) {
  return invokeCommand('create_task', taskSchema, { input });
}

export function updateTask(id: string, input: TaskInput) {
  return invokeCommand('update_task', taskSchema, { id, input });
}

export function setTaskStatus(id: string, status: TaskStatus) {
  return invokeCommand('set_task_status', taskSchema, {
    input: { id, status },
  });
}

export function deleteTask(id: string) {
  return invokeCommand('delete_task', z.null(), { input: { id } });
}
