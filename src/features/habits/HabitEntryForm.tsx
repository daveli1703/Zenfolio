import { type FormEvent, useState } from 'react';
import type { Habit, HabitDay } from '../../lib/ipc/habits';
import { habitEntryInputSchema } from '../../lib/ipc/habits';
import { useHabitMutations } from './habitsApi';
import { HabitDialog } from './HabitDialog';
export function HabitEntryForm({
  habit,
  day,
  onClose,
}: {
  habit: Habit;
  day: HabitDay;
  onClose: () => void;
}) {
  const [value, setValue] = useState(day.value);
  const [notes, setNotes] = useState(day.notes ?? '');
  const [error, setError] = useState('');
  const mutation = useHabitMutations().entry;
  const submit = (e: FormEvent) => {
    e.preventDefault();
    const parsed = habitEntryInputSchema.safeParse({
      habitId: habit.id,
      date: day.date,
      value,
      notes: notes || null,
    });
    if (!parsed.success) {
      setError(parsed.error.issues[0].message);
      return;
    }
    mutation.mutate(parsed.data, {
      onSuccess: onClose,
      onError: (e) => setError(e.message),
    });
  };
  return (
    <HabitDialog title={`Log ${habit.name}`} onClose={onClose}>
      <form className="habit-form" onSubmit={submit}>
        <p className="field-hint">
          {day.date} · Target {day.target}{' '}
          {habit.unit ?? (habit.targetType === 'duration' ? 'minutes' : '')}
        </p>
        {habit.targetType === 'boolean' ? (
          <label className="boolean-entry">
            <input
              autoFocus
              type="checkbox"
              checked={value === 1}
              onChange={(e) => setValue(e.target.checked ? 1 : 0)}
            />
            <span>Completed</span>
          </label>
        ) : (
          <label className="form-field">
            <span>
              {habit.targetType === 'duration'
                ? 'Minutes'
                : (habit.unit ?? 'Value')}
            </span>
            <input
              autoFocus
              type="number"
              min={0}
              value={value}
              onChange={(e) => setValue(Number(e.target.value))}
            />
          </label>
        )}
        <label className="form-field">
          <span>Notes</span>
          <textarea
            rows={3}
            value={notes}
            onChange={(e) => setNotes(e.target.value)}
          />
        </label>
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
            Save entry
          </button>
        </div>
      </form>
    </HabitDialog>
  );
}
