import { MoonStar } from 'lucide-react';
import { type ThemePreference, useTheme } from './theme';

export function ThemeControl() {
  const { preference, setPreference } = useTheme();

  return (
    <label className="theme-control">
      <MoonStar aria-hidden="true" size={17} />
      <span>Theme</span>
      <select
        aria-label="Color theme"
        value={preference}
        onChange={(event) =>
          setPreference(event.target.value as ThemePreference)
        }
      >
        <option value="system">System</option>
        <option value="light">Light</option>
        <option value="dark">Dark</option>
      </select>
    </label>
  );
}
