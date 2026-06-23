mod db;
mod detection;
mod domain;
mod download;
mod emulators;
mod error;
mod library;
mod runner;
mod security;
mod state;
mod windows_discovery;

use std::{fs, path::PathBuf};

use domain::{
    DetectionResult, EmulatorStatus, Game, LaunchCommand, LaunchResult, ResolvedDownload,
    WindowsDiscoveryResult, WindowsGameCandidate,
};
use error::{AppError, AppResult};
use state::AppState;
use tauri::{Manager, State};

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
fn detect_platform(path: String) -> AppResult<DetectionResult> {
    detection::detect(&PathBuf::from(path))
}

#[tauri::command]
fn list_emulators(state: State<'_, AppState>) -> AppResult<Vec<EmulatorStatus>> {
    let connection = db::open(&state.database_path)?;
    let mut statuses = emulators::statuses();
    for status in &mut statuses {
        let configured = connection.query_row(
            "SELECT executable, status FROM emulator_installations
             WHERE emulator_id=?1 ORDER BY detected_at DESC LIMIT 1",
            [&status.id],
            |row| Ok((row.get::<_, String>(0)?, row.get::<_, String>(1)?)),
        );
        if let Ok((executable, installation_status)) = configured {
            status.executable = Some(executable);
            status.state = installation_status;
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
        id: emulator_id,
        display_name: adapter.display_name().into(),
        state: "ready".into(),
        version: None,
        executable: Some(executable.display().to_string()),
        supported_systems: adapter
            .supported_systems()
            .iter()
            .map(|value| (*value).into())
            .collect(),
    })
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
        "SELECT g.primary_file, g.emulator_id, c.library_filename_windows
         FROM games g LEFT JOIN retroarch_cores c ON c.id=g.core_id WHERE g.id=?1",
    )?;
    let (file, emulator, core): (String, Option<String>, Option<String>) = statement
        .query_row([&game_id], |row| {
            Ok((row.get(0)?, row.get(1)?, row.get(2)?))
        })
        .map_err(|_| AppError::NotFound(game_id.clone()))?;
    let emulator =
        emulator.ok_or_else(|| AppError::InvalidInput("Najprv vyberte emulátor.".into()))?;
    let executable: String = connection
        .query_row(
            "SELECT executable FROM emulator_installations
             WHERE emulator_id=?1 AND status='ready' ORDER BY detected_at DESC LIMIT 1",
            [&emulator],
            |row| row.get(0),
        )
        .map_err(|_| {
            AppError::InvalidInput("Najprv nakonfigurujte executable emulátora.".into())
        })?;
    let adapter = emulators::adapter(&emulator)?;
    let command = adapter.build_launch_command(
        &PathBuf::from(executable),
        &PathBuf::from(file),
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
            detect_platform,
            list_emulators,
            configure_emulator,
            configure_retroarch_game,
            resolve_download_url,
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
