# RetroBox Desktop

RetroBox Desktop is a local Windows 10/11 x64 big-picture library for legally owned game backups. It is a Tauri 2 application: React renders the television-friendly interface while Rust owns the database, filesystem, detection, downloads, emulator processes, and security boundaries.

## Current state

The repository now builds as a native Windows application and includes a complete first RetroArch path: select and validate `retroarch.exe`, scan a watched folder, choose a libretro core DLL for a game, launch without shell interpolation, track the process and play session, capture a launch log, update play time, and restore application focus.

Windows Game Discovery can scan Steam libraries, Epic manifests, GOG registry entries, Windows shortcuts, and an explicitly selected portable-games directory. Results are deduplicated and shown for confirmation before entering the Windows category. See [WINDOWS_DISCOVERY.md](WINDOWS_DISCOVERY.md).

Remote metadata clients, resumable game downloads, credential-vault integration, and save snapshot UI remain follow-up work. Managed emulator installation now downloads official Windows portable releases, verifies archive boundaries, calculates SHA-256, extracts into the RetroBox data directory, and records the validated executable.

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

Adapters exist for RetroArch, PCSX2, Dolphin, PPSSPP, DuckStation, RPCS3, and Cemu. RetroArch, PCSX2, PPSSPP, DuckStation, and Cemu can be installed on demand from their official release channels. Dolphin and RPCS3 currently keep the official-download/manual-executable path because their release endpoints do not provide a stable resolver suitable for unattended installation.

`Vložiť BIOS` accepts only emulator-appropriate local BIOS/firmware files, enforces size and extension limits, calculates SHA-256, and stores the file under the RetroBox data directory. RetroBox never downloads or ships games, BIOS, firmware, saves, or credentials.

## System libraries and metadata

The Systems page lists every supported platform, including empty libraries. Opening a system allows importing one local game file or scanning a folder while explicitly assigning that platform. New imports normalize common dump and disc tags and attempt a strict title match through the public MediaWiki API. Accepted matches store the canonical title, introductory description, release year when available, a short neutral overview, source URL, and an available page image. Media is cached under `app-data/media/{systemId}/{gameId}` with HTTPS, MIME and 10 MB limits. Existing games can request a metadata refresh from their system detail. If the title match is uncertain, an image is unsuitable, rate limiting occurs, or the service is unavailable, the legal local import still succeeds and RetroBox keeps the filename-derived title without inventing metadata.

## Controls and fullscreen

Mouse and keyboard work naturally. The controller hook supports D-pad/left stick, deadzone, edge detection, delayed repeat, and prevents held accept buttons from repeatedly launching. Console-mode fullscreen/autostart settings remain to be wired to native Windows integration.

## Remote play

The Remote Play page integrates the official Sunshine host with Moonlight clients. RetroBox detects a local Sunshine installation, can start its executable, shows the PC's LAN address, and opens the local pairing Web UI. If Sunshine is missing, the page opens its official latest release; Moonlight links point to the official client downloads.

For the most reliable remote multiplayer setup, connect multiple Bluetooth controllers to one Moonlight phone, tablet, or TV and use a game that supports local split-screen or couch co-op. Separate client devices depend on the game's controller mapping and concurrent Sunshine sessions. RetroBox does not open Internet-facing ports; use a trusted VPN for play outside the local network.

## Known limitations

- Real RetroArch execution still requires a compatible libretro core DLL for each configured game.
- Managed emulator downloads are not resumable yet and show an indeterminate progress state while downloading and extracting.
- Sunshine installation and firewall approval remain an explicit user-controlled step because the official Windows installer may require administrator access.
- Full per-emulator BIOS placement diagnostics, remote scraping, media cache, save snapshots, diagnostics ZIP, and autostart remain follow-up work.
- The generated concept image is design documentation, not shipped UI content.
