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

use std::{fs, path::PathBuf};

use domain::{DetectionResult, EmulatorStatus, Game, LaunchResult, ResolvedDownload};
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
    let session_id = uuid::Uuid::new_v4().to_string();
    connection.execute(
        "INSERT INTO play_sessions(id, game_id, emulator_id, started_at)
         VALUES (?1, ?2, ?3, CURRENT_TIMESTAMP)",
        rusqlite::params![session_id, game_id, emulator],
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
            launch_game
        ])
        .run(tauri::generate_context!())
        .expect("error while running RetroBox Desktop");
}
