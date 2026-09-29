import { type ReactNode, useEffect, useMemo, useState } from 'react';
import {
  useSettingsQuery,
  useUpdateSettingsMutation,
} from '../features/settings/settingsApi';
import {
  type ResolvedTheme,
  ThemeContext,
  type ThemePreference,
} from './theme';

const darkThemeQuery = '(prefers-color-scheme: dark)';

export function ThemeProvider({ children }: { children: ReactNode }) {
  const settings = useSettingsQuery();
  const updateSettings = useUpdateSettingsMutation();
  const [pendingPreference, setPendingPreference] =
    useState<ThemePreference | null>(null);
  const [systemTheme, setSystemTheme] = useState<ResolvedTheme>(() =>
    window.matchMedia(darkThemeQuery).matches ? 'dark' : 'light',
  );
  const preference = pendingPreference ?? settings.data?.theme ?? 'system';
  const resolvedTheme = preference === 'system' ? systemTheme : preference;

  useEffect(() => {
    const mediaQuery = window.matchMedia(darkThemeQuery);
    const updateSystemTheme = (event: MediaQueryListEvent) => {
      setSystemTheme(event.matches ? 'dark' : 'light');
    };

    mediaQuery.addEventListener('change', updateSystemTheme);
    return () => mediaQuery.removeEventListener('change', updateSystemTheme);
  }, []);

  useEffect(() => {
    document.documentElement.dataset.theme = resolvedTheme;
    document.documentElement.style.colorScheme = resolvedTheme;
  }, [resolvedTheme]);

  const value = useMemo(() => {
    const persistPreference = (theme: ThemePreference) => {
      if (!settings.data || updateSettings.isPending) return;
      setPendingPreference(theme);
      updateSettings.mutate(
        {
          currencyCode: settings.data.currencyCode,
          currencyExponent: settings.data.currencyExponent,
          applicationTimezone: settings.data.applicationTimezone,
          dateFormat: settings.data.dateFormat,
          timeFormat: settings.data.timeFormat,
          firstWeekday: settings.data.firstWeekday,
          theme,
        },
        { onSettled: () => setPendingPreference(null) },
      );
    };
    return {
      preference,
      resolvedTheme,
      setPreference: persistPreference,
      isSaving: settings.isPending || updateSettings.isPending,
    };
  }, [
    preference,
    resolvedTheme,
    settings.data,
    settings.isPending,
    updateSettings,
  ]);

  return (
    <ThemeContext.Provider value={value}>{children}</ThemeContext.Provider>
  );
}
