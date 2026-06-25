# Windows Game Discovery

Windows Game Discovery is a confirmation-first importer for locally installed PC games.

## Implemented sources

- Steam installation detection from common folders and the current-user registry.
- Steam `libraryfolders.vdf` plus every `appmanifest_*.acf`.
- Epic Games `.item` manifests from ProgramData.
- GOG game registry entries.
- Conservatively filtered Windows installed-program registry entries.
- `.lnk` shortcuts from user/public desktops and Start Menu folders.
- Optional, explicitly selected portable-game directory scan.

Steam entries use `steam://rungameid/{appId}`. Epic, GOG, shortcuts, and portable candidates use a canonical executable path, a separate argument array, and a working directory. Arbitrary shell strings are never stored or executed.

## Confirmation and confidence

Discovery results are stored separately from the game library. Steam/Epic/GOG matches are preselected because they carry stable launcher evidence. Shortcut and portable matches remain visible with confidence and evidence so the user can accept or reject them.

Portable discovery is bounded to four directory levels and 20,000 entries per selected root. It rejects installer, updater, uninstaller, crash reporter, launcher, server, editor, benchmark, and configuration executables. A candidate needs multiple signals such as a Games-like directory, significant local data, game archive/DLL presence, and a named executable.

## Duplicates

Candidates are deduplicated by launcher source ID and normalized launch target. Confirmed launch records are upserted rather than duplicated.

## Current limitations

- Xbox/Microsoft Store App IDs are not imported yet because reliable enumeration and launch validation require a dedicated packaged-app adapter.
- EA App, Ubisoft Connect, and Battle.net games may be found through Start Menu shortcuts, but native launcher manifest adapters are not implemented yet.
- Registry discovery requires a valid executable plus multiple game-like directory signals; it intentionally ignores most installed applications.
- Deep portable scanning is bounded and repeatable, but directory modification indexing for a true changed-folders-only scan is follow-up work.
- Launcher URI processes return immediately, so accurate play duration for Steam/Epic launcher handoff needs launcher-specific process association.
- Online IGDB/SteamGridDB metadata and executable icon extraction remain separate metadata work.
