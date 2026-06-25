mod bios;
mod db;
mod detection;
mod domain;
mod download;
mod emulators;
mod error;
mod library;
mod managed_install;
mod metadata;
mod remote_play;
mod runner;
mod security;
mod state;
mod windows_discovery;

use std::{fs, path::PathBuf};

use domain::{
    BiosImportResult, CatalogDownloadResult, CatalogGame, DetectionResult, EmulatorStatus, Game,
    InstallResult, LaunchCommand, LaunchResult, RemotePlayStatus, ResolvedDownload,
    WindowsDiscoveryResult, WindowsGameCandidate,
};
use error::{AppError, AppResult};
use state::AppState;
use tauri::{AppHandle, Manager, State};

fn app_data_dir() -> PathBuf {
    dirs::data_local_dir()
        .unwrap_or_else(|| PathBuf::from("."))
        .join("RetroBox")
}

fn ensure_directories() -> AppResult<()> {
    let root = app_data_dir();
    for path in [
        "config",
        "cores",
        "core-info",
        "shaders",
        "saves",
        "states",
        "screenshots",
        "system",
        "playlists",
        "logs",
        "media",
        "downloads",
        "emulators",
        "backups",
    ] {
        fs::create_dir_all(root.join(path))?;
    }
    Ok(())
}

#[tauri::command]
fn list_games(state: State<'_, AppState>) -> AppResult<Vec<Game>> {
    db::games(&db::open(&state.database_path)?)
}

#[tauri::command]
fn scan_directory(path: String, state: State<'_, AppState>) -> AppResult<Vec<Game>> {
    let canonical = PathBuf::from(&path)
        .canonicalize()
        .map_err(|_| AppError::NotFound(path))?;
    library::scan(&db::open(&state.database_path)?, &canonical)
}

#[tauri::command]
fn import_game_file(
    path: String,
    system_id: String,
    state: State<'_, AppState>,
) -> AppResult<Vec<Game>> {
    let canonical = PathBuf::from(&path)
        .canonicalize()
        .map_err(|_| AppError::NotFound(path))?;
    let connection = db::open(&state.database_path)?;
    let game = library::import_file(&connection, &canonical, Some(&system_id))?;
    let _ = enrich_game(&connection, &game.id, &game.title, &system_id);
    db::games(&connection)
}

#[tauri::command]
fn scan_directory_for_system(
    path: String,
    system_id: String,
    state: State<'_, AppState>,
) -> AppResult<Vec<Game>> {
    let canonical = PathBuf::from(&path)
        .canonicalize()
        .map_err(|_| AppError::NotFound(path))?;
    let connection = db::open(&state.database_path)?;
    let imported = library::scan_for_system(&connection, &canonical, Some(&system_id))?;
    for game in imported.iter().take(20) {
        let _ = enrich_game(&connection, &game.id, &game.title, &system_id);
    }
    db::games(&connection)
}

fn enrich_game(
    connection: &rusqlite::Connection,
    game_id: &str,
    title: &str,
    system_id: &str,
) -> AppResult<()> {
    let system_name = connection
        .query_row(
            "SELECT display_name FROM systems WHERE id=?1",
            [system_id],
            |row| row.get::<_, String>(0),
        )
        .unwrap_or_else(|_| system_id.to_owned());
    let Some(metadata) = metadata::fetch(title, &system_name)? else {
        return Ok(());
    };
    connection.execute(
        "UPDATE games SET title=?1, sort_title=?1, description=?2, short_review=?3,
         release_year=COALESCE(?4, release_year), metadata_source=?5,
         updated_at=CURRENT_TIMESTAMP WHERE id=?6",
        rusqlite::params![
            &metadata.title,
            &metadata.description,
            &metadata.short_review,
            metadata.release_year,
            &metadata.source,
            game_id
        ],
    )?;
    if let Some(cover_url) = metadata.cover_url {
        let media_directory = app_data_dir().join("media").join(system_id).join(game_id);
        if let Some(cover_path) = metadata::download_cover(&cover_url, &media_directory)? {
            connection.execute(
                "DELETE FROM game_assets WHERE game_id=?1 AND kind='cover'",
                [game_id],
            )?;
            connection.execute(
                "INSERT INTO game_assets(id, game_id, kind, relative_path, source)
                 VALUES (?1, ?2, 'cover', ?3, ?4)",
                rusqlite::params![
                    uuid::Uuid::new_v4().to_string(),
                    game_id,
                    cover_path.display().to_string(),
                    &metadata.source
                ],
            )?;
        }
    }
    Ok(())
}

#[tauri::command]
fn refresh_game_metadata(game_id: String, state: State<'_, AppState>) -> AppResult<Vec<Game>> {
    let connection = db::open(&state.database_path)?;
    let (title, system_id) = connection
        .query_row(
            "SELECT title, system_id FROM games WHERE id=?1",
            [&game_id],
            |row| Ok((row.get::<_, String>(0)?, row.get::<_, String>(1)?)),
        )
        .map_err(|_| AppError::NotFound(game_id.clone()))?;
    enrich_game(&connection, &game_id, &title, &system_id)?;
    db::games(&connection)
}

#[tauri::command]
fn detect_platform(path: String) -> AppResult<DetectionResult> {
    detection::detect(&PathBuf::from(path))
}

#[tauri::command]
fn list_emulators(state: State<'_, AppState>) -> AppResult<Vec<EmulatorStatus>> {
    let connection = db::open(&state.database_path)?;
    let mut statuses = emulators::statuses();
    for status in &mut statuses {
        status.can_managed_install = managed_install::can_install(&status.id);
        status.official_url = managed_install::official_url(&status.id).into();
        status.bios_required = managed_install::bios_required(&status.id);
        status.bios_configured = bios::is_configured(&status.id, &app_data_dir());
        let configured = connection.query_row(
            "SELECT executable, status, version FROM emulator_installations
             WHERE emulator_id=?1 ORDER BY detected_at DESC LIMIT 1",
            [&status.id],
            |row| {
                Ok((
                    row.get::<_, String>(0)?,
                    row.get::<_, String>(1)?,
                    row.get::<_, Option<String>>(2)?,
                ))
            },
        );
        if let Ok((executable, installation_status, version)) = configured {
            status.executable = Some(executable);
            status.state = installation_status;
            status.version = version;
        }
    }
    Ok(statuses)
}

#[tauri::command]
fn configure_emulator(
    emulator_id: String,
    executable: String,
    state: State<'_, AppState>,
) -> AppResult<EmulatorStatus> {
    let executable = emulators::validate_executable(&emulator_id, &PathBuf::from(executable))?;
    let connection = db::open(&state.database_path)?;
    connection.execute(
        "DELETE FROM emulator_installations WHERE emulator_id=?1",
        [&emulator_id],
    )?;
    connection.execute(
        "INSERT INTO emulator_installations
         (id, emulator_id, executable, managed, status, detected_at)
         VALUES (?1, ?2, ?3, 0, 'ready', CURRENT_TIMESTAMP)",
        rusqlite::params![
            uuid::Uuid::new_v4().to_string(),
            emulator_id,
            executable.display().to_string()
        ],
    )?;
    let adapter = emulators::adapter(&emulator_id)?;
    Ok(EmulatorStatus {
        id: emulator_id.clone(),
        display_name: adapter.display_name().into(),
        state: "ready".into(),
        version: None,
        executable: Some(executable.display().to_string()),
        supported_systems: adapter
            .supported_systems()
            .iter()
            .map(|value| (*value).into())
            .collect(),
        can_managed_install: managed_install::can_install(&emulator_id),
        official_url: managed_install::official_url(&emulator_id).into(),
        bios_required: managed_install::bios_required(&emulator_id),
        bios_configured: bios::is_configured(&emulator_id, &app_data_dir()),
    })
}

#[tauri::command]
fn install_managed_emulator(
    emulator_id: String,
    state: State<'_, AppState>,
) -> AppResult<InstallResult> {
    if !managed_install::can_install(&emulator_id) {
        return Err(AppError::InvalidInput(
            "Tento emulátor zatiaľ nemá bezpečný automatický release resolver.".into(),
        ));
    }
    let result = managed_install::install(&emulator_id, &app_data_dir())?;
    let executable =
        emulators::validate_executable(&emulator_id, &PathBuf::from(&result.executable))?;
    let mut connection = db::open(&state.database_path)?;
    let transaction = connection.transaction()?;
    transaction.execute(
        "DELETE FROM emulator_installations WHERE emulator_id=?1",
        [&emulator_id],
    )?;
    transaction.execute(
        "INSERT INTO emulator_installations
         (id, emulator_id, executable, version, managed, status, detected_at)
         VALUES (?1, ?2, ?3, ?4, 1, 'ready', CURRENT_TIMESTAMP)",
        rusqlite::params![
            uuid::Uuid::new_v4().to_string(),
            emulator_id,
            executable.display().to_string(),
            result.version
        ],
    )?;
    transaction.commit()?;
    Ok(result)
}

#[tauri::command]
fn import_bios(
    emulator_id: String,
    source_path: String,
    state: State<'_, AppState>,
) -> AppResult<BiosImportResult> {
    let root = app_data_dir();
    let result = bios::import(&emulator_id, &PathBuf::from(source_path), &root)?;
    let stored = PathBuf::from(&result.stored_path);
    let relative = stored
        .strip_prefix(&root)
        .map_err(|_| AppError::Forbidden("BIOS bol uložený mimo dátového priečinka.".into()))?;
    let connection = db::open(&state.database_path)?;
    connection.execute(
        "INSERT INTO bios_files(id, system_id, relative_path, size, sha256, status)
         VALUES (?1, ?2, ?3, ?4, ?5, 'verified')",
        rusqlite::params![
            uuid::Uuid::new_v4().to_string(),
            emulator_id,
            relative.display().to_string(),
            result.size,
            result.sha256
        ],
    )?;
    Ok(result)
}

#[tauri::command]
fn open_official_emulator_page(emulator_id: String) -> AppResult<()> {
    let url = managed_install::official_url(&emulator_id);
    if url.is_empty() {
        return Err(AppError::InvalidInput("Neznámy emulátor.".into()));
    }
    std::process::Command::new("explorer.exe")
        .arg(url)
        .spawn()
        .map_err(AppError::Io)?;
    Ok(())
}

#[tauri::command]
fn remote_play_status() -> RemotePlayStatus {
    remote_play::status()
}

#[tauri::command]
fn start_remote_play_host() -> AppResult<RemotePlayStatus> {
    remote_play::start()
}

#[tauri::command]
fn open_remote_play_target(target: String) -> AppResult<()> {
    remote_play::open_target(&target)
}

#[tauri::command]
fn configure_retroarch_game(
    game_id: String,
    core_path: String,
    state: State<'_, AppState>,
) -> AppResult<()> {
    let core = PathBuf::from(&core_path)
        .canonicalize()
        .map_err(|_| AppError::NotFound(core_path))?;
    if core.extension().and_then(|value| value.to_str()) != Some("dll") {
        return Err(AppError::InvalidInput(
            "RetroArch core musí byť Windows DLL súbor.".into(),
        ));
    }
    let connection = db::open(&state.database_path)?;
    let system_id: String = connection
        .query_row(
            "SELECT system_id FROM games WHERE id=?1",
            [&game_id],
            |row| row.get(0),
        )
        .map_err(|_| AppError::NotFound(game_id.clone()))?;
    let core_id = format!(
        "custom:{}",
        core.file_stem()
            .and_then(|value| value.to_str())
            .unwrap_or("core")
    );
    connection.execute(
        "INSERT INTO retroarch_cores
         (id, system_id, display_name, library_filename_windows, supported_extensions, priority, requires_bios)
         VALUES (?1, ?2, ?3, ?4, '', 100, 0)
         ON CONFLICT(id) DO UPDATE SET library_filename_windows=excluded.library_filename_windows",
        rusqlite::params![
            core_id,
            system_id,
            core.file_stem().and_then(|value| value.to_str()).unwrap_or("Core"),
            core.display().to_string()
        ],
    )?;
    connection.execute(
        "UPDATE games SET emulator_id='retroarch', core_id=?1, updated_at=CURRENT_TIMESTAMP WHERE id=?2",
        rusqlite::params![core_id, game_id],
    )?;
    Ok(())
}

#[tauri::command]
fn resolve_download_url(url: String) -> AppResult<ResolvedDownload> {
    let provider = download::provider_for(&url)?;
    let resolved = download::resolve_public_url(&url)?;
    Ok(ResolvedDownload {
        provider: format!("{provider:?}"),
        url: resolved.to_string(),
        log_safe_url: security::redact_url(resolved.as_str()),
    })
}

#[tauri::command]
fn fetch_catalog() -> Vec<CatalogGame> {
    download::catalog()
}

#[tauri::command]
fn get_download_directory(state: State<'_, AppState>) -> AppResult<Option<String>> {
    let connection = db::open(&state.database_path)?;
    let value = connection.query_row(
        "SELECT value_json FROM settings WHERE key='download_directory'",
        [],
        |row| row.get::<_, String>(0),
    );
    match value {
        Ok(value) => serde_json::from_str(&value)
            .map(Some)
            .map_err(|error| AppError::InvalidInput(format!("Nastavenie priečinka: {error}"))),
        Err(rusqlite::Error::QueryReturnedNoRows) => Ok(None),
        Err(error) => Err(error.into()),
    }
}

#[tauri::command]
fn request_catalog_download(
    game_id: String,
    target_directory: String,
    state: State<'_, AppState>,
    app: AppHandle,
) -> AppResult<CatalogDownloadResult> {
    let connection = db::open(&state.database_path)?;
    let downloaded = download::download_catalog_game(
        &game_id,
        &PathBuf::from(target_directory),
        &app_data_dir(),
        &connection,
        &app,
    )?;
    let imported = library::import_file(
        &connection,
        &downloaded.path,
        Some(&downloaded.catalog.system_id),
    )?;
    connection.execute(
        "UPDATE games SET title=?1, sort_title=?1, description=?2, developer=?3,
         genre=?4, short_review=?2, metadata_source=?5, file_hash=?6,
         file_size=?7, updated_at=CURRENT_TIMESTAMP WHERE id=?8",
        rusqlite::params![
            &downloaded.catalog.title,
            &downloaded.catalog.description,
            &downloaded.catalog.developer,
            &downloaded.catalog.genre,
            &downloaded.catalog.source_url,
            &downloaded.sha256,
            downloaded.catalog.file_size as i64,
            &imported.id
        ],
    )?;
    connection.execute(
        "UPDATE game_files SET sha256=?1, size=?2 WHERE game_id=?3 AND role='primary'",
        rusqlite::params![
            &downloaded.sha256,
            downloaded.catalog.file_size as i64,
            &imported.id
        ],
    )?;
    let game = db::games(&connection)?
        .into_iter()
        .find(|game| game.id == imported.id)
        .ok_or_else(|| AppError::NotFound(imported.id))?;
    Ok(CatalogDownloadResult {
        game,
        sha256: downloaded.sha256,
        stored_path: downloaded.path.display().to_string(),
    })
}

#[tauri::command]
fn open_catalog_target(game_id: String, target: String) -> AppResult<()> {
    download::open_catalog_target(&game_id, &target)
}

#[tauri::command]
fn safe_filename(name: String) -> String {
    security::sanitize_filename(&name)
}

#[tauri::command]
fn scan_windows_games(
    roots: Vec<String>,
    state: State<'_, AppState>,
) -> AppResult<WindowsDiscoveryResult> {
    let roots = roots
        .into_iter()
        .map(PathBuf::from)
        .filter_map(|path| path.canonicalize().ok())
        .collect::<Vec<_>>();
    let result = windows_discovery::discover(&roots);
    let connection = db::open(&state.database_path)?;
    for item in &result.candidates {
        connection.execute(
            "INSERT INTO windows_game_candidates (
              id, source, source_id, title, install_path, launch_kind, launch_target,
              launch_arguments_json, working_directory, confidence, evidence_json, status
            ) VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7, ?8, ?9, ?10, ?11, 'pending')
            ON CONFLICT(source, source_id, launch_target) DO UPDATE SET
              title=excluded.title, install_path=excluded.install_path,
              launch_kind=excluded.launch_kind,
              launch_arguments_json=excluded.launch_arguments_json,
              working_directory=excluded.working_directory,
              confidence=excluded.confidence, evidence_json=excluded.evidence_json,
              discovered_at=CURRENT_TIMESTAMP,
              status=CASE
                WHEN windows_game_candidates.status='accepted' THEN 'accepted'
                ELSE 'pending'
              END",
            rusqlite::params![
                item.id,
                item.source,
                item.source_id,
                item.title,
                item.install_path,
                item.launch_kind,
                item.launch_target,
                serde_json::to_string(&item.launch_arguments)
                    .map_err(|error| AppError::InvalidInput(error.to_string()))?,
                item.working_directory,
                item.confidence,
                serde_json::to_string(&item.evidence)
                    .map_err(|error| AppError::InvalidInput(error.to_string()))?
            ],
        )?;
    }
    Ok(result)
}

#[tauri::command]
fn list_windows_candidates(state: State<'_, AppState>) -> AppResult<Vec<WindowsGameCandidate>> {
    let connection = db::open(&state.database_path)?;
    let mut statement = connection.prepare(
        "SELECT id, source, source_id, title, install_path, launch_kind, launch_target,
         launch_arguments_json, working_directory, confidence, evidence_json
         FROM windows_game_candidates WHERE status='pending'
         ORDER BY confidence DESC, title COLLATE NOCASE",
    )?;
    let rows = statement.query_map([], |row| {
        let arguments: String = row.get(7)?;
        let evidence: String = row.get(10)?;
        Ok(WindowsGameCandidate {
            id: row.get(0)?,
            source: row.get(1)?,
            source_id: row.get(2)?,
            title: row.get(3)?,
            install_path: row.get(4)?,
            launch_kind: row.get(5)?,
            launch_target: row.get(6)?,
            launch_arguments: serde_json::from_str(&arguments).unwrap_or_default(),
            working_directory: row.get(8)?,
            confidence: row.get(9)?,
            evidence: serde_json::from_str(&evidence).unwrap_or_default(),
        })
    })?;
    Ok(rows.collect::<Result<Vec<_>, _>>()?)
}

#[tauri::command]
fn confirm_windows_games(
    candidate_ids: Vec<String>,
    state: State<'_, AppState>,
) -> AppResult<Vec<Game>> {
    let mut connection = db::open(&state.database_path)?;
    let transaction = connection.transaction()?;
    for candidate_id in candidate_ids {
        let candidate = transaction
            .query_row(
                "SELECT source, source_id, title, install_path, launch_kind, launch_target,
                 launch_arguments_json, working_directory
                 FROM windows_game_candidates WHERE id=?1 AND status='pending'",
                [&candidate_id],
                |row| {
                    Ok((
                        row.get::<_, String>(0)?,
                        row.get::<_, Option<String>>(1)?,
                        row.get::<_, String>(2)?,
                        row.get::<_, Option<String>>(3)?,
                        row.get::<_, String>(4)?,
                        row.get::<_, String>(5)?,
                        row.get::<_, String>(6)?,
                        row.get::<_, Option<String>>(7)?,
                    ))
                },
            )
            .map_err(|_| AppError::NotFound(candidate_id.clone()))?;
        let game_id = uuid::Uuid::new_v4().to_string();
        let primary = candidate.3.clone().unwrap_or_else(|| candidate.5.clone());
        transaction.execute(
            "INSERT INTO games (
              id, title, sort_title, system_id, primary_file, launch_file, description,
              genre, total_play_time_seconds, favorite, accent, created_at, updated_at
            ) VALUES (?1, ?2, ?2, 'windows', ?3, ?4, ?5, 'Windows', 0, 0, '#39a0ff',
              CURRENT_TIMESTAMP, CURRENT_TIMESTAMP)
            ON CONFLICT(primary_file) DO UPDATE SET title=excluded.title, updated_at=CURRENT_TIMESTAMP",
            rusqlite::params![
                game_id,
                candidate.2,
                primary,
                candidate.5,
                format!("Nájdené cez {}. Metadata čakajú na scraping.", candidate.0)
            ],
        )?;
        let stable_game_id: String = transaction.query_row(
            "SELECT id FROM games WHERE primary_file=?1",
            [&primary],
            |row| row.get(0),
        )?;
        transaction.execute(
            "INSERT INTO game_launches (
              game_id, launch_kind, launch_target, launch_arguments_json,
              working_directory, source, source_id
            ) VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7)
            ON CONFLICT(game_id) DO UPDATE SET
              launch_kind=excluded.launch_kind, launch_target=excluded.launch_target,
              launch_arguments_json=excluded.launch_arguments_json,
              working_directory=excluded.working_directory, source=excluded.source,
              source_id=excluded.source_id",
            rusqlite::params![
                stable_game_id,
                candidate.4,
                candidate.5,
                candidate.6,
                candidate.7,
                candidate.0,
                candidate.1
            ],
        )?;
        transaction.execute(
            "UPDATE windows_game_candidates SET status='accepted' WHERE id=?1",
            [&candidate_id],
        )?;
    }
    transaction.commit()?;
    db::games(&connection)
}

#[tauri::command]
fn reject_windows_games(candidate_ids: Vec<String>, state: State<'_, AppState>) -> AppResult<()> {
    let connection = db::open(&state.database_path)?;
    for id in candidate_ids {
        connection.execute(
            "UPDATE windows_game_candidates SET status='rejected' WHERE id=?1",
            [id],
        )?;
    }
    Ok(())
}

#[tauri::command]
fn launch_game(
    game_id: String,
    app: tauri::AppHandle,
    state: State<'_, AppState>,
) -> AppResult<LaunchResult> {
    let _guard = state
        .launch_lock
        .try_lock()
        .map_err(|_| AppError::Forbidden("Hra sa už spúšťa".into()))?;
    let connection = db::open(&state.database_path)?;
    let windows_launch = connection.query_row(
        "SELECT launch_kind, launch_target, launch_arguments_json, working_directory
         FROM game_launches WHERE game_id=?1",
        [&game_id],
        |row| {
            Ok((
                row.get::<_, String>(0)?,
                row.get::<_, String>(1)?,
                row.get::<_, String>(2)?,
                row.get::<_, Option<String>>(3)?,
            ))
        },
    );
    if let Ok((kind, target, arguments_json, working_directory)) = windows_launch {
        let arguments: Vec<String> = serde_json::from_str(&arguments_json)
            .map_err(|error| AppError::InvalidInput(error.to_string()))?;
        let command = match kind.as_str() {
            "exe" => {
                let executable = PathBuf::from(&target)
                    .canonicalize()
                    .map_err(|_| AppError::NotFound(target.clone()))?;
                LaunchCommand {
                    working_directory: working_directory
                        .map(PathBuf::from)
                        .or_else(|| executable.parent().map(std::path::Path::to_path_buf)),
                    executable,
                    args: arguments,
                }
            }
            "uri" => {
                let uri = url::Url::parse(&target)
                    .map_err(|_| AppError::InvalidInput("Neplatné launch URI".into()))?;
                if !matches!(uri.scheme(), "steam" | "com.epicgames.launcher" | "shell") {
                    return Err(AppError::Forbidden(format!(
                        "Nepovolená launch URI schéma: {}",
                        uri.scheme()
                    )));
                }
                LaunchCommand {
                    executable: PathBuf::from("explorer.exe"),
                    args: vec![target],
                    working_directory: None,
                }
            }
            _ => {
                return Err(AppError::InvalidInput(format!(
                    "Nepodporovaný Windows launch typ: {kind}"
                )))
            }
        };
        return execute_launch(&connection, &game_id, "windows", command, &app);
    }

    let mut statement = connection.prepare(
        "SELECT g.primary_file, g.system_id, g.emulator_id, c.library_filename_windows
         FROM games g LEFT JOIN retroarch_cores c ON c.id=g.core_id WHERE g.id=?1",
    )?;
    let (file, system_id, configured_emulator, core): (
        String,
        String,
        Option<String>,
        Option<String>,
    ) = statement
        .query_row([&game_id], |row| {
            Ok((row.get(0)?, row.get(1)?, row.get(2)?, row.get(3)?))
        })
        .map_err(|_| AppError::NotFound(game_id.clone()))?;
    let emulator = configured_emulator
        .unwrap_or_else(|| emulators::preferred_for_system(&system_id).to_owned());
    connection.execute(
        "UPDATE games SET emulator_id=?1, updated_at=CURRENT_TIMESTAMP
         WHERE id=?2 AND emulator_id IS NULL",
        rusqlite::params![emulator, game_id],
    )?;
    let executable: String = connection
        .query_row(
            "SELECT executable FROM emulator_installations
             WHERE emulator_id=?1 AND status='ready' ORDER BY detected_at DESC LIMIT 1",
            [&emulator],
            |row| row.get(0),
        )
        .map_err(|_| {
            AppError::InvalidInput(format!(
                "Pre systém {system_id} je potrebný emulátor {emulator}. Nainštaluj alebo nastav ho v Správcovi emulátorov."
            ))
        })?;
    let launch_file =
        library::prepare_launch_file(&game_id, &PathBuf::from(file), &system_id, &app_data_dir())?;
    let adapter = emulators::adapter(&emulator)?;
    let command = adapter.build_launch_command(
        &PathBuf::from(executable),
        &launch_file,
        core.as_deref().map(std::path::Path::new),
    )?;
    execute_launch(&connection, &game_id, &emulator, command, &app)
}

fn execute_launch(
    connection: &rusqlite::Connection,
    game_id: &str,
    emulator_id: &str,
    command: LaunchCommand,
    app: &tauri::AppHandle,
) -> AppResult<LaunchResult> {
    let session_id = uuid::Uuid::new_v4().to_string();
    connection.execute(
        "INSERT INTO play_sessions(id, game_id, emulator_id, started_at)
         VALUES (?1, ?2, ?3, CURRENT_TIMESTAMP)",
        rusqlite::params![session_id, game_id, emulator_id],
    )?;
    if let Some(window) = app.get_webview_window("main") {
        let _ = window.minimize();
    }
    let outcome = runner::run(&command)?;
    let duration_seconds = outcome.duration_seconds;
    fs::write(
        app_data_dir()
            .join("logs")
            .join(format!("launch-{session_id}.log")),
        format!("stdout:\n{}\n\nstderr:\n{}", outcome.stdout, outcome.stderr),
    )?;
    connection.execute(
        "UPDATE play_sessions SET ended_at=CURRENT_TIMESTAMP, duration_seconds=?1, exit_code=?2 WHERE id=?3",
        rusqlite::params![duration_seconds, outcome.exit_code, session_id],
    )?;
    connection.execute(
        "UPDATE games SET total_play_time_seconds=total_play_time_seconds+?1,
         last_played_at=CURRENT_TIMESTAMP, updated_at=CURRENT_TIMESTAMP WHERE id=?2",
        rusqlite::params![duration_seconds, game_id],
    )?;
    if let Some(window) = app.get_webview_window("main") {
        let _ = window.unminimize();
        let _ = window.show();
        let _ = window.set_focus();
    }
    Ok(LaunchResult {
        session_id,
        exit_code: outcome.exit_code,
        duration_seconds,
    })
}

pub fn run() {
    ensure_directories().expect("cannot create RetroBox directories");
    let database_path = app_data_dir().join("retrobox.sqlite3");
    db::open(&database_path).expect("database migration failed");

    tauri::Builder::default()
        .plugin(tauri_plugin_dialog::init())
        .plugin(tauri_plugin_single_instance::init(|app, _, _| {
            if let Some(window) = app.get_webview_window("main") {
                let _ = window.show();
                let _ = window.set_focus();
            }
        }))
        .manage(AppState {
            database_path,
            launch_lock: std::sync::Mutex::new(()),
        })
        .invoke_handler(tauri::generate_handler![
            list_games,
            scan_directory,
            import_game_file,
            scan_directory_for_system,
            refresh_game_metadata,
            detect_platform,
            list_emulators,
            configure_emulator,
            install_managed_emulator,
            import_bios,
            open_official_emulator_page,
            remote_play_status,
            start_remote_play_host,
            open_remote_play_target,
            configure_retroarch_game,
            resolve_download_url,
            fetch_catalog,
            get_download_directory,
            request_catalog_download,
            open_catalog_target,
            safe_filename,
            scan_windows_games,
            list_windows_candidates,
            confirm_windows_games,
            reject_windows_games,
            launch_game
        ])
        .run(tauri::generate_context!())
        .expect("error while running RetroBox Desktop");
}
