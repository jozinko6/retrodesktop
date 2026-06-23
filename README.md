# RetroBox Desktop

RetroBox Desktop is a local Windows 10/11 x64 big-picture library for legally owned game backups. It is a Tauri 2 application: React renders the television-friendly interface while Rust owns the database, filesystem, detection, downloads, emulator processes, and security boundaries.

## Current state

The repository now builds as a native Windows application and includes a complete first RetroArch path: select and validate `retroarch.exe`, scan a watched folder, choose a libretro core DLL for a game, launch without shell interpolation, track the process and play session, capture a launch log, update play time, and restore application focus.

Remote metadata clients, streamed/resumable production downloads, managed emulator installation, archive extraction, credential-vault integration, and save snapshot UI remain follow-up work. The UI does not claim these are complete.

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

The Vite-only preview (`pnpm dev`) shows the production empty state and cannot launch a game. Demo data is available only with `VITE_DEMO_DATA=true`.

## Production build

```powershell
pnpm tauri build
```

NSIS/MSI output is written below `src-tauri/target/release/bundle/`. See [BUILD_WINDOWS.md](BUILD_WINDOWS.md).

## First run

The setup guide introduces the data directory, controller, emulator, BIOS, metadata, watched-folder, and scan steps. It may be skipped and revisited.

## Import and scanning

Rust scans explicitly selected watched folders without following symlinks. It persists watched folders and stable game IDs, fills `game_files`, validates every CUE track, uses magic bytes where available, treats CUE as the primary file for multi-track discs, and does not infer a platform from title words.

Link imports accept only HTTP/HTTPS and recognize direct links plus public Google Drive, Dropbox, and OneDrive link shapes. Private links, authentication bypasses, ROM sites, torrents, and BIOS downloads are intentionally unsupported.

## Scraping

The provider architecture targets ScreenScraper, SteamGridDB, IGDB, and an offline filename fallback. Credentials must eventually be stored in Windows Credential Manager; they must never be kept in localStorage or committed. Production remote clients are not yet implemented in this baseline.

## Emulators and BIOS

Adapters exist for RetroArch, PCSX2, Dolphin, PPSSPP, DuckStation, RPCS3, and Cemu. RetroArch has the first end-to-end configuration and launch flow. Detection and argument construction are implemented without shell interpolation. Before launch, users must provide a real executable, core, and any lawfully obtained BIOS/firmware required by that emulator.

RetroBox never ships games, BIOS, firmware, saves, credentials, or downloaded emulator packages.

## Controls and fullscreen

Mouse and keyboard work naturally. The controller hook supports D-pad/left stick, deadzone, edge detection, delayed repeat, and prevents held accept buttons from repeatedly launching. Console-mode fullscreen/autostart settings remain to be wired to native Windows integration.

## Known limitations

- Real RetroArch execution requires a user-installed RetroArch executable and compatible core DLL.
- Standalone emulator configuration is available, but full per-emulator BIOS/save diagnostics still follow the RetroArch vertical slice.
- Managed downloads, extraction, remote scraping, media cache, BIOS hash catalog, save snapshots, diagnostics ZIP, and autostart are specified but not complete.
- The generated concept image is design documentation, not shipped UI content.
