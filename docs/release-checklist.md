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
