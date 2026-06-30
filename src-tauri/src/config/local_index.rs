use std::collections::HashSet;
use std::path::{Path, PathBuf};
use std::sync::OnceLock;

use regex::Regex;
use serde::{Deserialize, Serialize};
use walkdir::WalkDir;

use crate::config::paths::{INDEX_FILENAME, LEGACY_INDEX_FILENAME};

#[derive(Debug, Deserialize, Serialize)]
struct ClipIndexData {
    #[serde(rename = "clipIds")]
    clip_ids: Vec<String>,
}

fn legacy_clip_id_pattern() -> &'static Regex {
    static PATTERN: OnceLock<Regex> = OnceLock::new();
    PATTERN.get_or_init(|| {
        Regex::new(r"(?i)([0-9a-f]{8}-[0-9a-f]{4}-[0-9a-f]{4}-[0-9a-f]{4}-[0-9a-f]{12})\.wav$")
            .unwrap()
    })
}

async fn read_index_file_at(output_dir: &Path, filename: &str) -> anyhow::Result<HashSet<String>> {
    let mut ids = HashSet::new();
    let path = output_dir.join(filename);

    match tokio::fs::read_to_string(&path).await {
        Ok(raw) => {
            let data: ClipIndexData = serde_json::from_str(&raw)?;
            for id in data.clip_ids {
                ids.insert(id.to_lowercase());
            }
        }
        Err(error) if error.kind() == std::io::ErrorKind::NotFound => {}
        Err(error) => return Err(error.into()),
    }

    Ok(ids)
}

fn scan_legacy_filenames(output_dir: &Path) -> HashSet<String> {
    let mut ids = HashSet::new();

    for entry in WalkDir::new(output_dir)
        .follow_links(false)
        .into_iter()
        .filter_map(Result::ok)
    {
        if !entry.file_type().is_file() {
            continue;
        }

        let file_name = entry.file_name().to_string_lossy();
        if let Some(captures) = legacy_clip_id_pattern().captures(&file_name) {
            if let Some(id) = captures.get(1) {
                ids.insert(id.as_str().to_lowercase());
            }
        }
    }

    ids
}

pub async fn persist_clip_index(
    output_dir: &Path,
    clip_ids: &HashSet<String>,
) -> anyhow::Result<()> {
    let mut sorted: Vec<String> = clip_ids.iter().cloned().collect();
    sorted.sort();

    let data = ClipIndexData { clip_ids: sorted };
    tokio::fs::write(
        output_dir.join(INDEX_FILENAME),
        serde_json::to_string_pretty(&data)?,
    )
    .await?;
    Ok(())
}

pub async fn build_local_clip_index(output_dir: &Path) -> anyhow::Result<HashSet<String>> {
    tokio::fs::create_dir_all(output_dir).await?;

    let from_index = read_index_file_at(output_dir, INDEX_FILENAME).await?;
    let from_legacy_index = read_index_file_at(output_dir, LEGACY_INDEX_FILENAME).await?;
    let from_legacy_files = scan_legacy_filenames(output_dir);

    let merged: HashSet<String> = from_index
        .iter()
        .chain(from_legacy_index.iter())
        .chain(from_legacy_files.iter())
        .cloned()
        .collect();

    if from_legacy_files.len() > from_index.len() && !from_legacy_files.is_empty() {
        persist_clip_index(output_dir, &merged).await?;
    }

    Ok(merged)
}

pub async fn list_wav_filenames(output_dir: &Path) -> anyhow::Result<HashSet<String>> {
    let mut filenames = HashSet::new();

    let mut entries = match tokio::fs::read_dir(output_dir).await {
        Ok(entries) => entries,
        Err(error) if error.kind() == std::io::ErrorKind::NotFound => return Ok(filenames),
        Err(error) => return Err(error.into()),
    };

    while let Some(entry) = entries.next_entry().await? {
        let name = entry.file_name().to_string_lossy().to_string();
        if name.to_lowercase().ends_with(".wav") {
            filenames.insert(name);
        }
    }

    Ok(filenames)
}

pub async fn mark_clip_downloaded(
    output_dir: &Path,
    clip_id: &str,
    clip_ids: &mut HashSet<String>,
) -> anyhow::Result<()> {
    clip_ids.insert(clip_id.to_lowercase());
    persist_clip_index(output_dir, clip_ids).await
}

pub fn clip_path(output_dir: &Path, filename: &str) -> PathBuf {
    output_dir.join(filename)
}
