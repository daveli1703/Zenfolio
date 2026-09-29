# Zenfolio

A Windows-first, local-only desktop workspace. Built with Tauri 2, React,
TypeScript, Vite, and Tailwind CSS. No account, backend, analytics, or runtime
network service is required.

## Current scope

Milestones 1 through 3 establish the project, navigable application shell, and
local persistence foundation. Settings are stored in SQLite and the Settings
screen can create and validate standalone database backups. Feature screens are
intentional placeholders without task or finance functionality. Stop for review
between milestones.

## Frontend development

Use Node.js 24 LTS and npm. From the project directory:

```powershell
npm ci
npm run dev
```

Open `http://127.0.0.1:1420`. The development server binds only to loopback.

```powershell
npm run check
```

This runs formatting, linting, TypeScript, the frontend test runner, and a
production frontend build. `npm run format` applies formatting.

## Desktop development

Windows prerequisites:

- Rust stable with the MSVC target, rustfmt, and Clippy.
- Visual Studio Build Tools with Desktop development with C++ and a Windows SDK.
- Microsoft Edge WebView2 Runtime.

See the [official Tauri prerequisites](https://v2.tauri.app/start/prerequisites/).
After installing tools, restart the terminal so PATH changes take effect.

```powershell
npm run desktop:dev
npm run desktop:build
npm run desktop:build:debug
npm run rust:fmt
npm run rust:check
npm run rust:lint
npm run rust:test
```

Native commands require the pinned Rust toolchain in `rust-toolchain.toml`.
Retain `src-tauri/Cargo.lock` in version control; checks use `--locked`.
Build the frontend before running Cargo checks directly because
the Tauri configuration embeds `dist` for release contexts.

Installer packaging is intentionally disabled until milestone 12. Native build
produces the desktop executable without creating an installer. Initial dependency
downloads require internet; application operation does not.
The debug build also embeds the frontend, allowing a faster standalone smoke
test without the Vite development server. The release build performs optimization.

## Project map

- `src/app`: application shell, hash router, navigation, and theme provider.
- `src/components/ui`: shared page-header and empty-state foundations.
- `src/features`: milestone-scoped feature pages and future module UI.
- `src/styles`: bundled styling; no remote fonts.
- `src/test`: frontend test setup.
- `src-tauri`: native app, configuration, and narrowly scoped capabilities.
- `src-tauri/migrations`: numbered, checksummed SQLite migrations.
- `docs`: architecture, data decisions, and milestone validation.

Do not put real personal data, database files, or backups in this repository.
The repository is under OneDrive; the future live database will use the operating
system's local application-data directory outside this checkout. Debug and
release builds use separate `development` and `production` subdirectories.
