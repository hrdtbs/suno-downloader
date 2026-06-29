use std::collections::HashSet;
use std::path::Path;
use std::time::Duration;

use reqwest::header::{HeaderMap, HeaderValue, ORIGIN, REFERER};
use reqwest::{Client, StatusCode};
use serde::Deserialize;
use serde_json::Value;

use crate::config::date_filters::should_stop_feed_paging;
use crate::config::paths::{API_BASE, CDN_BASE};
use crate::suno::auth::{build_headers, AuthError};
use crate::suno::types::{Clip, SessionData};

const FEED_PAGE_SIZE: u32 = 20;
const WAV_URL_KEYS: [&str; 5] = [
    "audio_url_wav",
    "wav_url",
    "wav_audio_url",
    "master_wav_url",
    "preview_wav_url",
];

#[derive(Debug, Clone)]
pub struct FetchClipsOptions {
    pub created_after: Option<chrono::DateTime<chrono::Utc>>,
}

#[derive(Debug, Deserialize)]
struct FeedV3Response {
    clips: Option<Vec<Clip>>,
    next_cursor: Option<String>,
    has_more: Option<bool>,
}

fn find_wav_url(data: &Value) -> Option<String> {
    match data {
        Value::String(value) => {
            let lower = value.to_lowercase();
            if lower.starts_with("http") && lower.contains(".wav") {
                Some(value.clone())
            } else {
                None
            }
        }
        Value::Array(items) => items.iter().find_map(find_wav_url),
        Value::Object(map) => {
            for key in WAV_URL_KEYS {
                if let Some(Value::String(value)) = map.get(key) {
                    if value.to_lowercase().starts_with("http") {
                        return Some(value.clone());
                    }
                }
            }
            map.values().find_map(find_wav_url)
        }
        _ => None,
    }
}

fn extract_clips(data: &Value) -> Vec<Clip> {
    if let Value::Array(items) = data {
        return serde_json::from_value(Value::Array(items.clone())).unwrap_or_default();
    }

    if let Value::Object(map) = data {
        for key in ["clips", "songs", "data", "items"] {
            if let Some(value) = map.get(key) {
                if let Ok(clips) = serde_json::from_value::<Vec<Clip>>(value.clone()) {
                    return clips;
                }
            }
        }
    }

    Vec::new()
}

async fn api_fetch(
    client: &Client,
    session: &SessionData,
    path: &str,
    method: reqwest::Method,
    body: Option<Value>,
) -> Result<reqwest::Response, AuthError> {
    let url = if path.starts_with("http") {
        path.to_string()
    } else {
        format!("{API_BASE}{path}")
    };

    let mut request = client.request(method, url).headers(build_headers(session).map_err(|error| AuthError::Message(error.to_string()))?);

    if let Some(payload) = body {
        request = request
            .header("Content-Type", "application/json")
            .json(&payload);
    }

    let response = request
        .send()
        .await
        .map_err(|error| AuthError::Message(error.to_string()))?;

    if response.status() == StatusCode::UNAUTHORIZED {
        return Err(AuthError::Message(
            "セッションの有効期限が切れました。再度認証してください。".to_string(),
        ));
    }

    Ok(response)
}

async fn fetch_feed_v3(
    client: &Client,
    session: &SessionData,
    max_pages: u32,
    options: &FetchClipsOptions,
) -> Result<Vec<Clip>, AuthError> {
    let mut cursor: Option<String> = None;
    let mut page = 0_u32;
    let mut seen = HashSet::new();
    let mut clips = Vec::new();

    loop {
        page += 1;
        if max_pages > 0 && page > max_pages {
            break;
        }

        let mut body = serde_json::json!({
            "limit": FEED_PAGE_SIZE,
            "filters": { "trashed": "False" }
        });
        if let Some(value) = &cursor {
            body["cursor"] = Value::String(value.clone());
        }

        let response = api_fetch(
            client,
            session,
            "/api/feed/v3",
            reqwest::Method::POST,
            Some(body),
        )
        .await?;

        if !response.status().is_success() {
            return Err(AuthError::Message(format!(
                "feed v3 failed: {}",
                response.status()
            )));
        }

        let data: FeedV3Response = response
            .json()
            .await
            .map_err(|error| AuthError::Message(error.to_string()))?;

        let page_clips = data.clips.clone().unwrap_or_default();
        if page_clips.is_empty() {
            break;
        }

        for clip in &page_clips {
            if seen.insert(clip.id.clone()) {
                clips.push(clip.clone());
            }
        }

        if let Some(cutoff) = options.created_after {
            if should_stop_feed_paging(&page_clips, &cutoff) {
                break;
            }
        }

        if !data.has_more.unwrap_or(false) || data.next_cursor.is_none() {
            break;
        }

        cursor = data.next_cursor;
    }

    Ok(clips)
}

async fn fetch_feed_v2(
    client: &Client,
    session: &SessionData,
    max_pages: u32,
    options: &FetchClipsOptions,
) -> Result<Vec<Clip>, AuthError> {
    let mut page = 0_u32;
    let mut seen = HashSet::new();
    let mut clips = Vec::new();

    loop {
        if max_pages > 0 && page >= max_pages {
            break;
        }

        let response = api_fetch(
            client,
            session,
            &format!("/api/feed/v2/?page={page}&page_size={FEED_PAGE_SIZE}"),
            reqwest::Method::GET,
            None,
        )
        .await?;

        if !response.status().is_success() {
            return Err(AuthError::Message(format!(
                "feed v2 failed on page {page}: {}",
                response.status()
            )));
        }

        let data: Value = response
            .json()
            .await
            .map_err(|error| AuthError::Message(error.to_string()))?;
        let page_clips = extract_clips(&data);

        if page_clips.is_empty() {
            break;
        }

        for clip in &page_clips {
            if seen.insert(clip.id.clone()) {
                clips.push(clip.clone());
            }
        }

        if let Some(cutoff) = options.created_after {
            if should_stop_feed_paging(&page_clips, &cutoff) {
                break;
            }
        }

        if page_clips.len() < FEED_PAGE_SIZE as usize {
            break;
        }

        page += 1;
    }

    Ok(clips)
}

async fn fetch_feed_v1(
    client: &Client,
    session: &SessionData,
    max_pages: u32,
    options: &FetchClipsOptions,
) -> Result<Vec<Clip>, AuthError> {
    let mut page = 1_u32;
    let mut seen = HashSet::new();
    let mut clips = Vec::new();

    loop {
        if max_pages > 0 && page > max_pages {
            break;
        }

        let response = api_fetch(
            client,
            session,
            &format!("/api/feed/?page={page}"),
            reqwest::Method::GET,
            None,
        )
        .await?;

        if !response.status().is_success() {
            return Err(AuthError::Message(format!(
                "feed v1 failed on page {page}: {}",
                response.status()
            )));
        }

        let data: Value = response
            .json()
            .await
            .map_err(|error| AuthError::Message(error.to_string()))?;
        let page_clips = extract_clips(&data);

        if page_clips.is_empty() {
            break;
        }

        for clip in &page_clips {
            if seen.insert(clip.id.clone()) {
                clips.push(clip.clone());
            }
        }

        if let Some(cutoff) = options.created_after {
            if should_stop_feed_paging(&page_clips, &cutoff) {
                break;
            }
        }

        page += 1;
    }

    Ok(clips)
}

pub async fn fetch_all_clips(
    session: &SessionData,
    max_pages: u32,
    options: &FetchClipsOptions,
) -> Result<Vec<Clip>, AuthError> {
    let client = Client::new();
    let mut seen = HashSet::new();
    let mut merged = Vec::new();
    let mut last_error: Option<AuthError> = None;

    let strategies: [(&str, bool); 3] = [("v2", true), ("v3", true), ("v1", true)];

    for (name, _) in strategies {
        let result = match name {
            "v2" => fetch_feed_v2(&client, session, max_pages, options).await,
            "v3" => fetch_feed_v3(&client, session, max_pages, options).await,
            _ => fetch_feed_v1(&client, session, max_pages, options).await,
        };

        match result {
            Ok(clips) => {
                let mut added = 0;
                for clip in clips {
                    if seen.insert(clip.id.clone()) {
                        merged.push(clip);
                        added += 1;
                    }
                }
                if added > 0 {
                    eprintln!("Feed {name}: +{added} clip(s)");
                }
            }
            Err(error) => {
                eprintln!("Feed {name} unavailable: {error}");
                last_error = Some(error);
            }
        }
    }

    if merged.is_empty() {
        if let Some(error) = last_error {
            return Err(error);
        }
    }

    Ok(merged)
}

pub fn find_wav_url_in_clip(clip: &Clip) -> Option<String> {
    if let Ok(value) = serde_json::to_value(clip) {
        return find_wav_url(&value);
    }
    None
}

pub async fn resolve_wav_source(clip: &Clip) -> (Option<String>, bool) {
    if let Some(url) = find_wav_url_in_clip(clip) {
        return (Some(url), false);
    }
    (None, true)
}

async fn wait_for_wav_url(
    client: &Client,
    session: &SessionData,
    clip_id: &str,
) -> Result<Option<String>, AuthError> {
    let deadline = tokio::time::Instant::now() + Duration::from_secs(120);

    while tokio::time::Instant::now() < deadline {
        let response = api_fetch(
            client,
            session,
            &format!("/api/gen/{clip_id}/wav_file/"),
            reqwest::Method::GET,
            None,
        )
        .await?;

        if response.status() == StatusCode::NOT_FOUND {
            tokio::time::sleep(Duration::from_secs(2)).await;
            continue;
        }

        if !response.status().is_success() {
            return Err(AuthError::Message(format!(
                "WAV status check failed: {}",
                response.status()
            )));
        }

        let content_type = response
            .headers()
            .get(reqwest::header::CONTENT_TYPE)
            .and_then(|value| value.to_str().ok())
            .unwrap_or("")
            .to_string();

        if content_type.contains("application/json") {
            let data: Value = response
                .json()
                .await
                .map_err(|error| AuthError::Message(error.to_string()))?;
            if let Some(url) = find_wav_url(&data) {
                return Ok(Some(url));
            }
        } else {
            let bytes = response
                .bytes()
                .await
                .map_err(|error| AuthError::Message(error.to_string()))?;
            if !bytes.is_empty() {
                return Ok(Some(format!("__binary__:{clip_id}")));
            }
        }

        tokio::time::sleep(Duration::from_secs(2)).await;
    }

    Ok(None)
}

async fn request_wav_conversion(
    client: &Client,
    session: &SessionData,
    clip_id: &str,
) -> Result<Option<String>, AuthError> {
    let response = api_fetch(
        client,
        session,
        &format!("/api/gen/{clip_id}/convert_wav/"),
        reqwest::Method::POST,
        None,
    )
    .await?;

    if !response.status().is_success() {
        return Err(AuthError::Message(format!(
            "WAV conversion request failed: {}",
            response.status()
        )));
    }

    wait_for_wav_url(client, session, clip_id).await
}

pub async fn fetch_wav_for_clip(
    session: &SessionData,
    clip: &Clip,
) -> Result<Vec<u8>, AuthError> {
    let client = Client::new();
    let clip_id = &clip.id;
    let mut wav_url = find_wav_url_in_clip(clip);

    if wav_url.is_none() {
        wav_url = request_wav_conversion(&client, session, clip_id).await?;
    }

    let wav_url = wav_url.unwrap_or_else(|| format!("{CDN_BASE}/{clip_id}.wav"));

    if wav_url.starts_with("__binary__:") {
        let response = api_fetch(
            &client,
            session,
            &format!("/api/gen/{clip_id}/wav_file/"),
            reqwest::Method::GET,
            None,
        )
        .await?;

        if !response.status().is_success() {
            return Err(AuthError::Message(format!(
                "Failed to download WAV binary: {}",
                response.status()
            )));
        }

        return response
            .bytes()
            .await
            .map(|bytes| bytes.to_vec())
            .map_err(|error| AuthError::Message(error.to_string()));
    }

    let mut headers = HeaderMap::new();
    headers.insert(REFERER, HeaderValue::from_static("https://suno.com/"));
    headers.insert(ORIGIN, HeaderValue::from_static("https://suno.com"));

    let response = client
        .get(&wav_url)
        .headers(headers)
        .send()
        .await
        .map_err(|error| AuthError::Message(error.to_string()))?;

    if !response.status().is_success() {
        return Err(AuthError::Message(format!(
            "Failed to download WAV from CDN: {}",
            response.status()
        )));
    }

    response
        .bytes()
        .await
        .map(|bytes| bytes.to_vec())
        .map_err(|error| AuthError::Message(error.to_string()))
}

pub async fn save_wav_file(output_path: &Path, data: &[u8]) -> anyhow::Result<()> {
    if let Some(parent) = output_path.parent() {
        tokio::fs::create_dir_all(parent).await?;
    }
    tokio::fs::write(output_path, data).await?;
    Ok(())
}
