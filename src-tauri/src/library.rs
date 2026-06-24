use std::{fs, path::Path};
use uuid::Uuid;
use walkdir::WalkDir;

use crate::{
    db, detection,
    domain::Game,
    error::{AppError, AppResult},
};

const SUPPORTED: &[&str] = &[
    "nes", "sfc", "smc", "gb", "gbc", "gba", "n64", "z64", "v64", "nds", "md", "gen", "sms", "gg",
    "a26", "a52", "a78", "lnx", "cue", "chd", "iso", "cso", "gcz", "rvz", "wbfs", "wad", "pbp",
    "zip", "7z", "rar", "jsdos", "adf", "d64", "exe", "bat", "com", "bin",
];

pub fn scan(connection: &rusqlite::Connection, root: &Path) -> AppResult<Vec<Game>> {
    scan_for_system(connection, root, None)
}

pub fn scan_for_system(
    connection: &rusqlite::Connection,
    root: &Path,
    forced_system: Option<&str>,
) -> AppResult<Vec<Game>> {
    if !root.is_dir() {
        return Err(AppError::NotFound(root.display().to_string()));
    }
    let mut games = Vec::new();
    for entry in WalkDir::new(root)
        .follow_links(false)
        .into_iter()
        .filter_map(Result::ok)
    {
        if !entry.file_type().is_file() {
            continue;
        }
        let extension = extension(entry.path());
        if !SUPPORTED.contains(&extension.as_str())
            || (extension == "bin" && entry.path().with_extension("cue").exists())
        {
            continue;
        }
        validate_cue(entry.path(), root, &extension)?;
        games.push(import_file(connection, entry.path(), forced_system)?);
    }
    db::save_watched_directory(connection, &root.display().to_string())?;
    Ok(games)
}

pub fn import_file(
    connection: &rusqlite::Connection,
    path: &Path,
    forced_system: Option<&str>,
) -> AppResult<Game> {
    if !path.is_file() {
        return Err(AppError::NotFound(path.display().to_string()));
    }
    let file_extension = extension(path);
    if !SUPPORTED.contains(&file_extension.as_str()) {
        return Err(AppError::InvalidInput(format!(
            "Nepodporovaný formát hry: .{file_extension}"
        )));
    }
    let parent = path.parent().unwrap_or_else(|| Path::new("."));
    validate_cue(path, parent, &file_extension)?;
    let result = detection::detect(path)?;
    let mut game = Game {
        id: Uuid::new_v4().to_string(),
        title: path
            .file_stem()
            .and_then(|value| value.to_str())
            .unwrap_or("Neznáma hra")
            .to_owned(),
        system_id: forced_system
            .map(str::to_owned)
            .or(result.system_id)
            .unwrap_or_else(|| "unknown".into()),
        primary_file: path.display().to_string(),
        description: String::new(),
        release_year: None,
        developer: None,
        genre: None,
        total_play_time_seconds: 0,
        last_played_at: None,
        favorite: false,
        accent: "#15d6ff".into(),
        short_review: None,
        metadata_source: None,
        cover_path: None,
    };
    game.id = db::upsert_scanned_game(connection, &game)?;
    Ok(game)
}

fn extension(path: &Path) -> String {
    path.extension()
        .and_then(|value| value.to_str())
        .unwrap_or("")
        .to_ascii_lowercase()
}

fn validate_cue(path: &Path, fallback_root: &Path, extension: &str) -> AppResult<()> {
    if extension != "cue" {
        return Ok(());
    }
    let contents = fs::read_to_string(path)?;
    let parent = path.parent().unwrap_or(fallback_root);
    for referenced in detection::parse_cue(&contents) {
        let track = crate::security::safe_child(parent, Path::new(&referenced))?;
        if !track.is_file() {
            return Err(AppError::InvalidInput(format!(
                "CUE odkazuje na chýbajúci track: {}",
                track.display()
            )));
        }
    }
    Ok(())
}
