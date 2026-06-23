use std::{fs::File, io::Read, path::Path};

use crate::{
    domain::{DetectionCandidate, DetectionResult},
    error::{AppError, AppResult},
};

fn extension_system(extension: &str) -> Option<(&'static str, f32)> {
    Some(match extension {
        "nes" => ("nes", 0.92),
        "sfc" | "smc" => ("snes", 0.88),
        "gb" => ("gb", 0.91),
        "gbc" => ("gbc", 0.91),
        "gba" => ("gba", 0.92),
        "n64" | "z64" | "v64" => ("n64", 0.88),
        "nds" => ("nds", 0.92),
        "md" | "gen" => ("genesis", 0.82),
        "sms" => ("mastersystem", 0.9),
        "gg" => ("gamegear", 0.9),
        "a26" => ("atari2600", 0.9),
        "a52" => ("atari5200", 0.9),
        "a78" => ("atari7800", 0.9),
        "lnx" => ("lynx", 0.9),
        "pbp" => ("ps1", 0.72),
        "cso" => ("psp", 0.82),
        "cue" | "chd" => ("ps1", 0.62),
        "iso" => ("unknown", 0.35),
        "zip" | "7z" | "rar" => ("unknown", 0.25),
        "exe" | "bat" | "com" => ("dos", 0.68),
        "gcz" | "rvz" | "wbfs" => ("gamecube", 0.72),
        "adf" => ("amiga", 0.88),
        "d64" => ("c64", 0.88),
        "jsdos" => ("dos", 0.9),
        _ => return None,
    })
}

pub fn detect(path: &Path) -> AppResult<DetectionResult> {
    if !path.exists() {
        return Err(AppError::NotFound(path.display().to_string()));
    }
    let extension = path
        .extension()
        .and_then(|item| item.to_str())
        .unwrap_or("")
        .to_ascii_lowercase();
    let mut evidence = Vec::new();
    let mut candidates = Vec::new();
    if let Some((system, confidence)) = extension_system(&extension) {
        evidence.push(format!("Prípona .{extension}"));
        candidates.push(DetectionCandidate {
            system_id: system.into(),
            confidence,
        });
    }
    let mut header = [0_u8; 16];
    if path.is_file() {
        let count = File::open(path)?.read(&mut header)?;
        if count >= 4 && &header[..4] == b"NES\x1a" {
            evidence.push("iNES magic bytes".into());
            candidates.push(DetectionCandidate {
                system_id: "nes".into(),
                confidence: 0.99,
            });
        }
        if count >= 4 && &header[..4] == b"\x24\xff\xae\x51" {
            evidence.push("Nintendo 64 big-endian header".into());
            candidates.push(DetectionCandidate {
                system_id: "n64".into(),
                confidence: 0.99,
            });
        }
    }
    candidates.sort_by(|a, b| b.confidence.total_cmp(&a.confidence));
    candidates.dedup_by(|a, b| a.system_id == b.system_id);
    let best = candidates.first();
    Ok(DetectionResult {
        system_id: best
            .filter(|item| item.confidence >= 0.7)
            .map(|item| item.system_id.clone()),
        confidence: best.map_or(0.0, |item| item.confidence),
        evidence,
        candidates,
    })
}

pub fn parse_cue(contents: &str) -> Vec<String> {
    contents
        .lines()
        .filter_map(|line| {
            let trimmed = line.trim();
            if !trimmed.to_ascii_uppercase().starts_with("FILE ") {
                return None;
            }
            let rest = trimmed[5..].trim();
            if let Some(quoted) = rest.strip_prefix('"') {
                return quoted.split('"').next().map(str::to_owned);
            }
            rest.split_whitespace().next().map(str::to_owned)
        })
        .collect()
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::io::Write;
    use tempfile::NamedTempFile;

    #[test]
    fn magic_bytes_override_filename_guessing() {
        let mut file = NamedTempFile::with_suffix(".bin").unwrap();
        file.write_all(b"NES\x1a synthetic fixture").unwrap();
        let result = detect(file.path()).unwrap();
        assert_eq!(result.system_id.as_deref(), Some("nes"));
        assert_eq!(result.confidence, 0.99);
    }

    #[test]
    fn parses_multibin_cue() {
        let files = parse_cue(
            "FILE \"Track 01.bin\" BINARY\n TRACK 01 MODE2/2352\nFILE \"Track 02.bin\" BINARY",
        );
        assert_eq!(files, vec!["Track 01.bin", "Track 02.bin"]);
    }
}
