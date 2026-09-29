import { useQuery } from '@tanstack/react-query';
import { DatabaseZap } from 'lucide-react';
import { type ReactNode } from 'react';
import { getStartupStatus } from '../lib/ipc/startup';

export function StartupBoundary({ children }: { children: ReactNode }) {
  const startup = useQuery({
    queryKey: ['startup-status'],
    queryFn: getStartupStatus,
    staleTime: Number.POSITIVE_INFINITY,
  });

  if (startup.isPending) {
    return (
      <main className="startup-screen" aria-busy="true">
        <p>Opening your local workspace…</p>
      </main>
    );
  }

  if (startup.isError || startup.data.state === 'recovery') {
    const message = startup.isError
      ? 'Zenfolio could not check the local database.'
      : (startup.data.error?.message ??
        'The local database could not be opened safely.');
    const path = startup.isError ? undefined : startup.data.databasePath;
    return (
      <main className="startup-screen">
        <section className="recovery-card" aria-labelledby="recovery-title">
          <div className="empty-state-icon">
            <DatabaseZap aria-hidden="true" />
          </div>
          <p className="page-eyebrow">Recovery required</p>
          <h1 id="recovery-title">Your database was preserved</h1>
          <p>{message}</p>
          <p>
            Zenfolio did not delete, reset, replace, or recreate the existing
            file.
          </p>
          {path ? (
            <div className="path-panel">
              <span>Database location</span>
              <code>{path}</code>
            </div>
          ) : null}
          <p className="recovery-help">
            Close Zenfolio and keep this file unchanged while diagnosing the
            problem.
          </p>
        </section>
      </main>
    );
  }

  return children;
}
