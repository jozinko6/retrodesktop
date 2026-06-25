use std::{
    fs::{self, File, OpenOptions},
    io::{BufWriter, Read, Write},
    path::{Path, PathBuf},
    process::Command,
    time::Duration,
};

use chrono::Utc;
use reqwest::{blocking::Client, redirect::Policy};
use rusqlite::Connection;
use serde::Serialize;
use sha2::{Digest, Sha256};
use tauri::{AppHandle, Emitter};
use url::Url;
use uuid::Uuid;

use crate::{
    domain::{CatalogDownloadProgress, CatalogGame},
    error::{AppError, AppResult},
    security::{sanitize_filename, validate_download_url},
};

#[derive(Debug, PartialEq)]
pub enum Provider {
    DirectHttp,
    GoogleDrivePublic,
    DropboxPublic,
    OneDrivePublic,
}

#[derive(Debug)]
pub struct DownloadedCatalogGame {
    pub catalog: CatalogGame,
    pub path: PathBuf,
    pub sha256: String,
}

#[derive(Serialize)]
#[serde(rename_all = "camelCase")]
struct CatalogSidecar<'a> {
    title: &'a str,
    description: &'a str,
    system_id: &'a str,
    developer: &'a str,
    genre: &'a str,
    license: &'a str,
    license_url: &'a str,
    source_url: &'a str,
    sha256: &'a str,
    file_size: u64,
}

pub fn catalog() -> Vec<CatalogGame> {
    vec![
        item(
            "gruniozerca",
            "Gruniożerca",
            "Jednoduchá arkádová hra pre NES, zverejnená autormi ako public-domain dielo.",
            "nes",
            "Łukasz Kur a Ryszard Brzukała",
            "Arkádová",
            "Unlicense / CC BY-SA 3.0 hudba",
            "https://github.com/arhneu/gruniozerca",
            "https://github.com/arhneu/gruniozerca",
            "grunio.nes",
            40_976,
            "ae049634a943de140c238b5b4c416d9cf22054cb88d228d22aade1942cf478b0",
            "https://opengraph.githubassets.com/42e94aa976af2230b43876f8c7aebd34a70a9149/arhneu/gruniozerca",
        ),
        item(
            "rainbow-chat",
            "Rainbow Chat",
            "Online chat demo pre NES od Broke Studio. Slúži aj ako verejne redistribuovateľný test sieťových možností konzoly.",
            "nes",
            "Broke Studio",
            "Homebrew demo",
            "CC0-1.0",
            "https://github.com/BrokeStudio/rainbow-chat/blob/9d08aa2a7667acc67eaac6cb75562fc5445a9304/LICENSE",
            "https://github.com/BrokeStudio/rainbow-chat",
            "rainbow-chat.nes",
            1_048_592,
            "ed41798abf97baa24060afab35d56a5873c8b6eb8c19a57bc8d0d3a8537ceabe",
            "https://opengraph.githubassets.com/9d08aa2a7667acc67eaac6cb75562fc5445a9304/BrokeStudio/rainbow-chat",
        ),
        item(
            "gbclock",
            "GBClock",
            "Homebrew hodiny pre pôvodný Game Boy. Praktická CC0 ukážka a malý testovací cartridge obraz.",
            "gb",
            "kresp0",
            "Nástroj / homebrew",
            "CC0-1.0",
            "https://github.com/kresp0/gbclock/blob/f4de4a9be6265810471e69cca62a2bc025242286/LICENSE",
            "https://github.com/kresp0/gbclock",
            "gbclock-v0.3.gb",
            262_144,
            "74094df68ca474bd3cbd8f61bc88f0e4bc456f17ad76125b33037d42d6ced59e",
            "https://opengraph.githubassets.com/f4de4a9be6265810471e69cca62a2bc025242286/kresp0/gbclock",
        ),
        item(
            "2048-nes",
            "2048 NES",
            "Port logickej hry 2048 pre Nintendo Entertainment System, distribuovaný pod licenciou Unlicense.",
            "nes",
            "Michał Muszkowski",
            "Logická",
            "Unlicense",
            "https://github.com/mmuszkow/2048-nes/blob/d0b2841384b7881619e53863d5bb7a27263ccccd/LICENSE",
            "https://github.com/mmuszkow/2048-nes",
            "2048.nes",
            24_592,
            "2769543d0b5bb44ff38c2f3d259b01c3a93868a0607fb57f43fd8986f5f8fd0f",
            "https://opengraph.githubassets.com/d0b2841384b7881619e53863d5bb7a27263ccccd/mmuszkow/2048-nes",
        ),
        item(
            "big2small",
            "Big2Small",
            "Pôvodná logická homebrew hra pre Game Boy a Game Boy Color s redistribuovateľným GPL-3.0 vydaním.",
            "gb",
            "Matthew D. Steele",
            "Logická",
            "GPL-3.0",
            "https://github.com/mdsteele/big2small/blob/v1.0.0/LICENSE",
            "https://github.com/mdsteele/big2small/releases/tag/v1.0.0",
            "big2small.gb",
            65_536,
            "59c096333f93c12c8eb25a5fcffa2be15001abcd5d20b9a82c2bc2dae7e625b2",
            "https://opengraph.githubassets.com/v1.0.0/mdsteele/big2small",
        ),
    ]
}

#[allow(clippy::too_many_arguments)]
fn item(
    id: &str,
    title: &str,
    description: &str,
    system_id: &str,
    developer: &str,
    genre: &str,
    license: &str,
    license_url: &str,
    source_url: &str,
    file_name: &str,
    file_size: u64,
    sha256: &str,
    thumbnail_url: &str,
) -> CatalogGame {
    CatalogGame {
        id: id.into(),
        title: title.into(),
        description: description.into(),
        system_id: system_id.into(),
        developer: developer.into(),
        genre: genre.into(),
        license: license.into(),
        license_url: license_url.into(),
        source_url: source_url.into(),
        file_name: file_name.into(),
        file_size,
        sha256: sha256.into(),
        thumbnail_url: thumbnail_url.into(),
    }
}

pub fn download_catalog_game(
    game_id: &str,
    target_directory: &Path,
    app_data: &Path,
    connection: &Connection,
    app: &AppHandle,
) -> AppResult<DownloadedCatalogGame> {
    let catalog = catalog()
        .into_iter()
        .find(|item| item.id == game_id)
        .ok_or_else(|| AppError::NotFound(format!("Katalógová hra {game_id}")))?;
    validate_catalog_item(&catalog)?;
    let target_directory = target_directory
        .canonicalize()
        .map_err(|_| AppError::NotFound(target_directory.display().to_string()))?;
    if !target_directory.is_dir() {
        return Err(AppError::InvalidInput(
            "Cieľ sťahovania musí byť existujúci priečinok.".into(),
        ));
    }

    let file_name = sanitize_filename(&catalog.file_name);
    let final_path = target_directory.join(file_name);
    let part_path = target_directory.join(format!(".{}.{}.part", catalog.id, Uuid::new_v4()));
    let download_id = Uuid::new_v4().to_string();
    connection.execute(
        "INSERT INTO downloads(id, source_redacted, target_path, status, total_bytes)
         VALUES (?1, ?2, ?3, 'downloading', ?4)",
        rusqlite::params![
            download_id,
            catalog.source_url,
            final_path.display().to_string(),
            catalog.file_size as i64
        ],
    )?;
    log(
        app_data,
        &format!("START {} -> {}", catalog.id, final_path.display()),
    )?;

    let result = perform_download(&catalog, &part_path, app);
    let hash = match result {
        Ok(hash) => hash,
        Err(error) => {
            let _ = fs::remove_file(&part_path);
            connection.execute(
                "UPDATE downloads SET status='failed' WHERE id=?1",
                [&download_id],
            )?;
            let _ = log(app_data, &format!("FAILED {}: {error}", catalog.id));
            let _ = emit_progress(app, &catalog, 0, "failed");
            return Err(error);
        }
    };

    if final_path.exists() {
        let existing = sha256_file(&final_path)?;
        if existing != catalog.sha256 {
            let _ = fs::remove_file(&part_path);
            return Err(AppError::Forbidden(format!(
                "Cieľový súbor {} už existuje s iným obsahom.",
                final_path.display()
            )));
        }
        fs::remove_file(&part_path)?;
    } else {
        fs::rename(&part_path, &final_path)?;
    }
    write_sidecar(&catalog, &final_path, &hash)?;
    connection.execute(
        "UPDATE downloads SET status='completed', bytes_downloaded=?1, sha256=?2 WHERE id=?3",
        rusqlite::params![catalog.file_size as i64, hash, download_id],
    )?;
    connection.execute(
        "INSERT INTO settings(key, value_json, updated_at)
         VALUES ('download_directory', ?1, CURRENT_TIMESTAMP)
         ON CONFLICT(key) DO UPDATE SET value_json=excluded.value_json, updated_at=CURRENT_TIMESTAMP",
        [serde_json::to_string(&target_directory.display().to_string())
            .map_err(|error| AppError::InvalidInput(error.to_string()))?],
    )?;
    log(app_data, &format!("FINISH {} sha256={hash}", catalog.id))?;
    emit_progress(app, &catalog, catalog.file_size, "completed")?;
    Ok(DownloadedCatalogGame {
        catalog,
        path: final_path,
        sha256: hash,
    })
}

pub fn open_catalog_target(game_id: &str, target: &str) -> AppResult<()> {
    let item = catalog()
        .into_iter()
        .find(|item| item.id == game_id)
        .ok_or_else(|| AppError::NotFound(format!("Katalógová hra {game_id}")))?;
    let url = match target {
        "license" => item.license_url,
        "source" => item.source_url,
        _ => return Err(AppError::InvalidInput("Neznámy katalógový odkaz.".into())),
    };
    let parsed = validate_download_url(&url)?;
    if parsed.host_str() != Some("github.com") {
        return Err(AppError::Forbidden(
            "Katalógové informácie sa otvárajú iba z GitHubu.".into(),
        ));
    }
    Command::new("explorer.exe").arg(parsed.as_str()).spawn()?;
    Ok(())
}

fn perform_download(catalog: &CatalogGame, part_path: &Path, app: &AppHandle) -> AppResult<String> {
    let url = download_url(catalog)?;
    let client = Client::builder()
        .user_agent("RetroBox-Desktop/0.1 curated legal catalog")
        .redirect(Policy::limited(3))
        .connect_timeout(Duration::from_secs(10))
        .timeout(Duration::from_secs(180))
        .build()
        .map_err(http_error)?;
    let mut response = client
        .get(url)
        .send()
        .map_err(http_error)?
        .error_for_status()
        .map_err(http_error)?;
    let final_host = response.url().host_str().unwrap_or_default();
    if response.url().scheme() != "https"
        || !matches!(
            final_host,
            "raw.githubusercontent.com" | "release-assets.githubusercontent.com"
        )
    {
        return Err(AppError::Forbidden(
            "Katalógový súbor bol presmerovaný na nepovolený server.".into(),
        ));
    }
    if response
        .content_length()
        .is_some_and(|length| length != catalog.file_size)
    {
        return Err(AppError::Forbidden(
            "Server vrátil inú veľkosť než overený katalóg.".into(),
        ));
    }
    let mime = response
        .headers()
        .get(reqwest::header::CONTENT_TYPE)
        .and_then(|value| value.to_str().ok())
        .unwrap_or("");
    if mime.contains("text/html") {
        return Err(AppError::Forbidden(
            "Server namiesto hry vrátil HTML stránku.".into(),
        ));
    }

    let mut writer = BufWriter::new(File::create(part_path)?);
    let mut hasher = Sha256::new();
    let mut total = 0_u64;
    let mut signature = Vec::with_capacity(336);
    let mut buffer = [0_u8; 64 * 1024];
    loop {
        let read = response.read(&mut buffer)?;
        if read == 0 {
            break;
        }
        total = total.saturating_add(read as u64);
        if total > catalog.file_size {
            return Err(AppError::Forbidden(
                "Sťahovaný súbor prekročil overenú veľkosť.".into(),
            ));
        }
        if signature.len() < 336 {
            let remaining = 336 - signature.len();
            signature.extend_from_slice(&buffer[..read.min(remaining)]);
        }
        hasher.update(&buffer[..read]);
        writer.write_all(&buffer[..read])?;
        emit_progress(app, catalog, total, "downloading")?;
    }
    writer.flush()?;
    drop(writer);
    if total != catalog.file_size {
        return Err(AppError::Forbidden(format!(
            "Neúplné stiahnutie: {total} z {} bajtov.",
            catalog.file_size
        )));
    }
    validate_signature(&catalog.file_name, &signature)?;
    let hash = hex::encode(hasher.finalize());
    if hash != catalog.sha256 {
        return Err(AppError::Forbidden(
            "SHA-256 stiahnutého súboru sa nezhoduje s katalógom.".into(),
        ));
    }
    Ok(hash)
}

fn validate_catalog_item(item: &CatalogGame) -> AppResult<()> {
    if !matches!(
        item.license.as_str(),
        "CC0-1.0" | "Unlicense" | "GPL-3.0" | "Unlicense / CC BY-SA 3.0 hudba"
    ) {
        return Err(AppError::Forbidden(
            "Položka nemá povolenú redistribučnú licenciu.".into(),
        ));
    }
    if item.file_size == 0 || item.file_size > 64 * 1024 * 1024 {
        return Err(AppError::Forbidden(
            "Položka prekračuje limit katalógu.".into(),
        ));
    }
    if item.sha256.len() != 64 || !item.sha256.bytes().all(|value| value.is_ascii_hexdigit()) {
        return Err(AppError::InvalidInput(
            "Katalóg obsahuje neplatný SHA-256.".into(),
        ));
    }
    let extension = Path::new(&item.file_name)
        .extension()
        .and_then(|value| value.to_str())
        .unwrap_or("");
    if !matches!(extension, "nes" | "gb") {
        return Err(AppError::Forbidden(
            "Katalóg obsahuje nepovolený typ súboru.".into(),
        ));
    }
    Ok(())
}

fn validate_signature(file_name: &str, bytes: &[u8]) -> AppResult<()> {
    let extension = Path::new(file_name)
        .extension()
        .and_then(|value| value.to_str())
        .unwrap_or("");
    let valid = match extension {
        "nes" => bytes.starts_with(b"NES\x1a"),
        "gb" => bytes.len() > 0x133 && bytes[0x104..0x108] == [0xce, 0xed, 0x66, 0x66],
        _ => false,
    };
    if valid {
        Ok(())
    } else {
        Err(AppError::Forbidden(
            "Obsah súboru nezodpovedá deklarovanému hernému formátu.".into(),
        ))
    }
}

fn download_url(catalog: &CatalogGame) -> AppResult<Url> {
    let source = match catalog.id.as_str() {
        "gruniozerca" => "https://raw.githubusercontent.com/arhneu/gruniozerca/42e94aa976af2230b43876f8c7aebd34a70a9149/grunio.nes",
        "rainbow-chat" => "https://raw.githubusercontent.com/BrokeStudio/rainbow-chat/9d08aa2a7667acc67eaac6cb75562fc5445a9304/cc65/roms/rainbow-chat.nes",
        "gbclock" => "https://raw.githubusercontent.com/kresp0/gbclock/f4de4a9be6265810471e69cca62a2bc025242286/build/rom/gbclock_v0.3.gb",
        "2048-nes" => "https://raw.githubusercontent.com/mmuszkow/2048-nes/d0b2841384b7881619e53863d5bb7a27263ccccd/2048.nes",
        "big2small" => "https://github.com/mdsteele/big2small/releases/download/v1.0.0/big2small.gb",
        _ => return Err(AppError::Forbidden("Neznámy zdroj katalógu.".into())),
    };
    validate_download_url(source)
}

fn write_sidecar(catalog: &CatalogGame, game_path: &Path, hash: &str) -> AppResult<()> {
    let sidecar = CatalogSidecar {
        title: &catalog.title,
        description: &catalog.description,
        system_id: &catalog.system_id,
        developer: &catalog.developer,
        genre: &catalog.genre,
        license: &catalog.license,
        license_url: &catalog.license_url,
        source_url: &catalog.source_url,
        sha256: hash,
        file_size: catalog.file_size,
    };
    let path = PathBuf::from(format!("{}.meta.json", game_path.display()));
    let bytes = serde_json::to_vec_pretty(&sidecar)
        .map_err(|error| AppError::InvalidInput(error.to_string()))?;
    fs::write(path, bytes)?;
    Ok(())
}

fn sha256_file(path: &Path) -> AppResult<String> {
    let mut reader = File::open(path)?;
    let mut hasher = Sha256::new();
    let mut buffer = [0_u8; 64 * 1024];
    loop {
        let read = reader.read(&mut buffer)?;
        if read == 0 {
            break;
        }
        hasher.update(&buffer[..read]);
    }
    Ok(hex::encode(hasher.finalize()))
}

fn emit_progress(
    app: &AppHandle,
    catalog: &CatalogGame,
    downloaded: u64,
    status: &str,
) -> AppResult<()> {
    app.emit(
        "catalog-download-progress",
        CatalogDownloadProgress {
            game_id: catalog.id.clone(),
            bytes_downloaded: downloaded,
            total_bytes: catalog.file_size,
            status: status.into(),
        },
    )
    .map_err(|error| AppError::InvalidInput(format!("Udalosť sťahovania: {error}")))
}

fn log(app_data: &Path, message: &str) -> AppResult<()> {
    let directory = app_data.join("logs");
    fs::create_dir_all(&directory)?;
    let mut file = OpenOptions::new()
        .create(true)
        .append(true)
        .open(directory.join("downloads.log"))?;
    writeln!(file, "{} {message}", Utc::now().to_rfc3339())?;
    Ok(())
}

fn http_error(error: reqwest::Error) -> AppError {
    AppError::InvalidInput(format!("Download HTTP chyba: {error}"))
}

pub fn provider_for(input: &str) -> AppResult<Provider> {
    let url = validate_download_url(input)?;
    let host = url.host_str().unwrap_or_default().to_ascii_lowercase();
    if host.ends_with("drive.google.com") {
        return Ok(Provider::GoogleDrivePublic);
    }
    if host.ends_with("dropbox.com") || host.ends_with("dropboxusercontent.com") {
        return Ok(Provider::DropboxPublic);
    }
    if host.ends_with("1drv.ms") || host.ends_with("onedrive.live.com") {
        return Ok(Provider::OneDrivePublic);
    }
    Ok(Provider::DirectHttp)
}

pub fn resolve_public_url(input: &str) -> AppResult<Url> {
    let mut url = validate_download_url(input)?;
    match provider_for(input)? {
        Provider::DropboxPublic => {
            url.query_pairs_mut().clear().append_pair("dl", "1");
        }
        Provider::GoogleDrivePublic => {
            let id = url
                .path_segments()
                .and_then(|mut segments| {
                    let values: Vec<_> = segments.by_ref().collect();
                    values
                        .windows(2)
                        .find(|pair| pair[0] == "d")
                        .map(|pair| pair[1].to_owned())
                })
                .or_else(|| {
                    url.query_pairs()
                        .find(|(key, _)| key == "id")
                        .map(|(_, value)| value.into_owned())
                })
                .ok_or_else(|| {
                    AppError::InvalidInput("Odkaz nevedie na priamo dostupný herný súbor.".into())
                })?;
            url = Url::parse(&format!(
                "https://drive.usercontent.google.com/download?id={id}&export=download"
            ))
            .map_err(|_| AppError::InvalidInput("Neplatný Google Drive odkaz".into()))?;
        }
        Provider::OneDrivePublic | Provider::DirectHttp => {}
    }
    Ok(url)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn curated_catalog_has_fixed_legal_artifacts() {
        let items = catalog();
        assert_eq!(items.len(), 5);
        for item in items {
            validate_catalog_item(&item).unwrap();
            assert!(matches!(
                download_url(&item).unwrap().host_str(),
                Some("raw.githubusercontent.com") | Some("github.com")
            ));
        }
    }

    #[test]
    fn validates_known_rom_signatures() {
        assert!(validate_signature("game.nes", b"NES\x1a1234").is_ok());
        let mut gb = vec![0; 0x134];
        gb[0x104..0x108].copy_from_slice(&[0xce, 0xed, 0x66, 0x66]);
        assert!(validate_signature("game.gb", &gb).is_ok());
        assert!(validate_signature("game.nes", b"<html>").is_err());
    }

    #[test]
    fn detects_providers() {
        assert_eq!(
            provider_for("https://drive.google.com/file/d/abc/view").unwrap(),
            Provider::GoogleDrivePublic
        );
        assert_eq!(
            provider_for("https://example.com/game.zip").unwrap(),
            Provider::DirectHttp
        );
    }

    #[test]
    fn rewrites_dropbox() {
        let url = resolve_public_url("https://www.dropbox.com/s/abc/game.zip?dl=0").unwrap();
        assert!(url.as_str().ends_with("?dl=1"));
    }

    #[test]
    fn parses_google_drive_file_id() {
        let url = resolve_public_url("https://drive.google.com/file/d/abc123/view").unwrap();
        assert!(url.as_str().contains("id=abc123"));
    }
}
