import { type FormEvent, type ReactNode, useState } from 'react';
import { Modal } from '../tasks/TaskForm';
import {
  formatScaled,
  parseAtScale,
  parseGoalValues,
} from '../../lib/goalValues';
import {
  goalInputSchema,
  type Goal,
  type GoalInput,
  type GoalStatus,
} from '../../lib/ipc/goals';
import { IpcError } from '../../lib/ipc/errors';
import { useGoalMutations } from './goalsApi';

export function GoalForm({
  goal,
  onClose,
}: {
  goal: Goal | null;
  onClose: () => void;
}) {
  const mutations = useGoalMutations();
  const [title, setTitle] = useState(goal?.title ?? '');
  const [description, setDescription] = useState(goal?.description ?? '');
  const [target, setTarget] = useState(
    goal ? formatScaled(goal.targetValue, goal.decimalScale) : '',
  );
  const [current, setCurrent] = useState(
    goal ? formatScaled(goal.currentValue, goal.decimalScale) : '0',
  );
  const [unit, setUnit] = useState(goal?.unit ?? '');
  const [startDate, setStart] = useState(
    goal?.startDate ?? new Date().toLocaleDateString('en-CA'),
  );
  const [endDate, setEnd] = useState(goal?.endDate ?? '');
  const [status, setStatus] = useState<GoalStatus>(goal?.status ?? 'active');
  const [errors, setErrors] = useState<Record<string, string>>({});
  const [message, setMessage] = useState('');
  const submit = (event: FormEvent) => {
    event.preventDefault();
    setErrors({});
    setMessage('');
    let values;
    try {
      values = goal
        ? {
            targetValue: parseAtScale(target, goal.decimalScale),
            currentValue: parseAtScale(current, goal.decimalScale),
            decimalScale: goal.decimalScale,
          }
        : parseGoalValues(target, current);
    } catch (error) {
      const next: Record<string, string> = {};
      try {
        if (goal) parseAtScale(target, goal.decimalScale);
        else parseGoalValues(target, '0');
      } catch (targetError) {
        next.targetValue = (targetError as Error).message;
      }
      if (!next.targetValue) next.currentValue = (error as Error).message;
      setErrors(next);
      return;
    }
    const input: GoalInput = {
      title,
      description: description.trim() || null,
      ...values,
      unit,
      startDate,
      endDate: endDate || null,
      status,
    };
    const parsed = goalInputSchema.safeParse(input);
    if (!parsed.success) {
      const next: Record<string, string> = {};
      for (const issue of parsed.error.issues)
        next[String(issue.path[0])] = issue.message;
      setErrors(next);
      return;
    }
    const onError = (error: Error) => {
      if (error instanceof IpcError) {
        setErrors(error.fieldErrors ?? {});
        setMessage(error.message);
      } else
        setMessage('The goal could not be saved. Your changes are still here.');
    };
    if (goal)
      mutations.update.mutate(
        { id: goal.id, input: parsed.data },
        { onSuccess: onClose, onError },
      );
    else mutations.create.mutate(parsed.data, { onSuccess: onClose, onError });
  };
  const pending = mutations.create.isPending || mutations.update.isPending;
  return (
    <Modal title={goal ? 'Edit goal' : 'Create goal'} onClose={onClose}>
      <form className="task-form" onSubmit={submit}>
        <Field label="Title" error={errors.title}>
          <input
            autoFocus
            value={title}
            onChange={(e) => setTitle(e.target.value)}
          />
        </Field>
        <Field label="Description" error={errors.description}>
          <textarea
            rows={3}
            value={description}
            onChange={(e) => setDescription(e.target.value)}
          />
        </Field>
        <div className="task-form-grid">
          <Field label="Target" error={errors.targetValue}>
            <input
              inputMode="decimal"
              value={target}
              onChange={(e) => setTarget(e.target.value)}
            />
          </Field>
          <Field label="Current progress" error={errors.currentValue}>
            <input
              inputMode="decimal"
              value={current}
              onChange={(e) => setCurrent(e.target.value)}
            />
          </Field>
          <Field label="Unit" error={errors.unit}>
            <input value={unit} onChange={(e) => setUnit(e.target.value)} />
          </Field>
          <Field label="Status" error={errors.status}>
            <select
              value={status}
              onChange={(e) => setStatus(e.target.value as GoalStatus)}
            >
              <option value="active">Active</option>
              <option value="paused">Paused</option>
              <option value="completed">Completed</option>
              <option value="abandoned">Abandoned</option>
            </select>
          </Field>
          <Field label="Start date" error={errors.startDate}>
            <input
              type="date"
              value={startDate}
              onChange={(e) => setStart(e.target.value)}
            />
          </Field>
          <Field label="End date" error={errors.endDate}>
            <input
              type="date"
              value={endDate}
              onChange={(e) => setEnd(e.target.value)}
            />
          </Field>
        </div>
        {message ? (
          <p role="alert" className="form-message error-message">
            {message}
          </p>
        ) : null}
        <div className="form-actions task-form-actions">
          <button type="button" className="button" onClick={onClose}>
            Cancel
          </button>
          <button className="button button-primary" disabled={pending}>
            {pending ? 'Saving…' : goal ? 'Save goal' : 'Create goal'}
          </button>
        </div>
      </form>
    </Modal>
  );
}
function Field({
  label,
  error,
  children,
}: {
  label: string;
  error?: string;
  children: ReactNode;
}) {
  return (
    <label className="form-field">
      <span>{label}</span>
      {children}
      {error ? <span className="field-error">{error}</span> : null}
    </label>
  );
}
