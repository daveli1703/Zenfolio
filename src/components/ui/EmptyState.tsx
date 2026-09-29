import type { LucideIcon } from 'lucide-react';

type EmptyStateProps = { icon: LucideIcon; title: string; description: string };

export function EmptyState({
  icon: Icon,
  title,
  description,
}: EmptyStateProps) {
  return (
    <section className="empty-state" aria-labelledby="empty-state-title">
      <div className="empty-state-icon" aria-hidden="true">
        <Icon size={25} strokeWidth={1.7} />
      </div>
      <h2 id="empty-state-title">{title}</h2>
      <p>{description}</p>
    </section>
  );
}
