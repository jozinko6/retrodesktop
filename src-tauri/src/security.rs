use std::path::{Component, Path, PathBuf};

use regex::Regex;
use url::Url;

use crate::error::{AppError, AppResult};

pub fn sanitize_filename(input: &str) -> String {
    let invalid = ['<', '>', ':', '"', '/', '\\', '|', '?', '*'];
    let value: String = input
        .chars()
        .map(|character| if invalid.contains(&character) || character.is_control() { '_' } else { character })
        .collect();
    let value = value.trim().trim_end_matches(['.', ' ']);
    if value.is_empty() { "download".into() } else { value.chars().take(180).collect() }
}

pub fn safe_child(root: &Path, relative: &Path) -> AppResult<PathBuf> {
    if relative.is_absolute()
        || relative.components().any(|component| matches!(component, Component::ParentDir | Component::Prefix(_) | Component::RootDir))
    {
        return Err(AppError::Forbidden("Path traversal".into()));
    }
    Ok(root.join(relative))
}

pub fn validate_download_url(input: &str) -> AppResult<Url> {
    let url = Url::parse(input).map_err(|_| AppError::InvalidInput("Neplatná URL".into()))?;
    if !matches!(url.scheme(), "https" | "http") {
        return Err(AppError::Forbidden("Povolené sú iba HTTP a HTTPS odkazy".into()));
    }
    if !url.username().is_empty() || url.password().is_some() {
        return Err(AppError::Forbidden("URL nesmie obsahovať prihlasovacie údaje".into()));
    }
    Ok(url)
}

pub fn redact_url(input: &str) -> String {
    let token = Regex::new(r"(?i)(token|key|signature|password|auth)=([^&\s]+)").expect("valid regex");
    token.replace_all(input, "$1=[REDACTED]").to_string()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn blocks_unsafe_protocols_and_credentials() {
        assert!(validate_download_url("file:///secret").is_err());
        assert!(validate_download_url("https://user:pass@example.com/game.zip").is_err());
        assert!(validate_download_url("https://example.com/game.zip").is_ok());
    }

    #[test]
    fn sanitizes_windows_names() {
        assert_eq!(sanitize_filename("my:game?.zip"), "my_game_.zip");
    }

    #[test]
    fn blocks_traversal() {
        assert!(safe_child(Path::new("C:/safe"), Path::new("../escape")).is_err());
        assert!(safe_child(Path::new("C:/safe"), Path::new("game/file.bin")).is_ok());
    }

    #[test]
    fn redacts_tokens() {
        assert!(!redact_url("https://x.test/a?token=secret&x=1").contains("secret"));
    }
}
