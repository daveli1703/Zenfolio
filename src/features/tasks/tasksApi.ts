import { useMutation, useQuery, useQueryClient } from '@tanstack/react-query';
import {
  createProject,
  createTag,
  createTask,
  deleteProject,
  deleteTag,
  deleteTask,
  listProjects,
  listTags,
  listTasks,
  setProjectArchived,
  setTaskStatus,
  updateProject,
  updateTag,
  updateTask,
  type ProjectInput,
  type TagInput,
  type TaskFilters,
  type TaskInput,
  type TaskStatus,
} from '../../lib/ipc/tasks';

export const projectsQueryKey = ['projects'] as const;
export const tagsQueryKey = ['tags'] as const;
export const tasksQueryKey = ['tasks'] as const;

export function useProjectsQuery() {
  return useQuery({ queryKey: projectsQueryKey, queryFn: listProjects });
}

export function useTagsQuery() {
  return useQuery({ queryKey: tagsQueryKey, queryFn: listTags });
}

export function useTasksQuery(filters: TaskFilters) {
  return useQuery({
    queryKey: [...tasksQueryKey, filters],
    queryFn: () => listTasks(filters),
  });
}

export function useTaskMutations() {
  const queryClient = useQueryClient();
  const refresh = () =>
    queryClient.invalidateQueries({ queryKey: tasksQueryKey });
  return {
    create: useMutation({ mutationFn: createTask, onSuccess: refresh }),
    update: useMutation({
      mutationFn: ({ id, input }: { id: string; input: TaskInput }) =>
        updateTask(id, input),
      onSuccess: refresh,
    }),
    status: useMutation({
      mutationFn: ({ id, status }: { id: string; status: TaskStatus }) =>
        setTaskStatus(id, status),
      onSuccess: refresh,
    }),
    remove: useMutation({ mutationFn: deleteTask, onSuccess: refresh }),
  };
}

export function useProjectMutations() {
  const queryClient = useQueryClient();
  const refresh = async () => {
    await Promise.all([
      queryClient.invalidateQueries({ queryKey: projectsQueryKey }),
      queryClient.invalidateQueries({ queryKey: tasksQueryKey }),
    ]);
  };
  return {
    create: useMutation({ mutationFn: createProject, onSuccess: refresh }),
    update: useMutation({
      mutationFn: ({ id, input }: { id: string; input: ProjectInput }) =>
        updateProject(id, input),
      onSuccess: refresh,
    }),
    archive: useMutation({
      mutationFn: ({ id, archived }: { id: string; archived: boolean }) =>
        setProjectArchived(id, archived),
      onSuccess: refresh,
    }),
    remove: useMutation({ mutationFn: deleteProject, onSuccess: refresh }),
  };
}

export function useTagMutations() {
  const queryClient = useQueryClient();
  const refresh = async () => {
    await Promise.all([
      queryClient.invalidateQueries({ queryKey: tagsQueryKey }),
      queryClient.invalidateQueries({ queryKey: tasksQueryKey }),
    ]);
  };
  return {
    create: useMutation({ mutationFn: createTag, onSuccess: refresh }),
    update: useMutation({
      mutationFn: ({ id, input }: { id: string; input: TagInput }) =>
        updateTag(id, input),
      onSuccess: refresh,
    }),
    remove: useMutation({ mutationFn: deleteTag, onSuccess: refresh }),
  };
}
