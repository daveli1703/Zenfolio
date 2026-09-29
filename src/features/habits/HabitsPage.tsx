import { Leaf } from 'lucide-react';
import { PlaceholderPage } from '../../components/ui/PlaceholderPage';

export function HabitsPage() {
  return (
    <PlaceholderPage
      eyebrow="Consistency"
      title="Habits"
      description="Build routines with clear, gentle progress."
      milestone={5}
      feature="Habit tracking"
      icon={Leaf}
    />
  );
}
