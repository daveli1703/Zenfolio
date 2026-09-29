import { Goal } from 'lucide-react';
import { PlaceholderPage } from '../../components/ui/PlaceholderPage';

export function GoalsPage() {
  return (
    <PlaceholderPage
      eyebrow="Direction"
      title="Goals"
      description="Keep meaningful outcomes visible and measurable."
      milestone={6}
      feature="Goal tracking"
      icon={Goal}
    />
  );
}
