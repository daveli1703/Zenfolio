# Validation record

## Milestone 1

- [x] Inspect initial repository: empty, no commits.
- [x] Confirm Node/npm available.
- [x] Confirm WebView2 installed.
- [x] Confirm Rust stable, rustfmt and Clippy available.
- [x] Confirm MSVC C++ Build Tools and Windows SDK available.
- [x] Resolve and retain npm and Cargo lockfiles.
- [x] Pass frontend formatting, lint, typecheck, test and build.
- [x] Pass Rust format, check, Clippy and test harness (no domain tests yet).
- [x] Launch the standalone debug desktop window with the Vite server stopped.
- [x] Inspect the welcome screen at desktop size (browser preview, 1200 x 800).

Unchecked items are not claimed as passing. Browser rendering alone does not
validate Tauri IPC or native packaging. Foundation tests only validate the entry
screen; business-logic tests arrive with the corresponding modules.

Validated on 2026-09-29: Node 24.15.0, npm 11.12.1, Rust 1.98.1,
Visual Studio Build Tools 2022 with C++ and Windows SDK, WebView2 154.0.4258.37.
`npm run check` passed (one frontend smoke test). Rust formatting, check,
Clippy with warnings denied, and the test harness passed. The Rust harness has
zero tests until domain logic is introduced. `npm run desktop:build:debug --
--offline` succeeded with locked dependencies. The resulting executable created
a responsive foundation window while port 1420 had no listener.
Visual rendering was inspected in the browser at the intended desktop size;
native window content was not screenshot-verified. Optimized release packaging
and clean-machine installation remain later release checks.

## Later release gate

- Backup/restore and interrupted-restore recovery pass on real SQLite files.
- Migration fixtures cover all previously released schemas.
- Exact money, historical habit rules, date boundaries and goal calculations pass.
- All seven modules work after restart and without network access.
- Keyboard navigation, focus handling, 200% text zoom and both themes work.
- Fresh Windows installation and upgrade preserve expected data locations.
- Uninstall preserves personal data.
- Installer includes offline WebView2 installation support.

## Milestone 2

- [x] Rename visible product identity to Zenfolio.
- [x] Replace the placeholder application icon with the Zenfolio Z mark.
- [x] Set the permanent Tauri identifier to `app.zenfolio.local`.
- [x] Add the persistent sidebar and seven hash-routed placeholder pages.
- [x] Confirm every route and active state in the packaged desktop app.
- [x] Confirm direct hash routing through automated frontend tests.
- [x] Confirm system, light, and dark themes in the packaged desktop app.
- [x] Confirm keyboard focus visibility and keyboard route activation.
- [x] Inspect layouts with 1200 x 800 and 1024 x 768 client areas.
- [x] Pass Prettier, ESLint, TypeScript, frontend tests, and production build.
- [x] Pass Cargo formatting, check, Clippy with warnings denied, and tests.
- [x] Build and launch the packaged debug application without a dev server.
- [x] Confirm the runtime has no remote service requirement.

Validated on 2026-09-29. The frontend suite contains three shell tests covering
navigation structure, hash routing/active state, and theme selection. All seven
routes were also opened in the desktop application. The system theme resolved to
the operating system's current light preference; explicit dark and light modes
were inspected separately. Sidebar focus was visibly outlined and a route was
activated with Shift+Tab and Enter. Window screenshots measured 1202 x 831 and
1026 x 799 including Windows borders/title bars, corresponding to the configured
1200 x 800 and 1024 x 768 client areas. No database, feature data, remote API,
analytics, or telemetry was introduced.

## Milestone 3

- [x] Store the live SQLite database outside the repository under the stable
      `app.zenfolio.local` application-data directory.
- [x] Keep development, production, and test databases in separate directories.
- [x] Own one serialized `rusqlite` connection in Rust and run database work on
      Tauri's blocking thread pool.
- [x] Enable foreign keys, WAL mode, `synchronous=FULL`, and a bounded busy
      timeout.
- [x] Apply numbered, checksummed SQL migrations transactionally.
- [x] Reject corrupt, foreign, and newer-schema databases without replacing or
      resetting them.
- [x] Create and validate an online backup before migrating an existing database.
- [x] Persist and validate application settings through typed Tauri IPC.
- [x] Integrate persisted settings and theme state through TanStack Query.
- [x] Prevent concurrent Zenfolio desktop instances.
- [x] Create and independently validate manual SQLite online backups.
- [x] Present a non-destructive recovery screen when the live database cannot be
      opened safely.
- [x] Pass frontend formatting, lint, typecheck, tests, and production build.
- [x] Pass Rust formatting, check, Clippy with warnings denied, and tests.
- [x] Build and launch the packaged debug application without a dev server.
- [x] Confirm first launch, restart persistence, environment separation,
      single-instance behavior, backup creation, backup validation, recovery paths,
      and offline startup.

Validated on 2026-09-29. The frontend suite contains five tests covering the
application shell and IPC serialization/error contracts. The Rust suite contains
18 tests covering settings validation and persistence, database configuration,
migration checksums and rollback, incompatible/corrupt database preservation,
pre-migration backup failure, online backup round trips, and IPC contracts.
Manual failure checks used isolated database fixtures and confirmed byte-identical
database hashes before and after corrupt, newer-schema, and blocked-backup startup
failures. The development and production executables created distinct databases,
and offline startup showed no TCP connections or Vite listener. Live database
restore, replacement, recovery markers, retention controls, and export remain
deferred to Milestone 10.

## Milestone 4

- [x] Add migration `0002_tasks.sql` without changing the applied settings
      migration.
- [x] Add constrained projects, tags, tasks, and task/tag relationship tables
      with the required foreign-key behavior and indexes.
- [x] Implement explicit repositories, authoritative services, typed commands,
      and matching TypeScript/Zod contracts.
- [x] Support task create, edit, delete, completion, reopening, status and
      priority changes, project assignment, and tag assignment.
- [x] Support project create, edit, archive, unarchive, and delete while
      preserving tasks.
- [x] Support case-insensitive unique tag create, edit, and delete while
      preserving tasks.
- [x] Support search, combined status/priority/project/tag filters, and
      allowlisted due-date/created-date/priority sorting.
- [x] Replace the Tasks placeholder with a keyboard-friendly list, filter bar,
      task drawer, taxonomy manager, and useful loading/error/empty states.
- [x] Preserve task form drafts and display safe errors after rejected writes.
- [x] Confirm a schema-1 database is backed up and upgraded to schema 2.
- [x] Pass frontend formatting, lint, typecheck, tests, and production build.
- [x] Pass Rust formatting, check, Clippy with warnings denied, and tests.
- [x] Build and launch the packaged debug application without a dev server.

Validated on 2026-09-30. The frontend suite contains twelve tests, including task
creation, editing, completion, reopening, project/tag assignment, search,
combined filters, sorting, empty state behavior, failed-write draft retention,
and IPC payload parsing. The Rust suite contains 34 tests, including an
end-to-end task service workflow on a real temporary SQLite file, migration from
the Milestone 3 schema, restart persistence, transaction rollback, deletion
foreign-key behavior, validation, sorting allowlists, and DTO contracts. The
packaged application upgraded the development database to schema 2 after
creating an independently valid schema-1 migration backup; the persisted dark
theme remained intact. No future-module or recurrence tables were introduced.

## Milestone 5

- [x] Add migration `0003_habits.sql` without modifying migrations 1 or 2.
- [x] Add habits, dated rules, and unique daily aggregate entries.
- [x] Support boolean, count, and whole-minute duration targets.
- [x] Support daily and selected-weekday schedules.
- [x] Preserve historical targets and schedules through tomorrow-effective rules.
- [x] Replace an already pending tomorrow rule instead of duplicating it.
- [x] Reject future, pre-start, unscheduled, and post-archive entries.
- [x] Support eligible backdated entries and tomorrow-effective archive boundaries.
- [x] Calculate streaks and completion rates from scheduled opportunities.
- [x] Render a leap-year-safe, first-weekday-aware yearly heatmap.
- [x] Provide mouse and roving-keyboard heatmap interaction.
- [x] Preserve task and settings records through the schema 2 to 3 migration.
- [x] Complete final automated and packaged desktop validation.

Validated on 2026-09-30. The frontend suite contains twenty tests covering
typed habit contracts, timezone-safe dates, leap-year and first-weekday calendar
grids, creation and entry flows, selected weekdays, rule changes, and roving
heatmap keyboard navigation. The Rust suite contains 46 tests covering migration
2 to 3, target types, historical rules, pending-rule replacement, eligibility,
archive boundaries, scheduled-opportunity streaks, completion denominators,
intensity thresholds, persistence, cascades, and existing recovery behavior.
The offline debug build succeeded and launched a Zenfolio window with no TCP
connections. Full visual interaction remains part of the user review because
native-window automation was unavailable in this environment.
