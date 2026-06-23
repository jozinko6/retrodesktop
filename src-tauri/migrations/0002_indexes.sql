CREATE INDEX idx_games_system ON games(system_id);
CREATE INDEX idx_games_recent ON games(last_played_at DESC);
CREATE INDEX idx_game_files_game ON game_files(game_id);
CREATE INDEX idx_play_sessions_game ON play_sessions(game_id, started_at DESC);
CREATE INDEX idx_downloads_status ON downloads(status);
CREATE INDEX idx_scrape_jobs_status ON scrape_jobs(status);
