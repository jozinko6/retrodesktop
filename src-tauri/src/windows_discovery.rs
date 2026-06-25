use std::{
    collections::{HashMap, HashSet},
    env, fs,
    path::{Path, PathBuf},
    process::Command,
};

use regex::Regex;
use serde_json::Value;
use uuid::Uuid;
use walkdir::WalkDir;

use crate::{
    domain::{WindowsDiscoveryResult, WindowsGameCandidate},
    error::{AppError, AppResult},
};

const PORTABLE_MAX_DEPTH: usize = 4;
const PORTABLE_MAX_ENTRIES: usize = 20_000;

pub fn discover(extra_roots: &[PathBuf]) -> WindowsDiscoveryResult {
    let mut warnings = Vec::new();
    let mut scanned_sources = Vec::new();
    let mut candidates = Vec::new();

    if let Some(steam) = steam_root() {
        scanned_sources.push("Steam".into());
        match discover_steam(&steam) {
            Ok(items) => candidates.extend(items),
            Err(error) => warnings.push(format!("Steam: {error}")),
        }
    }
    scanned_sources.push("Epic Games".into());
    match discover_epic() {
        Ok(items) => candidates.extend(items),
        Err(error) => warnings.push(format!("Epic Games: {error}")),
    }
    scanned_sources.push("GOG".into());
    match discover_gog() {
        Ok(items) => candidates.extend(items),
        Err(error) => warnings.push(format!("GOG: {error}")),
    }
    scanned_sources.push("Windows Registry".into());
    match discover_installed_programs() {
        Ok(items) => candidates.extend(items),
        Err(error) => warnings.push(format!("Registry: {error}")),
    }
    scanned_sources.push("Windows skratky".into());
    match discover_shortcuts() {
        Ok(items) => candidates.extend(items),
        Err(error) => warnings.push(format!("Skratky: {error}")),
    }
    if !extra_roots.is_empty() {
        scanned_sources.push("Portable hry".into());
        for root in extra_roots {
            match discover_portable(root) {
                Ok(items) => candidates.extend(items),
                Err(error) => warnings.push(format!("{}: {error}", root.display())),
            }
        }
    }

    WindowsDiscoveryResult {
        candidates: deduplicate(candidates),
        scanned_sources,
        warnings,
    }
}

#[allow(clippy::too_many_arguments)]
fn candidate(
    source: &str,
    source_id: Option<String>,
    title: String,
    install_path: Option<PathBuf>,
    launch_kind: &str,
    launch_target: String,
    launch_arguments: Vec<String>,
    working_directory: Option<PathBuf>,
    confidence: f32,
    evidence: Vec<String>,
) -> WindowsGameCandidate {
    WindowsGameCandidate {
        id: Uuid::new_v4().to_string(),
        source: source.into(),
        source_id,
        title,
        install_path: install_path.map(|path| path.display().to_string()),
        launch_kind: launch_kind.into(),
        launch_target,
        launch_arguments,
        working_directory: working_directory.map(|path| path.display().to_string()),
        confidence,
        evidence,
    }
}

fn steam_root() -> Option<PathBuf> {
    let candidates = [
        env::var_os("ProgramFiles(x86)").map(|root| PathBuf::from(root).join("Steam")),
        env::var_os("ProgramFiles").map(|root| PathBuf::from(root).join("Steam")),
        env::var_os("LOCALAPPDATA").map(|root| PathBuf::from(root).join("Steam")),
        steam_registry_path(),
    ];
    candidates.into_iter().flatten().find(|path| path.is_dir())
}

#[cfg(windows)]
fn steam_registry_path() -> Option<PathBuf> {
    use winreg::{enums::HKEY_CURRENT_USER, RegKey};

    let current_user = RegKey::predef(HKEY_CURRENT_USER);
    let key = current_user.open_subkey(r"Software\Valve\Steam").ok()?;
    key.get_value::<String, _>("SteamPath")
        .ok()
        .map(PathBuf::from)
}

#[cfg(not(windows))]
fn steam_registry_path() -> Option<PathBuf> {
    None
}

fn vdf_value(contents: &str, key: &str) -> Option<String> {
    let expression = Regex::new(&format!(r#""{}"\s+"([^"]+)""#, regex::escape(key))).ok()?;
    expression
        .captures(contents)
        .and_then(|capture| capture.get(1))
        .map(|value| value.as_str().replace(r"\\", r"\"))
}

fn steam_libraries(root: &Path) -> Vec<PathBuf> {
    let mut libraries = vec![root.to_path_buf()];
    let file = root.join("steamapps").join("libraryfolders.vdf");
    if let Ok(contents) = fs::read_to_string(file) {
        let expression = Regex::new(r#""path"\s+"([^"]+)""#).expect("valid regex");
        libraries.extend(expression.captures_iter(&contents).filter_map(|capture| {
            capture
                .get(1)
                .map(|value| PathBuf::from(value.as_str().replace(r"\\", r"\")))
        }));
    }
    libraries
}

fn discover_steam(root: &Path) -> AppResult<Vec<WindowsGameCandidate>> {
    let mut result = Vec::new();
    for library in steam_libraries(root) {
        let steamapps = library.join("steamapps");
        if !steamapps.is_dir() {
            continue;
        }
        for entry in fs::read_dir(&steamapps)? {
            let path = entry?.path();
            let filename = path
                .file_name()
                .and_then(|value| value.to_str())
                .unwrap_or("");
            if !filename.starts_with("appmanifest_")
                || path.extension().and_then(|value| value.to_str()) != Some("acf")
            {
                continue;
            }
            let contents = fs::read_to_string(&path)?;
            let Some(app_id) = vdf_value(&contents, "appid") else {
                continue;
            };
            let Some(name) = vdf_value(&contents, "name") else {
                continue;
            };
            let install = vdf_value(&contents, "installdir")
                .map(|folder| steamapps.join("common").join(folder));
            result.push(candidate(
                "steam",
                Some(app_id.clone()),
                name,
                install.clone(),
                "uri",
                format!("steam://rungameid/{app_id}"),
                vec![],
                install,
                1.0,
                vec!["Steam appmanifest".into(), format!("Steam App ID {app_id}")],
            ));
        }
    }
    Ok(result)
}

fn discover_epic() -> AppResult<Vec<WindowsGameCandidate>> {
    let Some(program_data) = env::var_os("PROGRAMDATA") else {
        return Ok(vec![]);
    };
    let manifests = PathBuf::from(program_data)
        .join("Epic")
        .join("EpicGamesLauncher")
        .join("Data")
        .join("Manifests");
    if !manifests.is_dir() {
        return Ok(vec![]);
    }
    let mut result = Vec::new();
    for entry in fs::read_dir(manifests)? {
        let path = entry?.path();
        if path.extension().and_then(|value| value.to_str()) != Some("item") {
            continue;
        }
        let value: Value = serde_json::from_str(&fs::read_to_string(path)?)
            .map_err(|error| AppError::InvalidInput(error.to_string()))?;
        let Some(title) = value["DisplayName"].as_str() else {
            continue;
        };
        let app_name = value["AppName"]
            .as_str()
            .or_else(|| value["MainGameAppName"].as_str());
        let catalog_id = value["CatalogItemId"].as_str();
        let install = value["InstallLocation"].as_str().map(PathBuf::from);
        let executable = value["LaunchExecutable"].as_str();
        let args = value["LaunchCommand"]
            .as_str()
            .map(split_windows_arguments)
            .unwrap_or_default();
        if let (Some(install), Some(executable)) = (&install, executable) {
            let target = install.join(executable);
            if target.is_file() {
                let source_id = app_name
                    .or(catalog_id)
                    .map(str::to_owned)
                    .unwrap_or_else(|| target.display().to_string());
                result.push(candidate(
                    "epic",
                    Some(source_id),
                    title.to_owned(),
                    Some(install.clone()),
                    "exe",
                    target.display().to_string(),
                    args,
                    Some(install.clone()),
                    0.98,
                    vec!["Epic Games manifest".into()],
                ));
            }
        }
    }
    Ok(result)
}

#[cfg(windows)]
fn discover_gog() -> AppResult<Vec<WindowsGameCandidate>> {
    use winreg::{enums::HKEY_LOCAL_MACHINE, RegKey};

    let hklm = RegKey::predef(HKEY_LOCAL_MACHINE);
    let roots = [
        r"SOFTWARE\WOW6432Node\GOG.com\Games",
        r"SOFTWARE\GOG.com\Games",
    ];
    let mut result = Vec::new();
    for root in roots {
        let Ok(key) = hklm.open_subkey(root) else {
            continue;
        };
        for game_id in key.enum_keys().flatten() {
            let Ok(game) = key.open_subkey(&game_id) else {
                continue;
            };
            let title: String = game.get_value("gameName").unwrap_or(game_id.clone());
            let path: String = game.get_value("path").unwrap_or_default();
            let exe: String = game
                .get_value("exe")
                .or_else(|_| game.get_value("launchCommand"))
                .unwrap_or_default();
            if path.is_empty() || exe.is_empty() {
                continue;
            }
            let install = PathBuf::from(path);
            let target = if Path::new(&exe).is_absolute() {
                PathBuf::from(exe)
            } else {
                install.join(exe)
            };
            if target.is_file() {
                result.push(candidate(
                    "gog",
                    Some(game_id.clone()),
                    title,
                    Some(install.clone()),
                    "exe",
                    target.display().to_string(),
                    vec![],
                    Some(install),
                    0.98,
                    vec!["GOG registry".into(), format!("GOG ID {game_id}")],
                ));
            }
        }
    }
    Ok(result)
}

#[cfg(not(windows))]
fn discover_gog() -> AppResult<Vec<WindowsGameCandidate>> {
    Ok(vec![])
}

#[cfg(windows)]
fn discover_installed_programs() -> AppResult<Vec<WindowsGameCandidate>> {
    use winreg::{
        enums::{HKEY_CURRENT_USER, HKEY_LOCAL_MACHINE},
        RegKey,
    };

    let roots = [
        (
            RegKey::predef(HKEY_LOCAL_MACHINE),
            r"SOFTWARE\Microsoft\Windows\CurrentVersion\Uninstall",
        ),
        (
            RegKey::predef(HKEY_LOCAL_MACHINE),
            r"SOFTWARE\WOW6432Node\Microsoft\Windows\CurrentVersion\Uninstall",
        ),
        (
            RegKey::predef(HKEY_CURRENT_USER),
            r"SOFTWARE\Microsoft\Windows\CurrentVersion\Uninstall",
        ),
    ];
    let mut result = Vec::new();
    for (root, path) in roots {
        let Ok(uninstall) = root.open_subkey(path) else {
            continue;
        };
        for key_name in uninstall.enum_keys().flatten() {
            let Ok(key) = uninstall.open_subkey(&key_name) else {
                continue;
            };
            let title: String = key.get_value("DisplayName").unwrap_or_default();
            let install: String = key.get_value("InstallLocation").unwrap_or_default();
            let display_icon: String = key.get_value("DisplayIcon").unwrap_or_default();
            if title.is_empty() || install.is_empty() || display_icon.is_empty() {
                continue;
            }
            let icon_path = display_icon
                .trim()
                .trim_matches('"')
                .split(',')
                .next()
                .unwrap_or_default();
            let executable = PathBuf::from(icon_path);
            let install_path = PathBuf::from(&install);
            if !executable.is_file()
                || executable
                    .extension()
                    .and_then(|value| value.to_str())
                    .map(|value| !value.eq_ignore_ascii_case("exe"))
                    .unwrap_or(true)
                || excluded_executable(icon_path)
            {
                continue;
            }
            let evidence = portable_evidence(&install_path, &executable);
            if evidence.len() < 2 {
                continue;
            }
            result.push(candidate(
                "registry",
                Some(key_name),
                title,
                Some(install_path.clone()),
                "exe",
                executable.display().to_string(),
                vec![],
                Some(install_path),
                0.66,
                [vec!["Windows installed-program registry".into()], evidence].concat(),
            ));
        }
    }
    Ok(result)
}

#[cfg(not(windows))]
fn discover_installed_programs() -> AppResult<Vec<WindowsGameCandidate>> {
    Ok(vec![])
}

fn shortcut_roots() -> Vec<PathBuf> {
    [
        env::var_os("APPDATA")
            .map(|root| PathBuf::from(root).join(r"Microsoft\Windows\Start Menu\Programs")),
        env::var_os("PROGRAMDATA")
            .map(|root| PathBuf::from(root).join(r"Microsoft\Windows\Start Menu\Programs")),
        env::var_os("USERPROFILE").map(|root| PathBuf::from(root).join("Desktop")),
        env::var_os("PUBLIC").map(|root| PathBuf::from(root).join("Desktop")),
    ]
    .into_iter()
    .flatten()
    .filter(|path| path.is_dir())
    .collect()
}

fn discover_shortcuts() -> AppResult<Vec<WindowsGameCandidate>> {
    let script = r#"$s=(New-Object -ComObject WScript.Shell).CreateShortcut($args[0]); [pscustomobject]@{Target=$s.TargetPath;Arguments=$s.Arguments;WorkingDirectory=$s.WorkingDirectory}|ConvertTo-Json -Compress"#;
    let mut result = Vec::new();
    for root in shortcut_roots() {
        for entry in WalkDir::new(root)
            .max_depth(5)
            .follow_links(false)
            .into_iter()
            .filter_map(Result::ok)
        {
            let path = entry.path();
            if !entry.file_type().is_file()
                || path
                    .extension()
                    .and_then(|value| value.to_str())
                    .map(|value| !value.eq_ignore_ascii_case("lnk"))
                    .unwrap_or(true)
            {
                continue;
            }
            let output = Command::new("powershell.exe")
                .args(["-NoProfile", "-NonInteractive", "-Command", script])
                .arg(path)
                .output()?;
            if !output.status.success() {
                continue;
            }
            let Ok(value) = serde_json::from_slice::<Value>(&output.stdout) else {
                continue;
            };
            let Some(target) = value["Target"].as_str() else {
                continue;
            };
            if !target.to_ascii_lowercase().ends_with(".exe") || excluded_executable(target) {
                continue;
            }
            let target_path = PathBuf::from(target);
            if !target_path.is_file() {
                continue;
            }
            let title = path
                .file_stem()
                .and_then(|value| value.to_str())
                .unwrap_or("Windows hra")
                .to_owned();
            let arguments = value["Arguments"]
                .as_str()
                .map(split_windows_arguments)
                .unwrap_or_default();
            let working = value["WorkingDirectory"]
                .as_str()
                .filter(|value| !value.is_empty())
                .map(PathBuf::from);
            result.push(candidate(
                "shortcut",
                Some(path.display().to_string()),
                title,
                target_path.parent().map(Path::to_path_buf),
                "exe",
                target.into(),
                arguments,
                working.or_else(|| target_path.parent().map(Path::to_path_buf)),
                0.72,
                vec!["Windows .lnk skratka".into()],
            ));
        }
    }
    Ok(result)
}

fn discover_portable(root: &Path) -> AppResult<Vec<WindowsGameCandidate>> {
    let canonical = root
        .canonicalize()
        .map_err(|_| AppError::NotFound(root.display().to_string()))?;
    let mut result = Vec::new();
    for entry in WalkDir::new(&canonical)
        .max_depth(PORTABLE_MAX_DEPTH)
        .follow_links(false)
        .into_iter()
        .filter_map(Result::ok)
        .take(PORTABLE_MAX_ENTRIES)
    {
        let path = entry.path();
        if !entry.file_type().is_file()
            || path
                .extension()
                .and_then(|value| value.to_str())
                .map(|value| !value.eq_ignore_ascii_case("exe"))
                .unwrap_or(true)
            || excluded_executable(&path.display().to_string())
        {
            continue;
        }
        let Some(parent) = path.parent() else {
            continue;
        };
        let evidence = portable_evidence(parent, path);
        if evidence.len() < 2 {
            continue;
        }
        let title = path
            .file_stem()
            .and_then(|value| value.to_str())
            .unwrap_or("Portable hra")
            .replace(['_', '-'], " ");
        let launch_target = path.display().to_string();
        result.push(candidate(
            "portable",
            Some(launch_target.clone()),
            title,
            Some(parent.to_path_buf()),
            "exe",
            launch_target,
            vec![],
            Some(parent.to_path_buf()),
            0.55 + (evidence.len().min(4) as f32 * 0.08),
            evidence,
        ));
    }
    Ok(result)
}

fn portable_evidence(directory: &Path, executable: &Path) -> Vec<String> {
    let mut evidence = Vec::new();
    let directory_name = directory
        .file_name()
        .and_then(|value| value.to_str())
        .unwrap_or("")
        .to_ascii_lowercase();
    let path_lower = directory.display().to_string().to_ascii_lowercase();
    if path_lower.contains(r"\games\") || directory_name.contains("game") {
        evidence.push("Priečinok vyzerá ako herná knižnica".into());
    }
    let mut total_size = 0_u64;
    let mut game_files = 0;
    if let Ok(entries) = fs::read_dir(directory) {
        for entry in entries.flatten().take(500) {
            let path = entry.path();
            if let Ok(metadata) = entry.metadata() {
                total_size = total_size.saturating_add(metadata.len());
            }
            let extension = path
                .extension()
                .and_then(|value| value.to_str())
                .unwrap_or("")
                .to_ascii_lowercase();
            if matches!(
                extension.as_str(),
                "pak" | "vpk" | "wad" | "unity3d" | "assets" | "dll"
            ) {
                game_files += 1;
            }
        }
    }
    if total_size >= 100 * 1024 * 1024 {
        evidence.push("Priečinok má aspoň 100 MB dát".into());
    }
    if game_files >= 3 {
        evidence.push("Obsahuje herné dátové archívy alebo DLL".into());
    }
    if executable
        .file_stem()
        .and_then(|value| value.to_str())
        .map(|name| name.len() >= 3)
        .unwrap_or(false)
    {
        evidence.push("Samostatný pomenovaný executable".into());
    }
    evidence
}

fn excluded_executable(value: &str) -> bool {
    let name = Path::new(value)
        .file_stem()
        .and_then(|value| value.to_str())
        .unwrap_or("")
        .to_ascii_lowercase();
    [
        "unins",
        "uninstall",
        "setup",
        "install",
        "crash",
        "report",
        "updater",
        "update",
        "launcher",
        "editor",
        "server",
        "benchmark",
        "config",
    ]
    .iter()
    .any(|blocked| name.contains(blocked))
}

fn split_windows_arguments(value: &str) -> Vec<String> {
    let expression = Regex::new(r#""([^"]*)"|(\S+)"#).expect("valid regex");
    expression
        .captures_iter(value)
        .filter_map(|capture| capture.get(1).or_else(|| capture.get(2)))
        .map(|value| value.as_str().to_owned())
        .collect()
}

fn deduplicate(candidates: Vec<WindowsGameCandidate>) -> Vec<WindowsGameCandidate> {
    let mut by_key: HashMap<String, WindowsGameCandidate> = HashMap::new();
    let mut seen_targets = HashSet::new();
    for item in candidates {
        let target = item.launch_target.to_ascii_lowercase();
        let key = item
            .source_id
            .as_ref()
            .map(|id| format!("{}:{id}", item.source))
            .unwrap_or_else(|| target.clone());
        if seen_targets.contains(&target) {
            continue;
        }
        seen_targets.insert(target);
        by_key
            .entry(key)
            .and_modify(|existing| {
                if item.confidence > existing.confidence {
                    *existing = item.clone();
                }
            })
            .or_insert(item);
    }
    let mut values: Vec<_> = by_key.into_values().collect();
    values.sort_by_key(|item| item.title.to_lowercase());
    values
}

#[cfg(test)]
mod tests {
    use std::fs;

    use tempfile::tempdir;

    use super::*;

    #[test]
    fn parses_vdf_values() {
        let input = r#""AppState" { "appid" "123" "name" "Synthetic Game" }"#;
        assert_eq!(vdf_value(input, "appid").as_deref(), Some("123"));
        assert_eq!(vdf_value(input, "name").as_deref(), Some("Synthetic Game"));
    }

    #[test]
    fn splits_quoted_windows_arguments() {
        assert_eq!(
            split_windows_arguments(r#"-windowed "profile one" --safe"#),
            vec!["-windowed", "profile one", "--safe"]
        );
    }

    #[test]
    fn excludes_helpers() {
        assert!(excluded_executable("C:/Game/CrashReporter.exe"));
        assert!(excluded_executable("C:/Game/unins000.exe"));
        assert!(!excluded_executable("C:/Game/SyntheticGame.exe"));
    }

    #[test]
    fn deduplicates_launch_targets() {
        let first = candidate(
            "portable",
            None,
            "A".into(),
            None,
            "exe",
            "C:/A.exe".into(),
            vec![],
            None,
            0.5,
            vec![],
        );
        let second = candidate(
            "shortcut",
            None,
            "A".into(),
            None,
            "exe",
            "c:/a.exe".into(),
            vec![],
            None,
            0.8,
            vec![],
        );
        let result = deduplicate(vec![first, second]);
        assert_eq!(result.len(), 1);
    }

    #[test]
    fn discovers_synthetic_steam_manifest() {
        let directory = tempdir().unwrap();
        let steamapps = directory.path().join("steamapps");
        fs::create_dir_all(steamapps.join("common").join("Synthetic Game")).unwrap();
        fs::write(
            steamapps.join("appmanifest_4242.acf"),
            r#""AppState"
            {
              "appid" "4242"
              "name" "Synthetic Game"
              "installdir" "Synthetic Game"
            }"#,
        )
        .unwrap();
        let games = discover_steam(directory.path()).unwrap();
        assert_eq!(games.len(), 1);
        assert_eq!(games[0].title, "Synthetic Game");
        assert_eq!(games[0].launch_target, "steam://rungameid/4242");
    }
}
