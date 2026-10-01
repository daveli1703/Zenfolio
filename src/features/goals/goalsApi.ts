import { useMutation, useQuery, useQueryClient } from '@tanstack/react-query';
import {
  createGoal,
  deleteGoal,
  listGoals,
  updateGoal,
  updateGoalProgress,
  updateGoalStatus,
  type GoalFilters,
  type GoalInput,
  type GoalStatus,
} from '../../lib/ipc/goals';
export const goalsQueryKey = ['goals'] as const;
export const useGoalsQuery = (filters: GoalFilters) =>
  useQuery({
    queryKey: [...goalsQueryKey, filters],
    queryFn: () => listGoals(filters),
  });
export function useGoalMutations() {
  const client = useQueryClient();
  const refresh = () => client.invalidateQueries({ queryKey: goalsQueryKey });
  return {
    create: useMutation({ mutationFn: createGoal, onSuccess: refresh }),
    update: useMutation({
      mutationFn: ({ id, input }: { id: string; input: GoalInput }) =>
        updateGoal(id, input),
      onSuccess: refresh,
    }),
    progress: useMutation({
      mutationFn: ({
        id,
        currentValue,
      }: {
        id: string;
        currentValue: string;
      }) => updateGoalProgress(id, currentValue),
      onSuccess: refresh,
    }),
    status: useMutation({
      mutationFn: ({ id, status }: { id: string; status: GoalStatus }) =>
        updateGoalStatus(id, status),
      onSuccess: refresh,
    }),
    remove: useMutation({ mutationFn: deleteGoal, onSuccess: refresh }),
  };
}
