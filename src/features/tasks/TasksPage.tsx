import { CheckSquare2 } from 'lucide-react';
import { PlaceholderPage } from '../../components/ui/PlaceholderPage';

export function TasksPage() {
  return (
    <PlaceholderPage
      eyebrow="Focus"
      title="Tasks"
      description="Organize the work that matters to you."
      milestone={4}
      feature="Task management"
      icon={CheckSquare2}
    />
  );
}
