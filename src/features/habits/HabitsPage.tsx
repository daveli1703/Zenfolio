import {
  Archive,
  CalendarDays,
  Edit3,
  Leaf,
  Plus,
  Settings2,
  Trash2,
} from 'lucide-react';
import { useEffect, useState, type CSSProperties } from 'react';
import { useQueryClient } from '@tanstack/react-query';
import { PageHeader } from '../../components/ui/PageHeader';
import { currentDateInTimezone } from '../../lib/dates';
import type { HabitDay, Habit } from '../../lib/ipc/habits';
import { getHabitDeleteImpact } from '../../lib/ipc/habits';
import { useSettingsQuery } from '../settings/settingsApi';
import { HabitEntryForm } from './HabitEntryForm';
import { HabitForm } from './HabitForm';
import { HabitHeatmap } from './HabitHeatmap';
import { HabitRuleForm } from './HabitRuleForm';
import {
  habitsKey,
  useHabitDetailQuery,
  useHabitMutations,
  useHabitsQuery,
  useHabitYearQuery,
  useTodayHabitsQuery,
} from './habitsApi';

export function HabitsPage() {
  const habits = useHabitsQuery(),
    today = useTodayHabitsQuery(),
    settings = useSettingsQuery();
  const [selected, setSelected] = useState<string | null>(null),
    [creating, setCreating] = useState(false),
    [editing, setEditing] = useState(false),
    [changingRule, setChangingRule] = useState(false);
  const [entry, setEntry] = useState<{ habit: Habit; day: HabitDay } | null>(
      null,
    ),
    [year, setYear] = useState(new Date().getFullYear());
  const selectedId = selected ?? habits.data?.[0]?.id ?? null;
  const detail = useHabitDetailQuery(selectedId),
    yearData = useHabitYearQuery(selectedId, year),
    mutations = useHabitMutations(),
    queryClient = useQueryClient();
  useEffect(() => {
    const refresh = () => {
      void queryClient.invalidateQueries({ queryKey: habitsKey });
    };
    window.addEventListener('focus', refresh);
    const timer = window.setInterval(refresh, 60_000);
    return () => {
      window.removeEventListener('focus', refresh);
      window.clearInterval(timer);
    };
  }, [queryClient]);
  const active = habits.data?.filter((h) => !h.archiveDate) ?? [],
    archived = habits.data?.filter((h) => h.archiveDate) ?? [];
  const loading = habits.isPending || today.isPending || settings.isPending;
  const localToday = settings.data
    ? currentDateInTimezone(settings.data.applicationTimezone)
    : '';
  const remove = async () => {
    if (!selectedId) return;
    const impact = await getHabitDeleteImpact(selectedId);
    if (
      window.confirm(
        `Delete this habit and permanently remove ${impact.ruleCount} rule${impact.ruleCount === 1 ? '' : 's'} and ${impact.entryCount} entr${impact.entryCount === 1 ? 'y' : 'ies'}?`,
      )
    )
      mutations.remove.mutate(selectedId, {
        onSuccess: () => setSelected(null),
      });
  };
  return (
    <div className="habits-page">
      <div className="page-title-row">
        <PageHeader
          eyebrow="Consistency"
          title="Habits"
          description="Build routines with targets that preserve their history."
        />
        <button
          className="button button-primary"
          disabled={settings.isPending}
          onClick={() => setCreating(true)}
        >
          <Plus aria-hidden="true" />
          New habit
        </button>
      </div>
      {loading ? <State title="Loading habits…" /> : null}
      {habits.isError || today.isError ? (
        <State title="Habits could not be loaded" />
      ) : null}
      {!loading && !habits.isError ? (
        <>
          <section className="today-habits" aria-labelledby="today-heading">
            <div className="section-heading">
              <div>
                <span className="eyebrow">Today</span>
                <h2 id="today-heading">Daily progress</h2>
              </div>
              <CalendarDays aria-hidden="true" />
            </div>
            {today.data?.length ? (
              today.data.map((item) => (
                <button
                  key={item.habit.id}
                  className="today-habit"
                  onClick={() =>
                    item.day.state === 'eligible' && setEntry(item)
                  }
                  disabled={item.day.state !== 'eligible'}
                >
                  <span
                    className={`habit-check level-${item.day.intensity}`}
                    style={
                      { '--habit-color': item.habit.color } as CSSProperties
                    }
                  >
                    {item.day.completed ? '✓' : ''}
                  </span>
                  <span>
                    <strong>{item.habit.name}</strong>
                    <small>
                      {item.day.state === 'unscheduled'
                        ? 'Not scheduled today'
                        : `${item.day.value} / ${item.day.target} ${item.habit.unit ?? (item.habit.targetType === 'duration' ? 'minutes' : '')}`}
                    </small>
                  </span>
                </button>
              ))
            ) : (
              <p className="empty-copy">
                Create a habit to begin tracking today.
              </p>
            )}
          </section>
          <div className="habit-workspace">
            <aside className="habit-list" aria-label="Habits">
              <h2>Active</h2>
              {active.map((h) => (
                <button
                  className={selectedId === h.id ? 'selected' : ''}
                  key={h.id}
                  onClick={() => setSelected(h.id)}
                >
                  <span style={{ background: h.color }} />
                  <span>
                    {h.name}
                    <small>
                      {h.startDate > localToday ? 'Starts later' : 'Active'}
                    </small>
                  </span>
                </button>
              ))}
              {archived.length ? (
                <>
                  <h2>Archived</h2>
                  {archived.map((h) => (
                    <button
                      className={selectedId === h.id ? 'selected' : ''}
                      key={h.id}
                      onClick={() => setSelected(h.id)}
                    >
                      <span style={{ background: h.color }} />
                      <span>
                        {h.name}
                        <small>Archived</small>
                      </span>
                    </button>
                  ))}
                </>
              ) : null}
            </aside>
            <main className="habit-detail">
              {selectedId && detail.data && yearData.data && settings.data ? (
                <>
                  <div className="habit-detail-heading">
                    <div>
                      <span className="eyebrow">
                        {detail.data.habit.archiveDate
                          ? 'Archived'
                          : 'Habit detail'}
                      </span>
                      <h2>{detail.data.habit.name}</h2>
                      <p>{detail.data.habit.description || 'No description'}</p>
                    </div>
                    <div className="habit-actions">
                      <button
                        className="button"
                        onClick={() => setEditing(true)}
                      >
                        <Edit3 aria-hidden="true" />
                        Edit
                      </button>
                      {!detail.data.habit.archiveDate ? (
                        <>
                          <button
                            className="button"
                            onClick={() => setChangingRule(true)}
                          >
                            <Settings2 aria-hidden="true" />
                            Target &amp; schedule
                          </button>
                          <button
                            className="icon-button"
                            aria-label="Archive habit"
                            onClick={() => mutations.archive.mutate(selectedId)}
                          >
                            <Archive aria-hidden="true" />
                          </button>
                        </>
                      ) : null}
                      <button
                        className="icon-button danger"
                        aria-label="Delete habit"
                        onClick={() => void remove()}
                      >
                        <Trash2 aria-hidden="true" />
                      </button>
                    </div>
                  </div>
                  <div className="habit-stats">
                    <Stat
                      label="Current streak"
                      value={`${yearData.data.statistics.currentStreak}`}
                    />
                    <Stat
                      label="Longest streak"
                      value={`${yearData.data.statistics.longestStreak}`}
                    />
                    <Stat
                      label="Completions"
                      value={`${yearData.data.statistics.completedDays}`}
                    />
                    <Stat
                      label="Completion rate"
                      value={
                        yearData.data.statistics.completionRate === null
                          ? 'No scheduled days'
                          : `${Math.round(yearData.data.statistics.completionRate * 100)}%`
                      }
                    />
                  </div>
                  <p className="rate-caption">
                    Completion through yesterday, plus today if completed.
                  </p>
                  <div className="heatmap-heading">
                    <h3>Yearly activity</h3>
                    <select
                      aria-label="Heatmap year"
                      value={year}
                      onChange={(e) => setYear(Number(e.target.value))}
                    >
                      {[year - 2, year - 1, year, year + 1]
                        .filter((v, i, a) => a.indexOf(v) === i)
                        .sort()
                        .map((value) => (
                          <option key={value}>{value}</option>
                        ))}
                    </select>
                  </div>
                  <HabitHeatmap
                    habit={yearData.data.habit}
                    days={yearData.data.days}
                    settings={settings.data}
                    onActivate={(day) =>
                      setEntry({ habit: yearData.data.habit, day })
                    }
                  />
                  <div className="rule-history">
                    <h3>Rule history</h3>
                    {detail.data.rules.map((rule) => (
                      <p key={rule.id}>
                        <strong>{rule.effectiveDate}</strong> · target{' '}
                        {rule.target} ·{' '}
                        {rule.weekdayMask === 127
                          ? 'daily'
                          : weekdayText(rule.weekdayMask)}
                      </p>
                    ))}
                  </div>
                </>
              ) : (
                <State
                  title={
                    selectedId
                      ? 'Loading habit details…'
                      : 'Choose or create a habit'
                  }
                />
              )}
            </main>
          </div>
        </>
      ) : null}
      {creating ? (
        <HabitForm detail={null} onClose={() => setCreating(false)} />
      ) : null}
      {editing && detail.data ? (
        <HabitForm detail={detail.data} onClose={() => setEditing(false)} />
      ) : null}
      {changingRule && detail.data ? (
        <HabitRuleForm
          detail={detail.data}
          onClose={() => setChangingRule(false)}
        />
      ) : null}
      {entry ? (
        <HabitEntryForm
          habit={entry.habit}
          day={entry.day}
          onClose={() => setEntry(null)}
        />
      ) : null}
    </div>
  );
}
function State({ title }: { title: string }) {
  return (
    <section className="habit-state">
      <Leaf aria-hidden="true" />
      <h2>{title}</h2>
    </section>
  );
}
function Stat({ label, value }: { label: string; value: string }) {
  return (
    <div>
      <span>{label}</span>
      <strong>{value}</strong>
    </div>
  );
}
function weekdayText(mask: number) {
  return ['Mon', 'Tue', 'Wed', 'Thu', 'Fri', 'Sat', 'Sun']
    .filter((_, i) => (mask & (1 << i)) !== 0)
    .join(', ');
}
