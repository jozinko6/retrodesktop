CREATE TABLE systems (
  id TEXT PRIMARY KEY,
  display_name TEXT NOT NULL,
  manufacturer TEXT,
  generation INTEGER
);
CREATE TABLE emulators (
  id TEXT PRIMARY KEY,
  display_name TEXT NOT NULL,
  advanced INTEGER NOT NULL DEFAULT 0
);
CREATE TABLE emulator_installations (
  id TEXT PRIMARY KEY,
  emulator_id TEXT NOT NULL REFERENCES emulators(id) ON DELETE CASCADE,
  executable TEXT NOT NULL UNIQUE,
  version TEXT,
  managed INTEGER NOT NULL DEFAULT 0,
  status TEXT NOT NULL,
  detected_at TEXT NOT NULL
);
CREATE TABLE retroarch_cores (
  id TEXT PRIMARY KEY,
  system_id TEXT NOT NULL REFERENCES systems(id),
  display_name TEXT NOT NULL,
  library_filename_windows TEXT NOT NULL,
  supported_extensions TEXT NOT NULL,
  priority INTEGER NOT NULL,
  requires_bios INTEGER NOT NULL DEFAULT 0
);
CREATE TABLE profiles (
  id TEXT PRIMARY KEY,
  name TEXT NOT NULL,
  avatar TEXT,
  pin_hash TEXT,
  child_mode INTEGER NOT NULL DEFAULT 0,
  created_at TEXT NOT NULL
);
CREATE TABLE launch_profiles (
  id TEXT PRIMARY KEY,
  name TEXT NOT NULL,
  emulator_id TEXT REFERENCES emulators(id),
  core_id TEXT REFERENCES retroarch_cores(id),
  arguments_json TEXT NOT NULL DEFAULT '[]'
);
CREATE TABLE games (
  id TEXT PRIMARY KEY,
  title TEXT NOT NULL,
  sort_title TEXT NOT NULL,
  system_id TEXT NOT NULL,
  primary_file TEXT NOT NULL UNIQUE,
  launch_file TEXT NOT NULL,
  file_hash TEXT,
  file_size INTEGER,
  region TEXT,
  serial TEXT,
  description TEXT,
  release_year INTEGER,
  release_date TEXT,
  developer TEXT,
  publisher TEXT,
  genre TEXT,
  players INTEGER,
  rating REAL,
  favorite INTEGER NOT NULL DEFAULT 0,
  hidden INTEGER NOT NULL DEFAULT 0,
  emulator_id TEXT REFERENCES emulators(id),
  core_id TEXT REFERENCES retroarch_cores(id),
  launch_profile_id TEXT REFERENCES launch_profiles(id),
  total_play_time_seconds INTEGER NOT NULL DEFAULT 0,
  last_played_at TEXT,
  accent TEXT,
  created_at TEXT NOT NULL,
  updated_at TEXT NOT NULL
);
CREATE TABLE game_files (
  id TEXT PRIMARY KEY,
  game_id TEXT NOT NULL REFERENCES games(id) ON DELETE CASCADE,
  path TEXT NOT NULL,
  role TEXT NOT NULL,
  size INTEGER,
  sha256 TEXT
);
CREATE TABLE game_assets (
  id TEXT PRIMARY KEY,
  game_id TEXT NOT NULL REFERENCES games(id) ON DELETE CASCADE,
  kind TEXT NOT NULL,
  relative_path TEXT NOT NULL,
  source TEXT
);
CREATE TABLE profile_favorites (
  profile_id TEXT NOT NULL REFERENCES profiles(id) ON DELETE CASCADE,
  game_id TEXT NOT NULL REFERENCES games(id) ON DELETE CASCADE,
  PRIMARY KEY(profile_id, game_id)
);
CREATE TABLE play_sessions (
  id TEXT PRIMARY KEY,
  game_id TEXT NOT NULL REFERENCES games(id),
  profile_id TEXT REFERENCES profiles(id),
  emulator_id TEXT REFERENCES emulators(id),
  started_at TEXT NOT NULL,
  ended_at TEXT,
  duration_seconds INTEGER,
  exit_code INTEGER
);
CREATE TABLE controller_profiles (id TEXT PRIMARY KEY, profile_id TEXT REFERENCES profiles(id), name TEXT NOT NULL, mapping_json TEXT NOT NULL);
CREATE TABLE watched_directories (id TEXT PRIMARY KEY, path TEXT NOT NULL UNIQUE, recursive INTEGER NOT NULL DEFAULT 1, last_scanned_at TEXT);
CREATE TABLE downloads (id TEXT PRIMARY KEY, source_redacted TEXT NOT NULL, target_path TEXT NOT NULL, status TEXT NOT NULL, bytes_downloaded INTEGER NOT NULL DEFAULT 0, total_bytes INTEGER, sha256 TEXT);
CREATE TABLE scrape_jobs (id TEXT PRIMARY KEY, game_id TEXT REFERENCES games(id), provider_id TEXT NOT NULL, status TEXT NOT NULL, error TEXT);
CREATE TABLE scrape_candidates (id TEXT PRIMARY KEY, job_id TEXT NOT NULL REFERENCES scrape_jobs(id) ON DELETE CASCADE, provider_candidate_id TEXT NOT NULL, title TEXT NOT NULL, confidence REAL NOT NULL, metadata_json TEXT NOT NULL);
CREATE TABLE settings (key TEXT PRIMARY KEY, value_json TEXT NOT NULL, updated_at TEXT NOT NULL);
CREATE TABLE bios_files (id TEXT PRIMARY KEY, system_id TEXT NOT NULL, relative_path TEXT NOT NULL UNIQUE, size INTEGER NOT NULL, sha256 TEXT NOT NULL, status TEXT NOT NULL);
CREATE TABLE save_locations (id TEXT PRIMARY KEY, game_id TEXT REFERENCES games(id), profile_id TEXT REFERENCES profiles(id), emulator_id TEXT REFERENCES emulators(id), path TEXT NOT NULL);
CREATE TABLE save_snapshots (id TEXT PRIMARY KEY, game_id TEXT NOT NULL REFERENCES games(id), profile_id TEXT REFERENCES profiles(id), emulator_id TEXT NOT NULL, original_path TEXT NOT NULL, backup_path TEXT NOT NULL, size INTEGER NOT NULL, sha256 TEXT NOT NULL, created_at TEXT NOT NULL, reason TEXT NOT NULL);

INSERT INTO emulators(id, display_name, advanced) VALUES
('retroarch','RetroArch',0),('duckstation','DuckStation',0),('pcsx2','PCSX2',0),
('dolphin','Dolphin',0),('ppsspp','PPSSPP',0),('rpcs3','RPCS3',1),('cemu','Cemu',1);
