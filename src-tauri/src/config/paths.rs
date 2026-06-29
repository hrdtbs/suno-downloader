use std::path::PathBuf;

pub const APP_NAME: &str = "suno-downloader";
pub const CLI_APP_NAME: &str = "suno-cli";
pub const LEGACY_APP_NAME: &str = "suno-sync-mini";

pub const API_BASE: &str = "https://studio-api.prod.suno.com";
pub const CDN_BASE: &str = "https://cdn1.suno.ai";

pub const TOKEN_SERVER_HOST: &str = "127.0.0.1";
pub const TOKEN_SERVER_PORT: u16 = 38946;

pub const SESSION_FILENAME: &str = "session.json";
pub const SETTINGS_FILENAME: &str = "settings.json";
pub const INDEX_FILENAME: &str = ".suno-cli-index.json";
pub const LEGACY_INDEX_FILENAME: &str = ".suno-sync-mini-index.json";

pub fn config_dir_for(app_name: &str) -> PathBuf {
    if cfg!(windows) {
        if let Some(app_data) = std::env::var_os("APPDATA") {
            return PathBuf::from(app_data).join(app_name);
        }
    }

    if let Some(xdg) = std::env::var_os("XDG_CONFIG_HOME") {
        return PathBuf::from(xdg).join(app_name);
    }

    dirs::home_dir()
        .unwrap_or_else(|| PathBuf::from("."))
        .join(".config")
        .join(app_name)
}

pub fn config_dir() -> PathBuf {
    config_dir_for(APP_NAME)
}

pub fn cli_config_dir() -> PathBuf {
    config_dir_for(CLI_APP_NAME)
}

pub fn legacy_config_dir() -> PathBuf {
    config_dir_for(LEGACY_APP_NAME)
}

pub fn session_path() -> PathBuf {
    config_dir().join(SESSION_FILENAME)
}

pub fn cli_session_path() -> PathBuf {
    cli_config_dir().join(SESSION_FILENAME)
}

pub fn legacy_session_path() -> PathBuf {
    legacy_config_dir().join(SESSION_FILENAME)
}

pub fn settings_path() -> PathBuf {
    config_dir().join(SETTINGS_FILENAME)
}

pub fn token_server_url() -> String {
    format!("http://{TOKEN_SERVER_HOST}:{TOKEN_SERVER_PORT}")
}

pub async fn ensure_config_dir() -> anyhow::Result<PathBuf> {
    let dir = config_dir();
    tokio::fs::create_dir_all(&dir).await?;
    Ok(dir)
}

pub fn default_output_dir() -> String {
    std::env::var("SUNO_OUTPUT_DIR").unwrap_or_else(|_| "./suno-wav".to_string())
}
