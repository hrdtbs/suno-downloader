use crate::config::paths::{default_output_dir, ensure_config_dir, settings_path};
use crate::suno::types::{AppSettings, OrganizeMode};

pub async fn load_settings() -> anyhow::Result<AppSettings> {
    let _ = ensure_config_dir().await?;
    match tokio::fs::read_to_string(settings_path()).await {
        Ok(raw) => Ok(serde_json::from_str(&raw).unwrap_or_default()),
        Err(error) if error.kind() == std::io::ErrorKind::NotFound => Ok(AppSettings::default()),
        Err(error) => Err(error.into()),
    }
}

pub async fn save_settings(settings: &AppSettings) -> anyhow::Result<()> {
    let _ = ensure_config_dir().await?;
    tokio::fs::write(settings_path(), serde_json::to_string_pretty(settings)?).await?;
    Ok(())
}

pub async fn resolve_output_dir(explicit: Option<&str>) -> anyhow::Result<String> {
    if let Some(dir) = explicit {
        return Ok(dir.to_string());
    }

    let settings = load_settings().await?;
    if let Some(dir) = settings.output_dir {
        return Ok(dir);
    }

    Ok(default_output_dir())
}

pub fn default_organize(settings: &AppSettings) -> OrganizeMode {
    settings.organize.clone().unwrap_or(OrganizeMode::Week)
}

pub fn default_delay(settings: &AppSettings) -> u32 {
    settings.delay.unwrap_or(5)
}

pub fn default_max_pages(settings: &AppSettings) -> u32 {
    settings.max_pages.unwrap_or(0)
}
