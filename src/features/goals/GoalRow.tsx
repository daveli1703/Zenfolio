import { useState } from 'react';
import { formatScaled, parseAtScale } from '../../lib/goalValues';
import type { Goal, GoalStatus } from '../../lib/ipc/goals';
import { useGoalMutations } from './goalsApi';

export function GoalRow({ goal, onEdit }: { goal: Goal; onEdit: () => void }) {
  const mutations = useGoalMutations();
  const [progress, setProgress] = useState(
    formatScaled(goal.currentValue, goal.decimalScale),
  );
  const basis = Number(goal.progressBasisPoints);
  const percent = (basis / 100).toLocaleString(undefined, {
    maximumFractionDigits: 2,
  });
  const save = () => {
    try {
      mutations.progress.mutate({
        id: goal.id,
        currentValue: parseAtScale(progress, goal.decimalScale),
      });
    } catch {
      return;
    }
  };
  const remove = () => {
    if (window.confirm(`Delete “${goal.title}”?`))
      mutations.remove.mutate(goal.id);
  };
  return (
    <article className="goal-card">
      <div className="goal-card-header">
        <div>
          <span className={`status-badge status-${goal.status}`}>
            {goal.status}
          </span>
          <h2>{goal.title}</h2>
          {goal.description ? <p>{goal.description}</p> : null}
        </div>
        <button className="button" onClick={onEdit}>
          Edit
        </button>
      </div>
      <div className="goal-progress" aria-label={`${percent}% progress`}>
        <div style={{ width: `${Math.min(100, basis / 100)}%` }} />
      </div>
      <div className="goal-values">
        <strong>
          {formatScaled(goal.currentValue, goal.decimalScale)} /{' '}
          {formatScaled(goal.targetValue, goal.decimalScale)} {goal.unit}
        </strong>
        <span>{percent}%</span>
      </div>
      <div className="goal-actions">
        <label>
          Progress{' '}
          <input
            aria-label={`Progress for ${goal.title}`}
            inputMode="decimal"
            value={progress}
            onChange={(e) => setProgress(e.target.value)}
          />
        </label>
        <button className="button" onClick={save}>
          Update
        </button>
        <select
          aria-label={`Status for ${goal.title}`}
          value={goal.status}
          onChange={(e) =>
            mutations.status.mutate({
              id: goal.id,
              status: e.target.value as GoalStatus,
            })
          }
        >
          <option value="active">Active</option>
          <option value="paused">Paused</option>
          <option value="completed">Completed</option>
          <option value="abandoned">Abandoned</option>
        </select>
        <button className="button danger-button" onClick={remove}>
          Delete
        </button>
      </div>
      <p className="goal-dates">
        Started {goal.startDate}
        {goal.endDate ? ` · Ends ${goal.endDate}` : ''}
        {goal.completedAt ? ' · Completed' : ''}
      </p>
    </article>
  );
}
