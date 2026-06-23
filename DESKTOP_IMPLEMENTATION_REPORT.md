# RetroBox Desktop MVP implementation report

## Delivered vertical slice

- New Tauri 2 / Rust / React / TypeScript / Vite / SQLite project.
- Versioned transactional schema covering the requested entity set.
- Fullscreen-oriented Slovak UI, onboarding, controller navigation primitives, library rails, and emulator manager.
- Typed frontend command client with an honest browser preview fallback.
- Local scan, conservative platform evidence, CUE multi-BIN parser, and non-symlink traversal.
- Public direct/Google Drive/Dropbox/OneDrive provider recognition.
- Security helpers for URL schemes, embedded credentials, path traversal, Windows filenames, and token redaction.
- Real adapter registry and argument arrays for seven emulators; no shell interpolation.
- Production empty state with no fictional games unless explicit preview mode is enabled.
- Persistent watched directories, stable game IDs, `game_files`, and integrated CUE track validation.
- RetroArch executable/core configuration and process launch with stdout/stderr log, play-session persistence, play-time update, minimize/restore, and focus return.
- Windows Game Discovery for Steam, Epic, GOG, Windows shortcuts, and bounded portable scans, including confidence evidence, confirmation, deduplication, and direct launch records.
- Frontend and Rust unit tests plus Windows CI.
- Successfully generated NSIS and MSI installers on Windows.

## Not represented as complete

Production streamed/resumable downloads, archive extraction, remote scraping/media clients, credential vault, managed emulator installer, BIOS hash catalog, save snapshots, autostart/kiosk actions, diagnostics export, and physical-controller/real-game compatibility testing.

## Verification

- `pnpm typecheck`, `pnpm lint`, `pnpm test`, and `pnpm build`
- `cargo fmt --check`, `cargo clippy -D warnings`, and `cargo test`
- native release executable startup with SQLite initialization
- fake-emulator integration test for argument passing, output capture, and exit tracking
- `pnpm tauri build` producing NSIS and MSI bundles

## Legal content

No ROM, BIOS, firmware, save, credential, emulator binary, or unauthorized download source is included. The application is designed only for user-owned backups and official emulator sources.
