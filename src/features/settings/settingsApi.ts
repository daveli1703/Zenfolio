import { useMutation, useQuery, useQueryClient } from '@tanstack/react-query';
import {
  getSettings,
  updateSettings,
  type UpdateSettingsInput,
} from '../../lib/ipc/settings';
import { getStorageInfo } from '../../lib/ipc/startup';

export const settingsQueryKey = ['settings'] as const;

export function useSettingsQuery() {
  return useQuery({ queryKey: settingsQueryKey, queryFn: getSettings });
}

export function useStorageInfoQuery() {
  return useQuery({ queryKey: ['storage-info'], queryFn: getStorageInfo });
}

export function useUpdateSettingsMutation() {
  const queryClient = useQueryClient();
  return useMutation({
    mutationFn: (input: UpdateSettingsInput) => updateSettings(input),
    onSuccess: (settings) => {
      queryClient.setQueryData(settingsQueryKey, settings);
    },
  });
}
