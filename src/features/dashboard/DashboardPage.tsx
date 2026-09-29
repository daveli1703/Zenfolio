import { LayoutDashboard } from 'lucide-react';
import { PlaceholderPage } from '../../components/ui/PlaceholderPage';

export function DashboardPage() {
  return (
    <PlaceholderPage
      eyebrow="Overview"
      title="Dashboard"
      description="A calm starting point for your day."
      milestone={8}
      feature="Dashboard summaries"
      icon={LayoutDashboard}
    />
  );
}
