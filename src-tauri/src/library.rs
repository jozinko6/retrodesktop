use std::{
    fs::{self, File},
    io::{self, BufReader, BufWriter, Read},
    path::{Path, PathBuf},
};
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
const MAX_ARCHIVE_ENTRIES: usize = 20_000;
const MAX_EXTRACTED_BYTES: u64 = 2_147_483_648;

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

pub fn prepare_launch_file(
    game_id: &str,
    source: &Path,
    system_id: &str,
    app_data: &Path,
) -> AppResult<PathBuf> {
    let canonical = source
        .canonicalize()
        .map_err(|_| AppError::NotFound(source.display().to_string()))?;
    let mut signature = [0_u8; 6];
    let count = File::open(&canonical)?.read(&mut signature)?;
    let archive_type = if count >= 6 && signature == [0x37, 0x7a, 0xbc, 0xaf, 0x27, 0x1c] {
        Some("7z")
    } else if count >= 4 && signature[..4] == [0x50, 0x4b, 0x03, 0x04] {
        Some("zip")
    } else {
        None
    };
    let Some(archive_type) = archive_type else {
        return Ok(canonical);
    };

    let imports = app_data.join("imports");
    fs::create_dir_all(&imports)?;
    let destination = imports.join(game_id);
    if destination.is_dir() {
        return find_launchable(&destination, system_id)
            .ok_or_else(|| AppError::NotFound("Spustiteľný obsah v rozbalenej hre".into()));
    }
    let staging = imports.join(format!(".{game_id}-{}", Uuid::new_v4()));
    fs::create_dir_all(&staging)?;
    let extraction = match archive_type {
        "zip" => extract_zip(&canonical, &staging),
        "7z" => extract_7z(&canonical, &staging),
        _ => unreachable!(),
    };
    if let Err(error) = extraction {
        let _ = fs::remove_dir_all(&staging);
        return Err(error);
    }
    let launchable = find_launchable(&staging, system_id).ok_or_else(|| {
        AppError::InvalidInput(
            "Archív neobsahuje podporovaný hlavný herný súbor pre zvolený systém.".into(),
        )
    })?;
    let relative = launchable
        .strip_prefix(&staging)
        .map_err(|_| AppError::Forbidden("Herný súbor mimo staging priečinka.".into()))?
        .to_path_buf();
    fs::rename(&staging, &destination)?;
    Ok(destination.join(relative))
}

fn extract_zip(source: &Path, destination: &Path) -> AppResult<()> {
    let mut archive = zip::ZipArchive::new(BufReader::new(File::open(source)?))
        .map_err(|error| AppError::InvalidInput(format!("ZIP archív: {error}")))?;
    if archive.len() > MAX_ARCHIVE_ENTRIES {
        return Err(AppError::Forbidden(
            "Archív obsahuje príliš veľa položiek.".into(),
        ));
    }
    let mut total = 0_u64;
    for index in 0..archive.len() {
        let mut entry = archive
            .by_index(index)
            .map_err(|error| AppError::InvalidInput(format!("ZIP položka: {error}")))?;
        let relative = entry
            .enclosed_name()
            .ok_or_else(|| AppError::Forbidden("Path traversal v ZIP archíve.".into()))?;
        if entry
            .unix_mode()
            .is_some_and(|mode| mode & 0o170000 == 0o120000)
        {
            return Err(AppError::Forbidden("Symlink v ZIP archíve.".into()));
        }
        total = total.saturating_add(entry.size());
        if total > MAX_EXTRACTED_BYTES {
            return Err(AppError::Forbidden(
                "Rozbalená hra prekračuje limit 2 GB.".into(),
            ));
        }
        let target = crate::security::safe_child(destination, &relative)?;
        if entry.is_dir() {
            fs::create_dir_all(target)?;
        } else {
            if let Some(parent) = target.parent() {
                fs::create_dir_all(parent)?;
            }
            let mut output = BufWriter::new(File::create(target)?);
            io::copy(&mut entry, &mut output)?;
        }
    }
    Ok(())
}

fn extract_7z(source: &Path, destination: &Path) -> AppResult<()> {
    let root = destination.to_path_buf();
    let mut total = 0_u64;
    let mut entries = 0_usize;
    sevenz_rust::decompress_file_with_extract_fn(source, destination, |entry, reader, _| {
        entries += 1;
        total = total.saturating_add(entry.size());
        if entries > MAX_ARCHIVE_ENTRIES || total > MAX_EXTRACTED_BYTES {
            return Err(sevenz_rust::Error::io(io::Error::new(
                io::ErrorKind::InvalidData,
                "7z archive limits exceeded",
            )));
        }
        let target =
            crate::security::safe_child(&root, Path::new(entry.name())).map_err(|error| {
                sevenz_rust::Error::io(io::Error::new(
                    io::ErrorKind::InvalidData,
                    error.to_string(),
                ))
            })?;
        if entry.is_directory() {
            fs::create_dir_all(&target).map_err(sevenz_rust::Error::io)?;
        } else {
            if let Some(parent) = target.parent() {
                fs::create_dir_all(parent).map_err(sevenz_rust::Error::io)?;
            }
            let mut output = BufWriter::new(File::create(target).map_err(sevenz_rust::Error::io)?);
            io::copy(reader, &mut output).map_err(sevenz_rust::Error::io)?;
        }
        Ok(true)
    })
    .map_err(|error| AppError::InvalidInput(format!("7z extrakcia hry: {error}")))
}

fn find_launchable(root: &Path, system_id: &str) -> Option<PathBuf> {
    let preferred: &[&str] = match system_id {
        "ps1" => &["cue", "chd", "pbp", "bin", "iso"],
        "ps2" | "ps3" => &["iso", "chd", "bin"],
        "psp" => &["cso", "iso", "pbp"],
        "gamecube" | "wii" => &["rvz", "gcz", "wbfs", "iso"],
        "wiiu" => &["wud", "wux", "rpx"],
        _ => SUPPORTED,
    };
    for extension in preferred {
        if let Some(path) = WalkDir::new(root)
            .max_depth(8)
            .follow_links(false)
            .into_iter()
            .filter_map(Result::ok)
            .find(|entry| {
                entry.file_type().is_file()
                    && entry
                        .path()
                        .extension()
                        .and_then(|value| value.to_str())
                        .is_some_and(|value| value.eq_ignore_ascii_case(extension))
            })
            .map(|entry| entry.into_path())
        {
            return Some(path);
        }
    }
    None
}

#[cfg(test)]
mod tests {
    use super::*;
    use tempfile::tempdir;

    #[test]
    fn detects_disguised_7z_and_prepares_ps1_cue() {
        let fixture = tempdir().unwrap();
        let content = fixture.path().join("content");
        fs::create_dir_all(&content).unwrap();
        fs::write(
            content.join("Tekken 3.cue"),
            "FILE \"Tekken 3.bin\" BINARY\n TRACK 01 MODE2/2352",
        )
        .unwrap();
        fs::write(content.join("Tekken 3.bin"), b"synthetic track").unwrap();
        let archive = fixture.path().join("Tekken 3.zip");
        sevenz_rust::compress_to_path(&content, &archive).unwrap();
        let app_data = fixture.path().join("app-data");

        let launch = prepare_launch_file("game-1", &archive, "ps1", &app_data).unwrap();

        assert_eq!(
            launch.extension().and_then(|value| value.to_str()),
            Some("cue")
        );
        assert!(launch.is_file());
    }
}
