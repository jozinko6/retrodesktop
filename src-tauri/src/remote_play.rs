use std::{
    env,
    net::{SocketAddr, TcpStream, UdpSocket},
    path::PathBuf,
    process::Command,
    time::Duration,
};

use crate::{
    domain::RemotePlayStatus,
    error::{AppError, AppResult},
};

const WEB_UI: &str = "https://localhost:47990";
const SUNSHINE_RELEASES: &str = "https://github.com/LizardByte/Sunshine/releases/latest";
const MOONLIGHT_CLIENTS: &str = "https://moonlight-stream.org/";

pub fn status() -> RemotePlayStatus {
    let executable = find_sunshine();
    let running = TcpStream::connect_timeout(
        &SocketAddr::from(([127, 0, 0, 1], 47990)),
        Duration::from_millis(250),
    )
    .is_ok();
    RemotePlayStatus {
        installed: executable.is_some() || running,
        running,
        executable: executable.map(|path| path.display().to_string()),
        local_ip: local_ip(),
        web_ui_url: WEB_UI.into(),
        port: 47990,
    }
}

pub fn start() -> AppResult<RemotePlayStatus> {
    let current = status();
    if current.running {
        return Ok(current);
    }
    let executable = find_sunshine()
        .ok_or_else(|| AppError::NotFound("Sunshine host nie je nainštalovaný.".into()))?;
    Command::new(executable).spawn().map_err(AppError::Io)?;
    for _ in 0..20 {
        std::thread::sleep(Duration::from_millis(250));
        let current = status();
        if current.running {
            return Ok(current);
        }
    }
    Ok(status())
}

pub fn open_target(target: &str) -> AppResult<()> {
    let url = match target {
        "web-ui" => WEB_UI,
        "sunshine-download" => SUNSHINE_RELEASES,
        "moonlight-download" => MOONLIGHT_CLIENTS,
        _ => return Err(AppError::InvalidInput("Neznámy cieľ Remote hrania.".into())),
    };
    Command::new("explorer.exe")
        .arg(url)
        .spawn()
        .map_err(AppError::Io)?;
    Ok(())
}

fn find_sunshine() -> Option<PathBuf> {
    let mut candidates = Vec::new();
    for key in ["ProgramFiles", "ProgramFiles(x86)", "LOCALAPPDATA"] {
        if let Some(root) = env::var_os(key) {
            let root = PathBuf::from(root);
            candidates.extend([
                root.join("Sunshine").join("sunshine.exe"),
                root.join("LizardByte")
                    .join("Sunshine")
                    .join("sunshine.exe"),
                root.join("Programs").join("Sunshine").join("sunshine.exe"),
            ]);
        }
    }
    candidates.into_iter().find(|path| path.is_file())
}

fn local_ip() -> Option<String> {
    let socket = UdpSocket::bind("0.0.0.0:0").ok()?;
    socket.connect("1.1.1.1:80").ok()?;
    let ip = socket.local_addr().ok()?.ip();
    (!ip.is_loopback()).then(|| ip.to_string())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn remote_targets_are_https() {
        assert_eq!(WEB_UI, "https://localhost:47990");
        assert!(SUNSHINE_RELEASES.starts_with("https://github.com/"));
        assert!(MOONLIGHT_CLIENTS.starts_with("https://"));
    }
}
