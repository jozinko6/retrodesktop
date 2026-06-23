use url::Url;

use crate::{
    error::{AppError, AppResult},
    security::validate_download_url,
};

#[derive(Debug, PartialEq)]
pub enum Provider {
    DirectHttp,
    GoogleDrivePublic,
    DropboxPublic,
    OneDrivePublic,
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
