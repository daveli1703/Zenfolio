# Agreed data decisions (for milestone 3 onward)

Milestone 3 introduces migration `0001_settings.sql` with only
`schema_migrations` and the singleton `app_settings` table. Milestone 4 adds
`0002_tasks.sql` for projects, tags, tasks, and task/tag relationships.
Milestone 5 adds `0003_habits.sql` for habits, effective-dated rules, and one
aggregate entry per habit/date. Later features continue through numbered,
checksummed, forward-only migrations.

## Storage and boundaries

- SQLite in platform-local application data, never the executable or repository.
- UUID v4 text IDs; UTC RFC 3339 audit timestamps.
- Validated date-only strings `YYYY-MM-DD` and months `YYYY-MM`.
- Wall-clock planner times are separate from timestamps.
- Persist an IANA application timezone; travel must not rewrite historical dates.
- Foreign keys, WAL, bounded busy timeout, synchronous FULL.
- Explicit transactions for multi-row changes and cross-row validation.
- Backup before schema upgrades; reject unknown newer database versions.

## Module records

- Settings singleton and migration history are implemented in Milestone 3.
- Projects, tags, tasks, and task/tag joins are implemented in Milestone 4.
  Project deletion clears task associations; tag deletion removes join rows.
  Task completion timestamps follow explicit status transitions. Tag names are
  unique after trimming and case-insensitive comparison.
- Habits own effective-dated target/schedule rules and one aggregate entry per
  habit/date. Monday is bit zero in the seven-bit weekday mask. The applicable
  rule is the latest rule effective on or before a date. Target and schedule
  edits replace tomorrow's pending rule without rewriting earlier rules.
- Independent goals with manual scaled-integer progress and explicit status.
- Accounts, typed transaction categories, transactions, monthly and category budgets.
- Single-day planner events with optional task links.

Money uses checked 64-bit minor-unit integers in Rust/SQLite and decimal integer
strings over IPC. VND is the initial currency. Lock currency once any financial
record exists. Never silently reinterpret balances or use float arithmetic.

Habit rules support daily/selected weekdays. Changes take effect tomorrow and
preserve earlier rules. Durations are whole minutes; counts are whole units.
Streaks count scheduled opportunities; incomplete today does not break a streak.
Past missing eligible days do. Completion rates exclude incomplete today and
return an empty state when no elapsed days are scheduled. Numeric heatmap
intensity is relative to that day's historical target. Archive takes effect
tomorrow and is the first inactive date. Type, unit, and start date lock after
the first entry.

Calculate balances, budget summaries, habit statistics, goal percentages,
dashboard values and planner task projections; do not persist duplicate derived
state. Keep habit rules and entries because they are historical facts.

## Recovery

Use SQLite online backup, not a raw live-file copy. Milestone 3 independently
validates manual and pre-migration backups through application identity,
migration history, integrity, and foreign-key checks. A failed startup preserves
the existing file and opens a non-destructive recovery screen. Backups are
unencrypted.

Live-database replacement, staged restore, interruption markers, automatic
retention controls, and exports remain deferred to Milestone 10.

## Deferred

Task recurrence, weekly habit quotas, automatic goal links, transfers,
multicurrency, bank connections, advanced calendar recurrence, encryption,
generic imports, and cloud synchronization remain out of scope for V1.
