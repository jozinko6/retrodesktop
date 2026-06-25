use std::{
    fs::{self, File},
    io::{self, BufReader, BufWriter, Read, Write},
    path::{Path, PathBuf},
    time::Duration,
};

use regex::Regex;
use reqwest::{blocking::Client, redirect::Policy};
use serde::Deserialize;
use sha2::{Digest, Sha256};
use uuid::Uuid;
use walkdir::WalkDir;

use crate::{
    emulators,
    error::{AppError, AppResult},
    security::{safe_child, sanitize_filename},
};

const MAX_DOWNLOAD_BYTES: u64 = 1_073_741_824;
const MAX_EXTRACTED_BYTES: u64 = 2_147_483_648;
const MAX_ARCHIVE_ENTRIES: usize = 20_000;

#[derive(Debug)]
pub struct Package {
    pub version: String,
    pub url: String,
    pub archive_type: &'static str,
}

#[derive(Deserialize)]
struct GithubRelease {
    tag_name: String,
    assets: Vec<GithubAsset>,
}

#[derive(Deserialize)]
struct GithubAsset {
    name: String,
    browser_download_url: String,
}

pub fn can_install(id: &str) -> bool {
    matches!(
        id,
        "retroarch" | "pcsx2" | "ppsspp" | "duckstation" | "cemu"
    )
}

pub fn official_url(id: &str) -> &'static str {
    match id {
        "retroarch" => "https://www.retroarch.com/?page=platforms",
        "pcsx2" => "https://pcsx2.net/downloads/",
        "dolphin" => "https://dolphin-emu.org/download/",
        "ppsspp" => "https://www.ppsspp.org/download/",
        "duckstation" => "https://github.com/stenzek/duckstation/releases",
        "rpcs3" => "https://rpcs3.net/download",
        "cemu" => "https://cemu.info/",
        _ => "",
    }
}

pub fn bios_required(id: &str) -> bool {
    matches!(id, "retroarch" | "pcsx2" | "duckstation" | "rpcs3")
}

pub fn resolve(id: &str) -> AppResult<Package> {
    let client = client()?;
    match id {
        "retroarch" => {
            let page = client
                .get("https://www.retroarch.com/?page=platforms")
                .send()
                .map_err(http_error)?
                .error_for_status()
                .map_err(http_error)?
                .text()
                .map_err(http_error)?;
            let version = Regex::new(r"current stable version is:\s*([0-9.]+)")
                .expect("valid regex")
                .captures(&page)
                .and_then(|capture| capture.get(1))
                .map(|value| value.as_str().to_owned())
                .ok_or_else(|| AppError::InvalidInput("Nepodarilo sa zistiť stabilnú verziu RetroArchu.".into()))?;
            Ok(Package {
                url: format!("https://buildbot.libretro.com/stable/{version}/windows/x86_64/RetroArch.7z"),
                version,
                archive_type: "7z",
            })
        }
        "ppsspp" => {
            let page = client
                .get("https://www.ppsspp.org/")
                .send()
                .map_err(http_error)?
                .error_for_status()
                .map_err(http_error)?
                .text()
                .map_err(http_error)?;
            let version = Regex::new(r"PPSSPP\s+([0-9]+\.[0-9]+\.[0-9]+)")
                .expect("valid regex")
                .captures(&page)
                .and_then(|capture| capture.get(1))
                .map(|value| value.as_str().to_owned())
                .ok_or_else(|| AppError::InvalidInput("Nepodarilo sa zistiť stabilnú verziu PPSSPP.".into()))?;
            Ok(Package {
                url: format!(
                    "https://www.ppsspp.org/files/{}/ppsspp_win.zip",
                    version.replace('.', "_")
                ),
                version,
                archive_type: "zip",
            })
        }
        "pcsx2" => github_package(
            &client,
            "PCSX2/pcsx2",
            None,
            &["windows-x64-Qt.7z"],
            &["symbols", "installer"],
            "7z",
        ),
        "duckstation" => github_package(
            &client,
            "stenzek/duckstation",
            Some("preview"),
            &["duckstation-windows-x64-release.zip"],
            &["symbols", "installer", "sse2"],
            "zip",
        ),
        "cemu" => github_package(
            &client,
            "cemu-project/Cemu",
            None,
            &["windows-x64.zip"],
            &[],
            "zip",
        ),
        _ => Err(AppError::InvalidInput(
            "Pre tento emulátor nie je dostupný stabilný automatický release resolver. Použi oficiálnu stránku alebo ručný výber executable.".into(),
        )),
    }
}

fn github_package(
    client: &Client,
    repository: &str,
    tag: Option<&str>,
    required: &[&str],
    excluded: &[&str],
    archive_type: &'static str,
) -> AppResult<Package> {
    let endpoint = tag.map_or_else(
        || format!("https://api.github.com/repos/{repository}/releases/latest"),
        |tag| format!("https://api.github.com/repos/{repository}/releases/tags/{tag}"),
    );
    let release: GithubRelease = client
        .get(endpoint)
        .send()
        .map_err(http_error)?
        .error_for_status()
        .map_err(http_error)?
        .json()
        .map_err(http_error)?;
    let asset = release
        .assets
        .into_iter()
        .find(|asset| {
            let lower = asset.name.to_ascii_lowercase();
            required
                .iter()
                .all(|part| lower.contains(&part.to_ascii_lowercase()))
                && excluded
                    .iter()
                    .all(|part| !lower.contains(&part.to_ascii_lowercase()))
        })
        .ok_or_else(|| {
            AppError::NotFound(format!("Windows x64 portable asset pre {repository}"))
        })?;
    Ok(Package {
        version: release.tag_name,
        url: asset.browser_download_url,
        archive_type,
    })
}

pub fn install(id: &str, app_data: &Path) -> AppResult<crate::domain::InstallResult> {
    let package = resolve(id)?;
    let root = app_data.join("emulators").join(id);
    fs::create_dir_all(&root)?;
    let download_name =
        sanitize_filename(package.url.rsplit('/').next().unwrap_or("emulator-package"));
    let part_path = root.join(format!("{download_name}.part"));
    let (sha256, _) = download(&package.url, &part_path)?;
    let staging = root.join(format!(".install-{}", Uuid::new_v4()));
    fs::create_dir_all(&staging)?;
    match package.archive_type {
        "zip" => extract_zip(&part_path, &staging)?,
        "7z" => extract_7z(&part_path, &staging)?,
        _ => return Err(AppError::InvalidInput("Nepodporovaný archív".into())),
    }
    let adapter = emulators::adapter(id)?;
    let executable = find_executable(&staging, adapter.executable_names())
        .ok_or_else(|| AppError::NotFound(format!("Executable po rozbalení: {}", id)))?;
    let final_dir = root.join(sanitize_filename(&package.version));
    if final_dir.exists() {
        fs::remove_dir_all(&staging)?;
        let existing =
            find_executable(&final_dir, adapter.executable_names()).ok_or_else(|| {
                AppError::NotFound(format!("Executable v existujúcej inštalácii: {id}"))
            })?;
        fs::remove_file(&part_path)?;
        return Ok(crate::domain::InstallResult {
            emulator_id: id.into(),
            version: package.version,
            executable: existing.display().to_string(),
            sha256,
        });
    }
    fs::rename(&staging, &final_dir)?;
    let relative = executable
        .strip_prefix(&staging)
        .map_err(|_| AppError::Forbidden("Executable mimo staging priečinka".into()))?;
    let final_executable = final_dir.join(relative);
    fs::remove_file(&part_path)?;
    Ok(crate::domain::InstallResult {
        emulator_id: id.into(),
        version: package.version,
        executable: final_executable.display().to_string(),
        sha256,
    })
}

fn client() -> AppResult<Client> {
    Client::builder()
        .user_agent("RetroBox-Desktop/0.1")
        .redirect(Policy::limited(5))
        .connect_timeout(Duration::from_secs(20))
        .timeout(Duration::from_secs(600))
        .build()
        .map_err(http_error)
}

fn http_error(error: reqwest::Error) -> AppError {
    AppError::InvalidInput(format!("HTTP chyba: {error}"))
}

fn download(url: &str, target: &Path) -> AppResult<(String, u64)> {
    let mut response = client()?
        .get(url)
        .send()
        .map_err(http_error)?
        .error_for_status()
        .map_err(http_error)?;
    if let Some(length) = response.content_length() {
        if length > MAX_DOWNLOAD_BYTES {
            return Err(AppError::Forbidden("Balík prekračuje limit 1 GB.".into()));
        }
    }
    let mut writer = BufWriter::new(File::create(target)?);
    let mut hasher = Sha256::new();
    let mut total = 0_u64;
    let mut buffer = [0_u8; 128 * 1024];
    loop {
        let read = response.read(&mut buffer).map_err(AppError::Io)?;
        if read == 0 {
            break;
        }
        total += read as u64;
        if total > MAX_DOWNLOAD_BYTES {
            return Err(AppError::Forbidden("Balík prekračuje limit 1 GB.".into()));
        }
        writer.write_all(&buffer[..read])?;
        hasher.update(&buffer[..read]);
    }
    writer.flush()?;
    Ok((hex::encode(hasher.finalize()), total))
}

fn extract_zip(source: &Path, destination: &Path) -> AppResult<()> {
    let file = File::open(source)?;
    let mut archive = zip::ZipArchive::new(BufReader::new(file))
        .map_err(|error| AppError::InvalidInput(error.to_string()))?;
    if archive.len() > MAX_ARCHIVE_ENTRIES {
        return Err(AppError::Forbidden(
            "Archív obsahuje príliš veľa položiek.".into(),
        ));
    }
    let mut total = 0_u64;
    for index in 0..archive.len() {
        let mut entry = archive
            .by_index(index)
            .map_err(|error| AppError::InvalidInput(error.to_string()))?;
        let relative = entry
            .enclosed_name()
            .ok_or_else(|| AppError::Forbidden("Path traversal v ZIP archíve".into()))?;
        if entry
            .unix_mode()
            .is_some_and(|mode| mode & 0o170000 == 0o120000)
        {
            return Err(AppError::Forbidden("Symlink v ZIP archíve".into()));
        }
        total = total.saturating_add(entry.size());
        if total > MAX_EXTRACTED_BYTES {
            return Err(AppError::Forbidden(
                "Rozbalený archív prekračuje limit 2 GB.".into(),
            ));
        }
        if entry.compressed_size() > 0 && entry.size() / entry.compressed_size() > 200 {
            return Err(AppError::Forbidden(
                "ZIP položka má podozrivý kompresný pomer.".into(),
            ));
        }
        let target = safe_child(destination, &relative)?;
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
        let target = safe_child(&root, Path::new(entry.name())).map_err(|error| {
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
    .map_err(|error| AppError::InvalidInput(format!("7z extrakcia: {error}")))
}

fn find_executable(root: &Path, names: &[&str]) -> Option<PathBuf> {
    WalkDir::new(root)
        .max_depth(5)
        .follow_links(false)
        .into_iter()
        .filter_map(Result::ok)
        .find(|entry| {
            entry.file_type().is_file()
                && entry.file_name().to_str().is_some_and(|name| {
                    names
                        .iter()
                        .any(|expected| expected.eq_ignore_ascii_case(name))
                })
        })
        .map(|entry| entry.into_path())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn managed_capabilities_are_explicit() {
        assert!(can_install("retroarch"));
        assert!(can_install("pcsx2"));
        assert!(!can_install("rpcs3"));
        assert!(!can_install("dolphin"));
    }

    #[test]
    fn official_pages_exist_for_all_adapters() {
        for id in [
            "retroarch",
            "pcsx2",
            "dolphin",
            "ppsspp",
            "duckstation",
            "rpcs3",
            "cemu",
        ] {
            assert!(official_url(id).starts_with("https://"));
        }
    }
}
