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

pub fn default_max_pages(settings: &AppSettings) -> u32 {
    settings.max_pages.unwrap_or(0)
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::suno::types::OrganizeMode;

    #[test]
    fn deserializes_settings_json_from_frontend() {
        let raw = r#"{
            "output_dir": "C:\\music",
            "organize": "month-week",
            "delay": 10,
            "max_pages": 3,
            "since": "7d"
        }"#;

        let settings: AppSettings = serde_json::from_str(raw).expect("settings json should parse");
        assert_eq!(settings.output_dir.as_deref(), Some("C:\\music"));
        assert_eq!(settings.organize, Some(OrganizeMode::MonthWeek));
        assert_eq!(settings.delay, Some(10));
        assert_eq!(settings.max_pages, Some(3));
        assert_eq!(settings.since.as_deref(), Some("7d"));
    }

    #[test]
    fn roundtrips_settings_json() {
        let settings = AppSettings {
            output_dir: Some("C:\\music".to_string()),
            organize: Some(OrganizeMode::Week),
            delay: Some(5),
            max_pages: Some(0),
            since: None,
        };

        let json = serde_json::to_string(&settings).expect("settings should serialize");
        let restored: AppSettings =
            serde_json::from_str(&json).expect("settings should deserialize");
        assert_eq!(restored.output_dir, settings.output_dir);
        assert_eq!(restored.organize, settings.organize);
        assert_eq!(restored.delay, settings.delay);
        assert_eq!(restored.max_pages, settings.max_pages);
        assert_eq!(restored.since, settings.since);
    }
}
