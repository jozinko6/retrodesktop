mod db;
mod detection;
mod domain;
mod download;
mod emulators;
mod error;
mod library;
mod security;
mod state;

use std::{fs, path::PathBuf};

use domain::{DetectionResult, EmulatorStatus, Game};
use error::{AppError, AppResult};
use state::AppState;
use tauri::{Manager, State};

fn app_data_dir() -> PathBuf {
    dirs::data_local_dir().unwrap_or_else(|| PathBuf::from(".")).join("RetroBox")
}

fn ensure_directories() -> AppResult<()> {
    let root = app_data_dir();
    for path in [
        "config", "cores", "core-info", "shaders", "saves", "states", "screenshots",
        "system", "playlists", "logs", "media", "downloads", "emulators", "backups",
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
    let canonical = PathBuf::from(&path).canonicalize().map_err(|_| AppError::NotFound(path))?;
    library::scan(&db::open(&state.database_path)?, &canonical)
}

#[tauri::command]
fn detect_platform(path: String) -> AppResult<DetectionResult> {
    detection::detect(&PathBuf::from(path))
}

#[tauri::command]
fn list_emulators() -> Vec<EmulatorStatus> {
    emulators::statuses()
}

#[tauri::command]
fn launch_game(game_id: String, state: State<'_, AppState>) -> AppResult<()> {
    let _guard = state.launch_lock.try_lock().map_err(|_| AppError::Forbidden("Hra sa už spúšťa".into()))?;
    let connection = db::open(&state.database_path)?;
    let mut statement = connection.prepare("SELECT primary_file, emulator_id FROM games WHERE id=?1")?;
    let (_file, emulator): (String, Option<String>) = statement.query_row([&game_id], |row| Ok((row.get(0)?, row.get(1)?)))
        .map_err(|_| AppError::NotFound(game_id))?;
    if emulator.is_none() {
        return Err(AppError::InvalidInput("Najprv vyberte a nakonfigurujte emulátor pre túto hru.".into()));
    }
    Err(AppError::InvalidInput("Executable cesta emulátora ešte nie je nakonfigurovaná.".into()))
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
        .manage(AppState { database_path, launch_lock: std::sync::Mutex::new(()) })
        .invoke_handler(tauri::generate_handler![list_games, scan_directory, detect_platform, list_emulators, launch_game])
        .run(tauri::generate_context!())
        .expect("error while running RetroBox Desktop");
}
