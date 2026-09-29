import { Landmark } from 'lucide-react';
import { PlaceholderPage } from '../../components/ui/PlaceholderPage';

export function BudgetPage() {
  return (
    <PlaceholderPage
      eyebrow="Finances"
      title="Budget"
      description="Understand where your money goes each month."
      milestone={7}
      feature="Budget tracking"
      icon={Landmark}
    />
  );
}
