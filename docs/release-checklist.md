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
a responsive Personal Productivity window while port 1420 had no listener.
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
