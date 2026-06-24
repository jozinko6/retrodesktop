use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct Game {
    pub id: String,
    pub title: String,
    pub system_id: String,
    pub primary_file: String,
    pub description: String,
    pub release_year: Option<i32>,
    pub developer: Option<String>,
    pub genre: Option<String>,
    pub total_play_time_seconds: i64,
    pub last_played_at: Option<String>,
    pub favorite: bool,
    pub accent: String,
    pub short_review: Option<String>,
    pub metadata_source: Option<String>,
    pub cover_path: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
#[serde(rename_all = "camelCase")]
pub struct DetectionCandidate {
    pub system_id: String,
    pub confidence: f32,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
#[serde(rename_all = "camelCase")]
pub struct DetectionResult {
    pub system_id: Option<String>,
    pub confidence: f32,
    pub evidence: Vec<String>,
    pub candidates: Vec<DetectionCandidate>,
}

#[derive(Debug, Clone, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct EmulatorStatus {
    pub id: String,
    pub display_name: String,
    pub state: String,
    pub version: Option<String>,
    pub executable: Option<String>,
    pub supported_systems: Vec<String>,
    pub can_managed_install: bool,
    pub official_url: String,
    pub bios_required: bool,
    pub bios_configured: bool,
}

#[derive(Debug, Clone, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct InstallResult {
    pub emulator_id: String,
    pub version: String,
    pub executable: String,
    pub sha256: String,
}

#[derive(Debug, Clone, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct BiosImportResult {
    pub emulator_id: String,
    pub stored_path: String,
    pub sha256: String,
    pub size: u64,
}

#[derive(Debug, Clone, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct RemotePlayStatus {
    pub installed: bool,
    pub running: bool,
    pub executable: Option<String>,
    pub local_ip: Option<String>,
    pub web_ui_url: String,
    pub port: u16,
}

#[derive(Debug, Clone)]
pub struct LaunchCommand {
    pub executable: std::path::PathBuf,
    pub args: Vec<String>,
    pub working_directory: Option<std::path::PathBuf>,
}

#[derive(Debug, Clone, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct LaunchResult {
    pub session_id: String,
    pub exit_code: Option<i32>,
    pub duration_seconds: i64,
}

#[derive(Debug, Clone, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct ResolvedDownload {
    pub provider: String,
    pub url: String,
    pub log_safe_url: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct WindowsGameCandidate {
    pub id: String,
    pub source: String,
    pub source_id: Option<String>,
    pub title: String,
    pub install_path: Option<String>,
    pub launch_kind: String,
    pub launch_target: String,
    pub launch_arguments: Vec<String>,
    pub working_directory: Option<String>,
    pub confidence: f32,
    pub evidence: Vec<String>,
}

#[derive(Debug, Clone, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct WindowsDiscoveryResult {
    pub candidates: Vec<WindowsGameCandidate>,
    pub scanned_sources: Vec<String>,
    pub warnings: Vec<String>,
}
