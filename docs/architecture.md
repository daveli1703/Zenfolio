# Architecture and milestone boundaries

## Runtime

React renders the webview. Typed Tauri commands will call Rust application
services. Services own validation and transactions; feature repositories own
parameterized SQL. Pure Rust functions own money, habit, and goal calculations.
UI code owns presentation, calendar layout, and form state.

Milestone 3 adds typed settings, storage, and backup IPC commands. A recovery
boundary prevents normal screens from opening when the live database cannot be
validated. Dialog permissions are limited to selecting manual backup files.
Production CSP permits bundled content and Tauri IPC only. Development CSP
additionally permits the loopback Vite server and its hot-reload socket. No
remote fonts, images, scripts, telemetry, or updater are included.

Milestone 4 follows the same command → service → repository boundary for tasks,
projects, and tags. Services validate references and status transitions;
repositories own explicit SQL and transaction boundaries. Task list filters live
in URL search parameters while TanStack Query owns fetched records and confirmed
mutation refreshes.

## Dependencies added when needed

Milestone 2 adds hash routing and accessible UI primitives. Milestone 3 adds
rusqlite with bundled SQLite, typed command inputs/results, Zod, TanStack Query,
native backup dialogs, and single-instance protection. Recharts arrives with
Budget. Avoid installing unused feature libraries.

Milestone 4 adds only `uuid` for Rust-generated UUID v4 entity identifiers.

One Rust-owned SQLite connection serializes access through a mutex. Commands run
database work on Tauri's blocking pool. The application enforces a single
instance before opening persistence. No generic repository, global event bus, or
duplicated frontend persistence is used.

The permanent application identifier is `app.zenfolio.local`. Preserve it across
display-name changes. Tauri's identifier-scoped local data directory contains a
`development` database for debug builds and a `production` database for release
builds. Tests always receive temporary directories.

## Milestones

1. Foundation and build validation.
2. Shell, seven routes, themes, accessible shared controls.
3. Persistence, migrations, settings, validated backup and recovery mode.
4. Tasks, projects, tags, filters, completion and reopening.
5. Habits, dated rules, entries, heatmap and statistics.
6. Manual goals and explicit status transitions.
7. Budget accounts, transactions, categories and monthly limits.
8. Dashboard derived from existing module data.
9. Day/week/month planner and simple scheduled activities.
10. Complete settings, restore, retention, and data export/backup UX.
11. Cross-module testing, accessibility and performance.
12. Windows installer, upgrade and offline release validation.

Each milestone must leave a working app and stop for review. Do not pre-create
empty feature implementations or display invented user data.
