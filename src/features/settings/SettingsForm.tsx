import { type FormEvent, useState } from 'react';
import { IpcError } from '../../lib/ipc/errors';
import {
  type UpdateSettingsInput,
  type AppSettings,
  updateSettingsInputSchema,
} from '../../lib/ipc/settings';
import { useSettingsQuery, useUpdateSettingsMutation } from './settingsApi';

export function SettingsForm() {
  const settings = useSettingsQuery();

  if (settings.isPending) {
    return <div className="settings-card">Loading local preferences…</div>;
  }
  if (settings.isError) {
    return (
      <div className="settings-card error-message">
        Preferences could not be loaded.
      </div>
    );
  }

  return (
    <SettingsFormContent
      key={settings.data.updatedAt}
      settings={settings.data}
    />
  );
}

function SettingsFormContent({ settings }: { settings: AppSettings }) {
  const update = useUpdateSettingsMutation();
  const [draft, setDraft] = useState<UpdateSettingsInput>({
    currencyCode: settings.currencyCode,
    currencyExponent: settings.currencyExponent,
    applicationTimezone: settings.applicationTimezone,
    dateFormat: settings.dateFormat,
    timeFormat: settings.timeFormat,
    firstWeekday: settings.firstWeekday,
    theme: settings.theme,
  });
  const [message, setMessage] = useState('');

  const submit = (event: FormEvent) => {
    event.preventDefault();
    setMessage('');
    const parsed = updateSettingsInputSchema.safeParse(draft);
    if (!parsed.success) {
      setMessage('Check the preference values and try again.');
      return;
    }
    update.mutate(parsed.data, {
      onSuccess: () => setMessage('Preferences saved.'),
    });
  };

  const error = update.error;
  const errorMessage =
    error instanceof IpcError
      ? error.message
      : 'Preferences could not be saved.';

  return (
    <form className="settings-card settings-form" onSubmit={submit}>
      <div className="settings-card-heading">
        <div>
          <h2>Preferences</h2>
          <p>Stored locally in your Zenfolio database.</p>
        </div>
      </div>

      <div className="form-grid">
        <label>
          <span>Application timezone</span>
          <input
            value={draft.applicationTimezone}
            onChange={(event) =>
              setDraft({ ...draft, applicationTimezone: event.target.value })
            }
          />
        </label>
        <label>
          <span>Date format</span>
          <select
            value={draft.dateFormat}
            onChange={(event) =>
              setDraft({
                ...draft,
                dateFormat: event.target
                  .value as UpdateSettingsInput['dateFormat'],
              })
            }
          >
            <option value="DD/MM/YYYY">DD/MM/YYYY</option>
            <option value="MM/DD/YYYY">MM/DD/YYYY</option>
            <option value="YYYY-MM-DD">YYYY-MM-DD</option>
          </select>
        </label>
        <label>
          <span>Time format</span>
          <select
            value={draft.timeFormat}
            onChange={(event) =>
              setDraft({
                ...draft,
                timeFormat: event.target
                  .value as UpdateSettingsInput['timeFormat'],
              })
            }
          >
            <option value="24h">24 hour</option>
            <option value="12h">12 hour</option>
          </select>
        </label>
        <label>
          <span>First weekday</span>
          <select
            value={draft.firstWeekday}
            onChange={(event) =>
              setDraft({
                ...draft,
                firstWeekday: Number(event.target.value) as 1 | 7,
              })
            }
          >
            <option value={1}>Monday</option>
            <option value={7}>Sunday</option>
          </select>
        </label>
        <label>
          <span>Currency</span>
          <input
            maxLength={3}
            value={draft.currencyCode}
            onChange={(event) =>
              setDraft({
                ...draft,
                currencyCode: event.target.value.toUpperCase(),
              })
            }
          />
        </label>
        <label>
          <span>Currency decimal places</span>
          <input
            type="number"
            min={0}
            max={3}
            value={draft.currencyExponent}
            onChange={(event) =>
              setDraft({
                ...draft,
                currencyExponent: Number(event.target.value),
              })
            }
          />
        </label>
      </div>

      {update.isError ? (
        <p className="form-message error-message" role="alert">
          {errorMessage}
        </p>
      ) : null}
      {message ? (
        <p className="form-message" role="status">
          {message}
        </p>
      ) : null}
      <div className="form-actions">
        <button className="button button-primary" disabled={update.isPending}>
          {update.isPending ? 'Saving…' : 'Save preferences'}
        </button>
      </div>
    </form>
  );
}
