import { open, save } from '@tauri-apps/plugin-dialog';
import { useMutation } from '@tanstack/react-query';
import { DatabaseBackup, ShieldCheck } from 'lucide-react';
import { useState } from 'react';
import {
  createManualBackup,
  validateBackup,
  type BackupResult,
} from '../../lib/ipc/backup';
import { IpcError } from '../../lib/ipc/errors';
import { useStorageInfoQuery } from './settingsApi';

const databaseFilter = [{ name: 'Zenfolio database', extensions: ['sqlite3'] }];

export function BackupPanel() {
  const storage = useStorageInfoQuery();
  const [result, setResult] = useState<BackupResult | null>(null);
  const create = useMutation({ mutationFn: createManualBackup });
  const validate = useMutation({ mutationFn: validateBackup });

  const createBackup = async () => {
    setResult(null);
    const selected = await save({
      title: 'Create Zenfolio backup',
      defaultPath: `zenfolio-backup-${new Date().toISOString().slice(0, 10)}.sqlite3`,
      filters: databaseFilter,
    });
    if (!selected) return;
    create.mutate(selected, { onSuccess: setResult });
  };

  const checkBackup = async () => {
    setResult(null);
    const selected = await open({
      title: 'Validate Zenfolio backup',
      multiple: false,
      directory: false,
      filters: databaseFilter,
    });
    if (!selected || Array.isArray(selected)) return;
    validate.mutate(selected, { onSuccess: setResult });
  };

  const mutationError = create.error ?? validate.error;
  const errorMessage =
    mutationError instanceof IpcError
      ? mutationError.message
      : 'The backup operation could not be completed.';

  return (
    <section className="settings-card">
      <div className="settings-card-heading">
        <div>
          <h2>Database backup</h2>
          <p>Create and independently verify a complete SQLite backup.</p>
        </div>
        <DatabaseBackup aria-hidden="true" />
      </div>
      <div className="path-panel">
        <span>Live database</span>
        <code>{storage.data?.databasePath ?? 'Loading…'}</code>
      </div>
      <p className="backup-note">
        Backup files are local and unencrypted. Keep another copy on a separate
        disk for protection from drive failure.
      </p>
      <div className="backup-actions">
        <button
          className="button button-primary"
          type="button"
          disabled={create.isPending || validate.isPending}
          onClick={() => void createBackup()}
        >
          Create backup
        </button>
        <button
          className="button"
          type="button"
          disabled={create.isPending || validate.isPending}
          onClick={() => void checkBackup()}
        >
          Validate backup
        </button>
      </div>
      {mutationError ? (
        <p className="form-message error-message" role="alert">
          {errorMessage}
        </p>
      ) : null}
      {result ? (
        <div className="backup-result" role="status">
          <ShieldCheck aria-hidden="true" />
          <div>
            <strong>Valid Zenfolio backup</strong>
            <span>
              Schema {result.schemaVersion} · {result.path}
            </span>
          </div>
        </div>
      ) : null}
    </section>
  );
}
