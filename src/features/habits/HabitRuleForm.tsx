import { type FormEvent, useState } from 'react';
import type { HabitDetail } from '../../lib/ipc/habits';
import { habitRuleInputSchema } from '../../lib/ipc/habits';
import { useHabitMutations } from './habitsApi';
import { HabitDialog } from './HabitDialog';
import { WeekdaySelector } from './WeekdaySelector';
export function HabitRuleForm({
  detail,
  onClose,
}: {
  detail: HabitDetail;
  onClose: () => void;
}) {
  const latest = detail.rules.at(-1)!;
  const [target, setTarget] = useState(latest.target);
  const [mask, setMask] = useState(latest.weekdayMask);
  const [error, setError] = useState('');
  const mutation = useHabitMutations().rule;
  const submit = (e: FormEvent) => {
    e.preventDefault();
    const parsed = habitRuleInputSchema.safeParse({
      target,
      weekdayMask: mask,
    });
    if (!parsed.success) {
      setError(parsed.error.issues[0].message);
      return;
    }
    mutation.mutate(
      { id: detail.habit.id, input: parsed.data },
      { onSuccess: onClose, onError: (e) => setError(e.message) },
    );
  };
  return (
    <HabitDialog title="Change target & schedule" onClose={onClose}>
      <form className="habit-form" onSubmit={submit}>
        <p className="field-hint">
          This change starts tomorrow. Saving again today replaces tomorrow’s
          pending change.
        </p>
        <label className="form-field">
          <span>
            {detail.habit.targetType === 'duration'
              ? 'Target minutes'
              : 'Target'}
          </span>
          <input
            type="number"
            min={1}
            disabled={detail.habit.targetType === 'boolean'}
            value={target}
            onChange={(e) => setTarget(Number(e.target.value))}
          />
        </label>
        <WeekdaySelector value={mask} onChange={setMask} />
        {error ? (
          <p role="alert" className="error-message">
            {error}
          </p>
        ) : null}
        <div className="form-actions">
          <button type="button" className="button" onClick={onClose}>
            Cancel
          </button>
          <button
            className="button button-primary"
            disabled={mutation.isPending}
          >
            Save for tomorrow
          </button>
        </div>
      </form>
    </HabitDialog>
  );
}
