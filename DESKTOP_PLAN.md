# RetroBox Desktop — implementation plan

## MVP slices

1. Bootstrap Tauri 2 + React + TypeScript + Vite and establish a minimal permission model.
2. Add the versioned SQLite schema, application data directories, typed commands, and shared domain types.
3. Implement local library scanning, extension/magic-based platform detection, CUE validation, hashing, and watched folders.
4. Implement safe process launch/session tracking and modular adapters for RetroArch, PCSX2, Dolphin, PPSSPP, DuckStation, RPCS3, and Cemu.
5. Implement direct/public link resolution, safe streamed downloads, cancellation, archive policy, and filename/token sanitization.
6. Build the fullscreen library, onboarding, gamepad navigation, profiles, emulator/BIOS/settings/diagnostics surfaces.
7. Add metadata provider boundaries with LocalFilename as the offline implementation and configured remote provider clients.
8. Add save snapshots, tests, Windows packaging configuration, CI, and operating/security documentation.

## Definition of honest completion

Features are marked ready only when backed by a Rust command and testable behavior. UI-only surfaces are labelled as configuration or planned work. BIOS, ROMs, credentials, emulators, and user databases are never bundled.

## Environment note

At project creation time Node 22.15.1 and pnpm 10.18.2 were available. Rust/Cargo and GitHub CLI were not installed, so Rust compilation, Tauri packaging, remote creation, push, PR, and CI verification require those tools to be installed.
