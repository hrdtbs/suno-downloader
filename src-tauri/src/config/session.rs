use serde::{Deserialize, Serialize};

use crate::config::paths::{
    cli_session_path, ensure_config_dir, legacy_session_path, session_path,
};
use crate::suno::types::SessionData;

#[derive(Debug, thiserror::Error)]
pub enum SessionError {
    #[error("{0}")]
    Message(String),
}

impl SessionError {
    pub fn auth(message: impl Into<String>) -> Self {
        Self::Message(message.into())
    }
}

async fn read_session_file(path: &std::path::Path) -> anyhow::Result<Option<SessionData>> {
    match tokio::fs::read_to_string(path).await {
        Ok(raw) => Ok(Some(serde_json::from_str(&raw)?)),
        Err(error) if error.kind() == std::io::ErrorKind::NotFound => Ok(None),
        Err(error) => Err(error.into()),
    }
}

pub async fn load_session() -> Result<SessionData, SessionError> {
    let session = read_session_file(&session_path())
        .await
        .map_err(|error| SessionError::Message(error.to_string()))?
        .or(read_session_file(&cli_session_path())
            .await
            .map_err(|error| SessionError::Message(error.to_string()))?)
        .or(read_session_file(&legacy_session_path())
            .await
            .map_err(|error| SessionError::Message(error.to_string()))?);

    let Some(session) = session else {
        return Err(SessionError::auth(
            "セッションが見つかりません。認証を行ってください。",
        ));
    };

    if session.jwt.trim().is_empty() || session.device_id.trim().is_empty() {
        return Err(SessionError::auth(
            "セッションファイルが無効です。再度認証してください。",
        ));
    }

    Ok(session)
}

pub async fn try_load_session() -> Option<SessionData> {
    load_session().await.ok()
}

pub async fn save_session(jwt: &str, device_id: &str) -> anyhow::Result<()> {
    let _ = ensure_config_dir().await?;
    let data = SessionData {
        jwt: jwt.to_string(),
        device_id: device_id.to_string(),
        storage_state: None,
        saved_at: chrono::Utc::now().to_rfc3339(),
    };
    tokio::fs::write(session_path(), serde_json::to_string_pretty(&data)?).await?;
    Ok(())
}

pub async fn delete_session() -> anyhow::Result<()> {
    for path in [session_path(), cli_session_path(), legacy_session_path()] {
        match tokio::fs::remove_file(&path).await {
            Ok(()) => {}
            Err(error) if error.kind() == std::io::ErrorKind::NotFound => {}
            Err(error) => return Err(error.into()),
        }
    }
    Ok(())
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AuthStatus {
    pub authenticated: bool,
    pub saved_at: Option<String>,
    pub token_server_running: bool,
    pub token_server_url: String,
}

pub fn normalize_token(raw: &str) -> String {
    let mut token = raw.trim().to_string();
    token.retain(|ch| ch.is_ascii());
    let lower = token.to_lowercase();
    if let Some(rest) = lower.strip_prefix("bearer ") {
        return token[rest.len()..].trim().to_string();
    }
    token
}
