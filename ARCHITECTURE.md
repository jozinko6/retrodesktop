# Architecture

## Boundary

React is an unprivileged presentation layer. It calls narrow typed commands and cannot access SQLite, compose shell strings, read arbitrary secrets, or launch arbitrary programs.

Rust owns:

- application-data layout and SQLite migrations;
- path canonicalization and traversal policy;
- scanning, CUE handling, hashes, and detection;
- URL/provider parsing and redacted logging;
- emulator installation validation, argument arrays, process/session lifecycle;
- future download, extraction, credential, media, BIOS, and save services.

## Modules

- `db`: WAL SQLite, foreign keys, transactional migrations and queries.
- `library`: non-symlink folder traversal and game record creation.
- `detection`: extension plus file-signature evidence and confidence candidates.
- `download`: provider recognition and public-link normalization.
- `security`: URL, filename, child-path, and token-log safety.
- `emulators`: adapter registry and per-emulator argument arrays.
- `domain`: serialized command contracts shared with TypeScript equivalents.

## Data flow

UI → typed Tauri command → validation/canonicalization → Rust service → SQLite/filesystem/process → serialized result → UI.

No command uses `cmd.exe /c`, shell interpolation, or a frontend-provided executable plus arbitrary arguments.
