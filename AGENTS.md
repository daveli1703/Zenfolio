# Zenfolio Development Instructions

## Project

- Product name: Zenfolio.
- Zenfolio is a personal, local-first desktop productivity application.
- Windows is the primary platform. Keep the architecture reasonably portable to
  macOS and Linux.
- The permanent Tauri application identifier is `app.zenfolio.local`. Do not
  change it after user data exists.
- The existing GitHub repository and Git history are already configured. Do not
  reinitialize Git, create a replacement repository, rewrite history, or
  force-push.

## Technology

- Tauri 2
- React
- TypeScript
- Vite
- Tailwind CSS
- Rust
- SQLite through `rusqlite`
- Explicit, parameterized SQL
- React Router
- TanStack Query when persistence and data queries are introduced

Use the existing package managers and committed lockfiles. Add dependencies only
when they solve a clear need in the current milestone.

## Architecture

- React handles presentation, interaction, routing, accessible UI behavior, and
  frontend formatting.
- Rust owns authoritative business validation, persistence rules, financial
  calculations, habit statistics, and other important domain logic.
- Tauri IPC is the boundary between the frontend and Rust. Commands should accept
  structured inputs and return typed results.
- Database repositories contain SQL.
- Services coordinate business logic, use cases, and transaction boundaries.
- Keep important business logic out of React components.
- Do not expose arbitrary SQL execution to the frontend.
- Keep the app shell, router, and providers under `src/app`.
- Keep reusable UI controls under `src/components/ui`.
- Keep feature-specific pages and components under `src/features`.
- Keep shared styling and design tokens under `src/styles`.

## Development Principles

- Follow the approved milestone roadmap.
- Implement only the current milestone and stop for review when it is complete.
- Do not begin future milestones early or add placeholder infrastructure for
  unstarted features.
- Prefer simple, explicit solutions over generalized frameworks.
- Avoid premature abstractions, generic repository frameworks, dependency
  injection containers, event buses, and plugin architectures.
- Do not add dependencies unless they solve a clear current need.
- Inspect existing conventions before introducing new patterns.
- Preserve user changes and avoid unrelated refactors.
- Keep the repository, lockfiles, documentation, and application in a working
  state at every milestone boundary.

## Privacy and Local-First Requirements

All personal application data must remain local. Do not introduce any of the
following without explicit user permission:

- Cloud services
- Authentication or account systems
- Telemetry
- Analytics
- Advertising
- External databases
- Bank integrations
- Remote APIs
- AI features

Do not log sensitive user content, including notes, transaction descriptions,
financial details, or other private record contents. Runtime operation must not
depend on a remote service.

## Database Rules

- Use SQLite through `rusqlite` and explicit, parameterized SQL.
- Enable foreign keys on every database connection.
- Use numbered, forward-only migrations with appropriate validation.
- Use transactions for multi-row mutations and cross-row invariants.
- Never silently reset, replace, or delete the database after an error.
- Preserve user data when migrations fail and provide a recoverable error path.
- Calculate derived values instead of redundantly storing them unless persistence
  is required for correctness or historical meaning.
- Keep date-only values separate from UTC timestamps and local wall-clock times.
- Preserve historical habit rules with effective dates.
- Represent money with exact integer or scaled-integer values. Never use
  floating-point arithmetic as the authoritative representation.
- Use stable IDs, explicit foreign-key behavior, and indexes justified by actual
  queries.
- Keep live databases and backups outside the repository.

## Quality

Before completing a milestone, run all relevant checks:

- Formatting and formatting verification
- ESLint
- TypeScript type checking
- Frontend tests
- Frontend production build
- `cargo fmt`
- `cargo check`
- `cargo clippy`
- Rust tests
- Tauri build and runtime validation where relevant

Fix legitimate failures instead of suppressing them. Add meaningful tests for
business rules and failure cases; do not add tests solely to increase coverage.
Every milestone must leave Zenfolio in a working state.

## Milestone Completion

At the end of each milestone:

- Summarize files created and modified.
- Summarize dependencies added or removed.
- Report validation commands and results.
- Document important architectural or product decisions.
- Document known limitations and intentionally deferred work.
- Stop before starting the next milestone.

Do not commit or push milestone work unless the user requests it. Do not begin
Milestone 3 until the user explicitly approves and requests it.
