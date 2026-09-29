import { Settings } from 'lucide-react';
import { PlaceholderPage } from '../../components/ui/PlaceholderPage';

export function SettingsPage() {
  return (
    <PlaceholderPage
      eyebrow="Preferences"
      title="Settings"
      description="Shape Zenfolio around the way you work."
      milestone={10}
      feature="Application settings and data management"
      icon={Settings}
    />
  );
}
