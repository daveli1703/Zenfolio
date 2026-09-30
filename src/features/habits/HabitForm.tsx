import { type FormEvent, useState } from 'react';
import { IpcError } from '../../lib/ipc/errors';
import { currentDateInTimezone } from '../../lib/dates';
import {
  createHabitInputSchema,
  habitProfileInputSchema,
  type CreateHabitInput,
  type HabitDetail,
} from '../../lib/ipc/habits';
import { useHabitMutations } from './habitsApi';
import { WeekdaySelector } from './WeekdaySelector';
import { HabitDialog } from './HabitDialog';
import { useSettingsQuery } from '../settings/settingsApi';

export function HabitForm({
  detail,
  onClose,
}: {
  detail: HabitDetail | null;
  onClose: () => void;
}) {
  const mutations = useHabitMutations();
  const settings = useSettingsQuery();
  const latest = detail?.rules.at(-1);
  const [draft, setDraft] = useState<CreateHabitInput>(() =>
    detail
      ? {
          name: detail.habit.name,
          description: detail.habit.description,
          targetType: detail.habit.targetType,
          unit: detail.habit.unit,
          color: detail.habit.color,
          startDate: detail.habit.startDate,
          target: latest?.target ?? 1,
          weekdayMask: latest?.weekdayMask ?? 127,
        }
      : {
          name: '',
          description: null,
          targetType: 'boolean',
          unit: null,
          color: '#17735a',
          startDate: settings.data
            ? currentDateInTimezone(settings.data.applicationTimezone)
            : '',
          target: 1,
          weekdayMask: 127,
        },
  );
  const [errors, setErrors] = useState<Record<string, string>>({});
  const [message, setMessage] = useState('');
  const submit = (event: FormEvent) => {
    event.preventDefault();
    setErrors({});
    setMessage('');
    const schema = detail ? habitProfileInputSchema : createHabitInputSchema;
    const parsed = schema.safeParse(draft);
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
        setMessage(
          'The habit could not be saved. Your changes are still here.',
        );
    };
    if (detail)
      mutations.profile.mutate(
        { id: detail.habit.id, input: parsed.data },
        { onSuccess: onClose, onError },
      );
    else
      mutations.create.mutate(parsed.data as CreateHabitInput, {
        onSuccess: onClose,
        onError,
      });
  };
  const locked = detail?.hasEntries ?? false;
  return (
    <HabitDialog
      title={detail ? 'Edit habit' : 'Create habit'}
      onClose={onClose}
    >
      <form className="habit-form" onSubmit={submit}>
        <Field label="Name" error={errors.name}>
          <input
            autoFocus
            value={draft.name}
            onChange={(e) => setDraft({ ...draft, name: e.target.value })}
          />
        </Field>
        <Field label="Description" error={errors.description}>
          <textarea
            rows={3}
            value={draft.description ?? ''}
            onChange={(e) =>
              setDraft({ ...draft, description: e.target.value || null })
            }
          />
        </Field>
        <div className="habit-form-grid">
          <Field label="Type" error={errors.targetType}>
            <select
              disabled={locked}
              value={draft.targetType}
              onChange={(e) => {
                const targetType = e.target
                  .value as CreateHabitInput['targetType'];
                setDraft({
                  ...draft,
                  targetType,
                  unit: targetType === 'count' ? draft.unit : null,
                  target: targetType === 'boolean' ? 1 : draft.target,
                });
              }}
            >
              <option value="boolean">Boolean</option>
              <option value="count">Count</option>
              <option value="duration">Duration (minutes)</option>
            </select>
          </Field>
          <Field label="Start date" error={errors.startDate}>
            <input
              type="date"
              disabled={locked}
              value={draft.startDate}
              onChange={(e) =>
                setDraft({ ...draft, startDate: e.target.value })
              }
            />
          </Field>
          {draft.targetType === 'count' ? (
            <Field label="Unit" error={errors.unit}>
              <input
                disabled={locked}
                value={draft.unit ?? ''}
                onChange={(e) =>
                  setDraft({ ...draft, unit: e.target.value || null })
                }
              />
            </Field>
          ) : null}
          <Field label="Color" error={errors.color}>
            <input
              type="color"
              value={draft.color}
              onChange={(e) => setDraft({ ...draft, color: e.target.value })}
            />
          </Field>
        </div>
        {!detail ? (
          <>
            <Field
              label={
                draft.targetType === 'duration' ? 'Target minutes' : 'Target'
              }
              error={errors.target}
            >
              <input
                type="number"
                min={1}
                disabled={draft.targetType === 'boolean'}
                value={draft.target}
                onChange={(e) =>
                  setDraft({ ...draft, target: Number(e.target.value) })
                }
              />
            </Field>
            <WeekdaySelector
              value={draft.weekdayMask}
              onChange={(weekdayMask) => setDraft({ ...draft, weekdayMask })}
            />
            {errors.weekdayMask ? (
              <span className="field-error">{errors.weekdayMask}</span>
            ) : null}
          </>
        ) : null}
        {locked ? (
          <p className="field-hint">
            Type, unit, and start date are locked because this habit has
            entries.
          </p>
        ) : null}
        {message ? (
          <p role="alert" className="form-message error-message">
            {message}
          </p>
        ) : null}
        <div className="form-actions">
          <button type="button" className="button" onClick={onClose}>
            Cancel
          </button>
          <button
            className="button button-primary"
            disabled={mutations.create.isPending || mutations.profile.isPending}
          >
            {detail ? 'Save profile' : 'Create habit'}
          </button>
        </div>
      </form>
    </HabitDialog>
  );
}
function Field({
  label,
  error,
  children,
}: {
  label: string;
  error?: string;
  children: React.ReactNode;
}) {
  return (
    <label className="form-field">
      <span>{label}</span>
      {children}
      {error ? <span className="field-error">{error}</span> : null}
    </label>
  );
}
