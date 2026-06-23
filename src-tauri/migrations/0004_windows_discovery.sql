INSERT OR IGNORE INTO systems(id, display_name, manufacturer) VALUES
('windows', 'Windows', 'Microsoft');
INSERT OR IGNORE INTO emulators(id, display_name, advanced) VALUES
('windows', 'Windows Native', 0);

CREATE TABLE windows_game_candidates (
  id TEXT PRIMARY KEY,
  source TEXT NOT NULL,
  source_id TEXT,
  title TEXT NOT NULL,
  install_path TEXT,
  launch_kind TEXT NOT NULL,
  launch_target TEXT NOT NULL,
  launch_arguments_json TEXT NOT NULL DEFAULT '[]',
  working_directory TEXT,
  confidence REAL NOT NULL,
  evidence_json TEXT NOT NULL,
  status TEXT NOT NULL DEFAULT 'pending',
  discovered_at TEXT NOT NULL DEFAULT CURRENT_TIMESTAMP,
  UNIQUE(source, source_id, launch_target)
);

CREATE TABLE game_launches (
  game_id TEXT PRIMARY KEY REFERENCES games(id) ON DELETE CASCADE,
  launch_kind TEXT NOT NULL,
  launch_target TEXT NOT NULL,
  launch_arguments_json TEXT NOT NULL DEFAULT '[]',
  working_directory TEXT,
  source TEXT NOT NULL,
  source_id TEXT
);

CREATE INDEX idx_windows_candidates_status ON windows_game_candidates(status, source);
