import { ArrowDownAZ, Plus } from 'lucide-react';
import { useMemo, useState } from 'react';
import { useSearchParams } from 'react-router-dom';
import { PageHeader } from '../../components/ui/PageHeader';
import { goalFiltersSchema, type Goal } from '../../lib/ipc/goals';
import { GoalForm } from './GoalForm';
import { GoalRow } from './GoalRow';
import { useGoalsQuery } from './goalsApi';

export function GoalsPage() {
  const [params, setParams] = useSearchParams();
  const filters = useMemo(
    () =>
      goalFiltersSchema.parse({
        status: params.get('status') || null,
        sortField: params.get('sort') || 'endDate',
        sortDirection: params.get('direction') || 'asc',
      }),
    [params],
  );
  const goals = useGoalsQuery(filters);
  const [editing, setEditing] = useState<Goal | null | undefined>(undefined);
  const set = (key: string, value: string) => {
    const next = new URLSearchParams(params);
    if (value) next.set(key, value);
    else next.delete(key);
    setParams(next, { replace: true });
  };
  return (
    <div className="page goals-page">
      <div className="tasks-heading-row">
        <PageHeader
          eyebrow="Direction"
          title="Goals"
          description="Keep meaningful outcomes visible and measurable."
        />
        <button
          className="button button-primary"
          onClick={() => setEditing(null)}
        >
          <Plus aria-hidden="true" /> New goal
        </button>
      </div>
      <section className="task-toolbar" aria-label="Goal filters">
        <select
          aria-label="Filter goals by status"
          value={filters.status ?? ''}
          onChange={(e) => set('status', e.target.value)}
        >
          <option value="">All statuses</option>
          <option value="active">Active</option>
          <option value="paused">Paused</option>
          <option value="completed">Completed</option>
          <option value="abandoned">Abandoned</option>
        </select>
        <select
          aria-label="Sort goals"
          value={filters.sortField}
          onChange={(e) => set('sort', e.target.value)}
        >
          <option value="endDate">End date</option>
          <option value="createdAt">Created date</option>
          <option value="progress">Progress</option>
          <option value="title">Title</option>
        </select>
        <button
          className="icon-button sort-direction"
          aria-label="Change sort direction"
          onClick={() =>
            set('direction', filters.sortDirection === 'asc' ? 'desc' : 'asc')
          }
        >
          <ArrowDownAZ
            className={filters.sortDirection === 'desc' ? 'icon-rotated' : ''}
          />
        </button>
      </section>
      {goals.isPending ? (
        <State title="Loading goals…" text="Reading your local goals." />
      ) : null}
      {goals.isError ? (
        <State
          title="Goals could not be loaded"
          text="Check the local database and try again."
        />
      ) : null}
      {goals.data?.length ? (
        <section className="goal-list" aria-label="Goals">
          {goals.data.map((goal) => (
            <GoalRow
              key={goal.id}
              goal={goal}
              onEdit={() => setEditing(goal)}
            />
          ))}
        </section>
      ) : null}
      {!goals.isPending && !goals.isError && !goals.data?.length ? (
        <State
          title={
            filters.status
              ? 'No goals match this filter'
              : 'Set your first goal'
          }
          text={
            filters.status
              ? 'Choose another status to see more goals.'
              : 'Add a measurable outcome and track it over time.'
          }
          action={() =>
            filters.status ? setParams({}, { replace: true }) : setEditing(null)
          }
          label={filters.status ? 'Clear filter' : 'Create first goal'}
        />
      ) : null}
      {editing !== undefined ? (
        <GoalForm goal={editing} onClose={() => setEditing(undefined)} />
      ) : null}
    </div>
  );
}

function State({
  title,
  text,
  action,
  label,
}: {
  title: string;
  text: string;
  action?: () => void;
  label?: string;
}) {
  return (
    <section className="task-state">
      <h2>{title}</h2>
      <p>{text}</p>
      {action ? (
        <button className="button button-primary" onClick={action}>
          {label}
        </button>
      ) : null}
    </section>
  );
}
