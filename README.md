# RetroBox Desktop

RetroBox Desktop is a local Windows 10/11 x64 big-picture library for legally owned game backups. It is a Tauri 2 application: React renders the television-friendly interface while Rust owns the database, filesystem, detection, downloads, emulator processes, and security boundaries.

## Current state

The repository contains a working frontend shell, skippable onboarding, gamepad navigation primitives, library/emulator views, typed Tauri calls, SQLite migrations, library scanning, conservative platform detection, public-link parsing, filename/path/token safety, and launch-command adapters for seven emulators.

The first commit is an honest foundation, not the entirety of the long-term product specification. Remote metadata clients, streamed/resumable production downloads, managed emulator installation, archive extraction, credential-vault integration, save snapshot UI, and real-emulator end-to-end verification remain follow-up work. The UI does not claim these are complete.

## Requirements

- Windows 10 or 11 x64
- Node.js 22+
- pnpm 10+
- Rust stable with the MSVC target
- Visual Studio 2022 Build Tools with “Desktop development with C++”
- WebView2 Runtime

## Development

```powershell
pnpm install --frozen-lockfile
pnpm typecheck
pnpm lint
pnpm test
pnpm tauri dev
```

The Vite-only preview (`pnpm dev`) uses clearly identified sample data and cannot launch a game.

## Production build

```powershell
pnpm tauri build
```

NSIS/MSI output is written below `src-tauri/target/release/bundle/`. See [BUILD_WINDOWS.md](BUILD_WINDOWS.md).

## First run

The setup guide introduces the data directory, controller, emulator, BIOS, metadata, watched-folder, and scan steps. It may be skipped and revisited.

## Import and scanning

Rust scans explicitly selected watched folders without following symlinks. It recognizes supported extensions, uses magic bytes where available, treats CUE as the primary file for multi-track discs, and does not infer a platform from title words.

Link imports accept only HTTP/HTTPS and recognize direct links plus public Google Drive, Dropbox, and OneDrive link shapes. Private links, authentication bypasses, ROM sites, torrents, and BIOS downloads are intentionally unsupported.

## Scraping

The provider architecture targets ScreenScraper, SteamGridDB, IGDB, and an offline filename fallback. Credentials must eventually be stored in Windows Credential Manager; they must never be kept in localStorage or committed. Production remote clients are not yet implemented in this baseline.

## Emulators and BIOS

Adapters exist for RetroArch, PCSX2, Dolphin, PPSSPP, DuckStation, RPCS3, and Cemu. Detection and argument construction are implemented without shell interpolation. Before launch, users must provide a real executable and any lawfully obtained BIOS/firmware required by that emulator.

RetroBox never ships games, BIOS, firmware, saves, credentials, or downloaded emulator packages.

## Controls and fullscreen

Mouse and keyboard work naturally. The controller hook supports D-pad/left stick, deadzone, edge detection, delayed repeat, and prevents held accept buttons from repeatedly launching. Console-mode fullscreen/autostart settings remain to be wired to native Windows integration.

## Known limitations

- Rust/Tauri builds require a locally installed Rust MSVC toolchain.
- Real emulator launch/session persistence is intentionally blocked until an executable installation is configured.
- Managed downloads, extraction, remote scraping, media cache, BIOS hash catalog, save snapshots, diagnostics ZIP, and autostart are specified but not complete.
- The generated concept image is design documentation, not shipped UI content.
