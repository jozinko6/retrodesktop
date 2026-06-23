use std::path::Path;
use uuid::Uuid;
use walkdir::WalkDir;

use crate::{db, detection, domain::Game, error::{AppError, AppResult}};

const SUPPORTED: &[&str] = &[
    "nes","sfc","smc","gb","gbc","gba","n64","z64","v64","nds","md","gen","sms","gg",
    "a26","a52","a78","lnx","cue","chd","iso","cso","gcz","rvz","wbfs","wad","pbp",
    "zip","7z","rar","jsdos","adf","d64",
];

pub fn scan(connection: &rusqlite::Connection, root: &Path) -> AppResult<Vec<Game>> {
    if !root.is_dir() { return Err(AppError::NotFound(root.display().to_string())); }
    let mut games = Vec::new();
    for entry in WalkDir::new(root).follow_links(false).into_iter().filter_map(Result::ok) {
        if !entry.file_type().is_file() { continue; }
        let extension = entry.path().extension().and_then(|value| value.to_str()).unwrap_or("").to_ascii_lowercase();
        if !SUPPORTED.contains(&extension.as_str()) || (extension == "bin" && entry.path().with_extension("cue").exists()) { continue; }
        let result = detection::detect(entry.path())?;
        let title = entry.path().file_stem().and_then(|value| value.to_str()).unwrap_or("Neznáma hra").to_owned();
        let game = Game {
            id: Uuid::new_v4().to_string(),
            title,
            system_id: result.system_id.unwrap_or_else(|| "unknown".into()),
            primary_file: entry.path().display().to_string(),
            description: String::new(),
            release_year: None,
            developer: None,
            genre: None,
            total_play_time_seconds: 0,
            last_played_at: None,
            favorite: false,
            accent: "#15d6ff".into(),
        };
        db::upsert_scanned_game(connection, &game)?;
        games.push(game);
    }
    Ok(games)
}
