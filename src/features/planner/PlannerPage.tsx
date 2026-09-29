import { CalendarDays } from 'lucide-react';
import { PlaceholderPage } from '../../components/ui/PlaceholderPage';

export function PlannerPage() {
  return (
    <PlaceholderPage
      eyebrow="Schedule"
      title="Planner"
      description="See how your time and commitments fit together."
      milestone={9}
      feature="Day, week, and month planning"
      icon={CalendarDays}
    />
  );
}
