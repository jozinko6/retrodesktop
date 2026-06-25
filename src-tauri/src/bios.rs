use std::{
    fs,
    fs::File,
    io::{BufReader, Read},
    path::Path,
};

use sha2::{Digest, Sha256};

use crate::{
    domain::BiosImportResult,
    error::{AppError, AppResult},
    security::sanitize_filename,
};

pub fn import(emulator_id: &str, source: &Path, app_data: &Path) -> AppResult<BiosImportResult> {
    let canonical = source
        .canonicalize()
        .map_err(|_| AppError::NotFound(source.display().to_string()))?;
    let metadata = canonical.metadata()?;
    validate(emulator_id, &canonical, metadata.len())?;
    let mut reader = BufReader::new(File::open(&canonical)?);
    let mut hasher = Sha256::new();
    let mut buffer = [0_u8; 64 * 1024];
    loop {
        let read = reader.read(&mut buffer)?;
        if read == 0 {
            break;
        }
        hasher.update(&buffer[..read]);
    }
    let sha256 = hex::encode(hasher.finalize());
    let filename = sanitize_filename(
        canonical
            .file_name()
            .and_then(|value| value.to_str())
            .unwrap_or("bios.bin"),
    );
    let directory = app_data.join("system").join(emulator_id);
    fs::create_dir_all(&directory)?;
    let destination = directory.join(filename);
    if destination.exists() {
        return Err(AppError::Forbidden(
            "BIOS už existuje. Odstráň ho alebo potvrď prepísanie v budúcom BIOS Manageri.".into(),
        ));
    }
    let temporary = destination.with_extension("part");
    fs::copy(&canonical, &temporary)?;
    fs::rename(temporary, &destination)?;
    Ok(BiosImportResult {
        emulator_id: emulator_id.into(),
        stored_path: destination.display().to_string(),
        sha256,
        size: metadata.len(),
    })
}

fn validate(emulator_id: &str, path: &Path, size: u64) -> AppResult<()> {
    let extension = path
        .extension()
        .and_then(|value| value.to_str())
        .unwrap_or("")
        .to_ascii_lowercase();
    let valid = match emulator_id {
        "pcsx2" | "duckstation" | "retroarch" => {
            matches!(extension.as_str(), "bin" | "rom" | "mec" | "nvm") && size <= 64 * 1024 * 1024
        }
        "rpcs3" => extension == "pup" && size <= 512 * 1024 * 1024,
        _ => false,
    };
    if !valid {
        return Err(AppError::InvalidInput(
            "Súbor nemá podporovaný BIOS/firmware formát alebo veľkosť pre tento emulátor.".into(),
        ));
    }
    Ok(())
}

pub fn is_configured(emulator_id: &str, app_data: &Path) -> bool {
    let directory = app_data.join("system").join(emulator_id);
    directory.read_dir().ok().is_some_and(|mut entries| {
        entries.any(|entry| entry.ok().is_some_and(|item| item.path().is_file()))
    })
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn validates_expected_formats() {
        assert!(validate("pcsx2", Path::new("bios.bin"), 4 * 1024 * 1024).is_ok());
        assert!(validate("rpcs3", Path::new("PS3UPDAT.PUP"), 200 * 1024 * 1024).is_ok());
        assert!(validate("pcsx2", Path::new("installer.exe"), 10).is_err());
    }
}
