# Security model

- Minimal Tauri permissions: core window APIs and file-open dialog only.
- No shell plugin and no general process permission in the webview.
- Emulator processes are constructed in Rust from registered adapters.
- HTTP/HTTPS only; embedded URL credentials and dangerous protocols are rejected.
- Query tokens are redacted before logging.
- User paths are canonicalized before scanning.
- Watched-folder traversal does not follow symlinks.
- Archive destinations must reject absolute paths, parent components, links, oversized totals, excessive ratios, and entry-count bombs.
- SQLite uses parameters, foreign keys, WAL, and migration transactions.
- Writes for downloads, media, BIOS, and saves must use temporary files followed by atomic rename.
- ROMs, BIOS, firmware, saves, credentials, user databases, and emulator packages are excluded from Git.

The current code includes path/URL/name/redaction tests. Production archive extraction, Windows Credential Manager storage, and streaming download policy are mandatory before those features can be marked ready.
