use std::{path::PathBuf, sync::Mutex};

pub struct AppState {
    pub database_path: PathBuf,
    pub launch_lock: Mutex<()>,
}
