import { useMutation, useQuery, useQueryClient } from '@tanstack/react-query';
import * as ipc from '../../lib/ipc/habits';
import type {
  CreateHabitInput,
  HabitEntryInput,
  HabitProfileInput,
  HabitRuleInput,
} from '../../lib/ipc/habits';

export const habitsKey = ['habits'] as const;
export const useHabitsQuery = () =>
  useQuery({ queryKey: [...habitsKey, 'list'], queryFn: ipc.listHabits });
export const useTodayHabitsQuery = () =>
  useQuery({ queryKey: [...habitsKey, 'today'], queryFn: ipc.listTodayHabits });
export const useHabitDetailQuery = (id: string | null) =>
  useQuery({
    queryKey: [...habitsKey, 'detail', id],
    queryFn: () => ipc.getHabitDetail(id!),
    enabled: !!id,
  });
export const useHabitYearQuery = (id: string | null, year: number) =>
  useQuery({
    queryKey: [...habitsKey, 'year', id, year],
    queryFn: () => ipc.getHabitYear(id!, year),
    enabled: !!id,
  });
export function useHabitMutations() {
  const client = useQueryClient();
  const refresh = () => client.invalidateQueries({ queryKey: habitsKey });
  return {
    create: useMutation({
      mutationFn: (input: CreateHabitInput) => ipc.createHabit(input),
      onSuccess: refresh,
    }),
    profile: useMutation({
      mutationFn: ({ id, input }: { id: string; input: HabitProfileInput }) =>
        ipc.updateHabitProfile(id, input),
      onSuccess: refresh,
    }),
    rule: useMutation({
      mutationFn: ({ id, input }: { id: string; input: HabitRuleInput }) =>
        ipc.scheduleHabitRuleChange(id, input),
      onSuccess: refresh,
    }),
    entry: useMutation({
      mutationFn: (input: HabitEntryInput) => ipc.saveHabitEntry(input),
      onSuccess: refresh,
    }),
    archive: useMutation({ mutationFn: ipc.archiveHabit, onSuccess: refresh }),
    remove: useMutation({ mutationFn: ipc.deleteHabit, onSuccess: refresh }),
  };
}
