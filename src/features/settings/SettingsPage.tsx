import { PageHeader } from '../../components/ui/PageHeader';
import { BackupPanel } from './BackupPanel';
import { SettingsForm } from './SettingsForm';

export function SettingsPage() {
  return (
    <section className="page">
      <PageHeader
        eyebrow="Preferences"
        title="Settings"
        description="Shape Zenfolio around the way you work."
      />
      <div className="settings-layout">
        <SettingsForm />
        <BackupPanel />
      </div>
    </section>
  );
}
