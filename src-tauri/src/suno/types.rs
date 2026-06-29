use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize, Default, PartialEq, Eq)]
#[serde(rename_all = "kebab-case")]
pub enum OrganizeMode {
    #[default]
    Flat,
    Month,
    Week,
    #[serde(rename = "month-week")]
    MonthWeek,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Clip {
    pub id: String,
    pub title: Option<String>,
    pub created_at: Option<String>,
    pub audio_url: Option<String>,
    pub audio_url_wav: Option<String>,
    pub wav_url: Option<String>,
    pub wav_audio_url: Option<String>,
    pub master_wav_url: Option<String>,
    pub preview_wav_url: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SessionData {
    pub jwt: String,
    #[serde(rename = "deviceId")]
    pub device_id: String,
    #[serde(rename = "storageState", skip_serializing_if = "Option::is_none")]
    pub storage_state: Option<serde_json::Value>,
    #[serde(rename = "savedAt")]
    pub saved_at: String,
}

#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct SyncSummary {
    pub downloaded: u32,
    pub skipped: u32,
    pub filtered: u32,
    pub failed: u32,
    pub remote_count: u32,
    pub pending_count: u32,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SyncOptions {
    pub dir: Option<String>,
    pub delay: Option<u32>,
    pub dry_run: Option<bool>,
    pub max_pages: Option<u32>,
    pub organize: Option<OrganizeMode>,
    pub since: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SyncProgressEvent {
    pub clip_id: String,
    pub title: String,
    pub status: String,
    pub message: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct AppSettings {
    pub output_dir: Option<String>,
    pub organize: Option<OrganizeMode>,
    pub delay: Option<u32>,
    pub max_pages: Option<u32>,
    pub since: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct LibraryClip {
    pub id: String,
    pub title: String,
    pub created_at: Option<String>,
    pub synced: bool,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct LibraryListResult {
    pub clips: Vec<LibraryClip>,
    pub local_count: usize,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TokenServerStatus {
    pub running: bool,
    pub url: String,
    pub port: u16,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SyncPreviewItem {
    pub id: String,
    pub title: String,
    pub display_path: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SyncPreviewResult {
    pub items: Vec<SyncPreviewItem>,
    pub summary: SyncSummary,
}
