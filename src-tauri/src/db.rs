use std::{fs, path::Path};

use rusqlite::{params, Connection};

use crate::{domain::Game, error::AppResult};

const MIGRATIONS: &[(&str, &str)] = &[
    (
        "0001_initial",
        include_str!("../migrations/0001_initial.sql"),
    ),
    (
        "0002_indexes",
        include_str!("../migrations/0002_indexes.sql"),
    ),
    (
        "0003_seed_systems",
        include_str!("../migrations/0003_seed_systems.sql"),
    ),
];

pub fn open(path: &Path) -> AppResult<Connection> {
    if let Some(parent) = path.parent() {
        fs::create_dir_all(parent)?;
    }
    let mut connection = Connection::open(path)?;
    connection.pragma_update(None, "foreign_keys", "ON")?;
    connection.pragma_update(None, "journal_mode", "WAL")?;
    migrate(&mut connection)?;
    Ok(connection)
}

pub fn migrate(connection: &mut Connection) -> AppResult<()> {
    connection.execute_batch(
        "CREATE TABLE IF NOT EXISTS schema_migrations (
          version TEXT PRIMARY KEY,
          applied_at TEXT NOT NULL DEFAULT CURRENT_TIMESTAMP
        );",
    )?;
    for (version, sql) in MIGRATIONS {
        let exists: bool = connection.query_row(
            "SELECT EXISTS(SELECT 1 FROM schema_migrations WHERE version = ?1)",
            [version],
            |row| row.get(0),
        )?;
        if !exists {
            let transaction = connection.transaction()?;
            transaction.execute_batch(sql)?;
            transaction.execute(
                "INSERT INTO schema_migrations(version) VALUES (?1)",
                [version],
            )?;
            transaction.commit()?;
        }
    }
    Ok(())
}

pub fn games(connection: &Connection) -> AppResult<Vec<Game>> {
    let mut statement = connection.prepare(
        "SELECT id, title, system_id, primary_file, COALESCE(description, ''),
         release_year, developer, genre, total_play_time_seconds, last_played_at,
         favorite, COALESCE(accent, '#15d6ff') FROM games ORDER BY sort_title",
    )?;
    let rows = statement.query_map([], |row| {
        Ok(Game {
            id: row.get(0)?,
            title: row.get(1)?,
            system_id: row.get(2)?,
            primary_file: row.get(3)?,
            description: row.get(4)?,
            release_year: row.get(5)?,
            developer: row.get(6)?,
            genre: row.get(7)?,
            total_play_time_seconds: row.get(8)?,
            last_played_at: row.get(9)?,
            favorite: row.get(10)?,
            accent: row.get(11)?,
        })
    })?;
    Ok(rows.collect::<Result<Vec<_>, _>>()?)
}

pub fn upsert_scanned_game(connection: &Connection, game: &Game) -> AppResult<String> {
    let existing = connection.query_row(
        "SELECT id FROM games WHERE primary_file=?1",
        [&game.primary_file],
        |row| row.get::<_, String>(0),
    );
    let stable_id = existing.unwrap_or_else(|_| game.id.clone());
    connection.execute(
        "INSERT INTO games (
          id, title, sort_title, system_id, primary_file, launch_file, description,
          total_play_time_seconds, favorite, accent, created_at, updated_at
        ) VALUES (?1, ?2, ?2, ?3, ?4, ?4, ?5, 0, 0, ?6, CURRENT_TIMESTAMP, CURRENT_TIMESTAMP)
        ON CONFLICT(primary_file) DO UPDATE SET title=excluded.title, system_id=excluded.system_id, updated_at=CURRENT_TIMESTAMP",
        params![stable_id, game.title, game.system_id, game.primary_file, game.description, game.accent],
    )?;
    connection.execute(
        "INSERT INTO game_files(id, game_id, path, role, size)
         VALUES (?1, ?2, ?3, 'primary', ?4)
         ON CONFLICT(id) DO UPDATE SET size=excluded.size",
        params![
            format!("{}:primary", stable_id),
            stable_id,
            game.primary_file,
            std::fs::metadata(&game.primary_file)
                .map(|item| item.len() as i64)
                .ok()
        ],
    )?;
    Ok(stable_id)
}

pub fn save_watched_directory(connection: &Connection, path: &str) -> AppResult<()> {
    connection.execute(
        "INSERT INTO watched_directories(id, path, last_scanned_at)
         VALUES (?1, ?2, CURRENT_TIMESTAMP)
         ON CONFLICT(path) DO UPDATE SET last_scanned_at=CURRENT_TIMESTAMP",
        params![uuid::Uuid::new_v4().to_string(), path],
    )?;
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn migrations_are_idempotent() {
        let mut connection = Connection::open_in_memory().unwrap();
        connection
            .pragma_update(None, "foreign_keys", "ON")
            .unwrap();
        migrate(&mut connection).unwrap();
        migrate(&mut connection).unwrap();
        let count: i64 = connection
            .query_row("SELECT COUNT(*) FROM schema_migrations", [], |row| {
                row.get(0)
            })
            .unwrap();
        assert_eq!(count, MIGRATIONS.len() as i64);
    }
}
