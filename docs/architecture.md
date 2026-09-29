# Architecture and milestone boundaries

## Runtime

React renders the webview. Typed Tauri commands will call Rust application
services. Services own validation and transactions; feature repositories own
parameterized SQL. Pure Rust functions own money, habit, and goal calculations.
UI code owns presentation, calendar layout, and form state.

The foundation has no IPC commands, filesystem grants, network plugins, or
database. Production CSP permits bundled content and Tauri IPC only. Development
CSP additionally permits the loopback Vite server and its hot-reload socket.
No remote fonts, images, scripts, telemetry, or updater are included.

## Dependencies added when needed

Milestone 2 adds hash routing and accessible UI primitives. Milestone 3 adds
rusqlite with bundled SQLite, typed command inputs/results, Zod, and TanStack
Query. Recharts arrives with Budget. Avoid installing unused feature libraries.

One Rust-owned SQLite connection will serialize access off the UI thread.
The application will enforce a single instance when persistence is introduced.
No generic repository, global event bus, or duplicated frontend persistence.

The permanent application identifier is `app.zenfolio.local`. Preserve
it across display-name changes. Development and release data directories must
be separated before persistence is enabled.

## Milestones

1. Foundation and build validation.
2. Shell, seven routes, themes, accessible shared controls.
3. Persistence, migrations, settings, backup and recoverable restore.
4. Tasks, projects, tags, filters, completion and reopening.
5. Habits, dated rules, entries, heatmap and statistics.
6. Manual goals and explicit status transitions.
7. Budget accounts, transactions, categories and monthly limits.
8. Dashboard derived from existing module data.
9. Day/week/month planner and simple scheduled activities.
10. Complete settings and data export/backup UX.
11. Cross-module testing, accessibility and performance.
12. Windows installer, upgrade and offline release validation.

Each milestone must leave a working app and stop for review. Do not pre-create
empty feature implementations or display invented user data.
