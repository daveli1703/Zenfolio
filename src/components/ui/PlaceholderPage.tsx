import type { LucideIcon } from 'lucide-react';
import { EmptyState } from './EmptyState';
import { PageHeader } from './PageHeader';

type PlaceholderPageProps = {
  eyebrow: string;
  title: string;
  description: string;
  milestone: number;
  feature: string;
  icon: LucideIcon;
};

export function PlaceholderPage(props: PlaceholderPageProps) {
  return (
    <div className="page">
      <PageHeader
        eyebrow={props.eyebrow}
        title={props.title}
        description={props.description}
      />
      <EmptyState
        icon={props.icon}
        title={`${props.feature} is coming soon`}
        description={`${props.feature} will be implemented in Milestone ${props.milestone}.`}
      />
    </div>
  );
}
