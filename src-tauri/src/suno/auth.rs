use base64::{engine::general_purpose::STANDARD, Engine as _};
use reqwest::header::{HeaderMap, HeaderValue, AUTHORIZATION, ORIGIN, REFERER};
use reqwest::Client;

use crate::config::paths::API_BASE;
use crate::suno::types::SessionData;

#[derive(Debug, thiserror::Error)]
pub enum AuthError {
    #[error("{0}")]
    Message(String),
}

pub fn build_browser_token() -> String {
    let payload = STANDARD.encode(format!(
        r#"{{"timestamp":{}}}"#,
        chrono::Utc::now().timestamp_millis()
    ));
    format!(r#"{{"token":"{payload}"}}"#)
}

pub fn build_headers(session: &SessionData) -> anyhow::Result<HeaderMap> {
    let mut headers = HeaderMap::new();
    headers.insert(
        AUTHORIZATION,
        HeaderValue::from_str(&format!("Bearer {}", session.jwt))?,
    );
    headers.insert(
        "device-id",
        HeaderValue::from_str(&session.device_id)?,
    );
    headers.insert(
        "browser-token",
        HeaderValue::from_str(&build_browser_token())?,
    );
    headers.insert(ORIGIN, HeaderValue::from_static("https://suno.com"));
    headers.insert(REFERER, HeaderValue::from_static("https://suno.com/"));
    Ok(headers)
}

pub async fn verify_session(jwt: &str, device_id: &str) -> anyhow::Result<()> {
    let session = SessionData {
        jwt: jwt.to_string(),
        device_id: device_id.to_string(),
        storage_state: None,
        saved_at: String::new(),
    };

    let client = Client::new();
    let response = client
        .get(format!("{API_BASE}/api/feed/?page=1"))
        .headers(build_headers(&session)?)
        .send()
        .await?;

    if response.status() == reqwest::StatusCode::UNAUTHORIZED {
        anyhow::bail!("Token is invalid or expired.");
    }

    if !response.status().is_success() {
        anyhow::bail!(
            "Token verification failed: {} {}",
            response.status(),
            response.status().canonical_reason().unwrap_or("")
        );
    }

    Ok(())
}
