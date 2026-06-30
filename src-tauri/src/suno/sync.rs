use std::path::Path;
use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::Arc;

use tauri::{AppHandle, Emitter};

use crate::config::date_filters::{is_clip_created_after, resolve_since_cutoff_optional};
use crate::config::date_folders::resolve_clip_output_dir;
use crate::config::filenames::build_wav_filename;
use crate::config::local_index::{
    build_local_clip_index, clip_path, list_wav_filenames, mark_clip_downloaded,
};
use crate::config::session::load_session;
use crate::config::settings::{default_max_pages, default_organize, resolve_output_dir};
use crate::suno::api::{
    fetch_all_clips, fetch_wav_for_clip, resolve_wav_source, save_wav_file, FetchClipsOptions,
};
use crate::suno::auth::AuthError;
use crate::suno::types::{
    LibraryClip, LibraryListResult, SyncOptions, SyncPreviewItem, SyncPreviewResult,
    SyncProgressEvent, SyncSummary,
};

fn emit_progress(app: &AppHandle, event: SyncProgressEvent) {
    let _ = app.emit("sync-progress", event);
}

fn clip_title(clip: &crate::suno::types::Clip) -> String {
    clip.title
        .as_deref()
        .map(str::trim)
        .filter(|value| !value.is_empty())
        .unwrap_or("untitled")
        .to_string()
}

pub async fn library_list(
    output_dir: Option<String>,
    since: Option<String>,
    max_pages: Option<u32>,
) -> Result<LibraryListResult, String> {
    let session = load_session().await.map_err(|error| error.to_string())?;
    let output_dir = resolve_output_dir(output_dir.as_deref())
        .await
        .map_err(|error| error.to_string())?;
    let local_ids = build_local_clip_index(Path::new(&output_dir))
        .await
        .map_err(|error| error.to_string())?;

    let since_cutoff = resolve_since_cutoff_optional(since.as_deref())
        .map_err(|error| error.to_string())?;

    let clips = fetch_all_clips(
        &session,
        max_pages.unwrap_or(0),
        &FetchClipsOptions {
            created_after: since_cutoff,
        },
    )
    .await
    .map_err(|error| error.to_string())?;

    let items = clips
        .into_iter()
        .filter(|clip| {
            since_cutoff
                .as_ref()
                .is_none_or(|cutoff| is_clip_created_after(clip, cutoff))
        })
        .map(|clip| LibraryClip {
            id: clip.id.clone(),
            title: clip_title(&clip),
            created_at: clip.created_at.clone(),
            synced: local_ids.contains(&clip.id.to_lowercase()),
        })
        .collect();

    Ok(LibraryListResult {
        local_count: local_ids.len(),
        clips: items,
    })
}

pub async fn sync_preview(options: SyncOptions) -> Result<SyncPreviewResult, String> {
    let cancel_flag = Arc::new(AtomicBool::new(false));
    let (summary, items) = run_sync_internal(None, options, true, &cancel_flag).await?;
    Ok(SyncPreviewResult { items, summary })
}

pub async fn sync_run(
    app: AppHandle,
    options: SyncOptions,
    cancel_flag: Arc<AtomicBool>,
) -> Result<SyncSummary, String> {
    let (summary, _) = run_sync_internal(Some(app), options, false, &cancel_flag).await?;
    Ok(summary)
}

#[allow(clippy::too_many_lines)]
async fn run_sync_internal(
    app: Option<AppHandle>,
    options: SyncOptions,
    dry_run: bool,
    cancel_flag: &Arc<AtomicBool>,
) -> Result<(SyncSummary, Vec<SyncPreviewItem>), String> {
    let output_dir = resolve_output_dir(options.dir.as_deref())
        .await
        .map_err(|error| error.to_string())?;
    let settings = crate::config::settings::load_settings()
        .await
        .map_err(|error| error.to_string())?;

    let max_pages = options
        .max_pages
        .unwrap_or_else(|| default_max_pages(&settings));
    let organize = options
        .organize
        .clone()
        .unwrap_or_else(|| default_organize(&settings));

    let since_cutoff = resolve_since_cutoff_optional(
        options
            .since
            .as_deref()
            .or(settings.since.as_deref()),
    )
    .map_err(|error| error.to_string())?;

    let session = load_session().await.map_err(|error| error.to_string())?;
    tokio::fs::create_dir_all(&output_dir)
        .await
        .map_err(|error| error.to_string())?;

    let mut local_ids = build_local_clip_index(Path::new(&output_dir))
        .await
        .map_err(|error| error.to_string())?;

    let mut summary = SyncSummary::default();
    let mut preview_items = Vec::new();
    let mut conversions_done = 0u32;

    let clips = fetch_all_clips(
        &session,
        max_pages,
        &FetchClipsOptions {
            created_after: since_cutoff,
        },
    )
    .await
    .map_err(|error| error.to_string())?;

    let pending_estimate = clips
        .iter()
        .filter(|clip| !local_ids.contains(&clip.id.to_lowercase()))
        .count() as u32;

    for clip in clips {
        if cancel_flag.load(Ordering::Relaxed) {
            break;
        }

        summary.remote_count += 1;
        let clip_id = clip.id.to_lowercase();
        let title = clip_title(&clip);
        let target_dir = resolve_clip_output_dir(
            Path::new(&output_dir),
            clip.created_at.as_deref(),
            &organize,
        );
        let display_path = target_dir
            .join(format!("{title}.wav"))
            .strip_prefix(&output_dir)
            .map_or_else(
                |_| format!("{title}.wav"),
                |path| path.to_string_lossy().to_string(),
            );

        if local_ids.contains(&clip_id) {
            summary.skipped += 1;
            if let Some(app) = &app {
                emit_progress(
                    app,
                    SyncProgressEvent {
                        clip_id: clip.id.clone(),
                        title: title.clone(),
                        status: "skipped".to_string(),
                        message: None,
                    },
                );
            }
            continue;
        }

        if let Some(cutoff) = since_cutoff {
            if !is_clip_created_after(&clip, &cutoff) {
                summary.filtered += 1;
                if let Some(app) = &app {
                    emit_progress(
                        app,
                        SyncProgressEvent {
                            clip_id: clip.id.clone(),
                            title: title.clone(),
                            status: "filtered".to_string(),
                            message: None,
                        },
                    );
                }
                continue;
            }
        }

        summary.pending_count += 1;

        if dry_run {
            preview_items.push(SyncPreviewItem {
                id: clip.id.clone(),
                title: title.clone(),
                display_path,
            });
            continue;
        }

        if let Some(app) = &app {
            emit_progress(
                app,
                SyncProgressEvent {
                    clip_id: clip.id.clone(),
                    title: title.clone(),
                    status: "downloading".to_string(),
                    message: None,
                },
            );
        }

        match download_clip(
            &session,
            &clip,
            &title,
            &target_dir,
            Path::new(&output_dir),
            &mut local_ids,
            conversions_done,
            pending_estimate,
        )
        .await
        {
            Ok((path, converted)) => {
                if converted {
                    conversions_done += 1;
                }
                summary.downloaded += 1;
                if let Some(app) = &app {
                    emit_progress(
                        app,
                        SyncProgressEvent {
                            clip_id: clip.id.clone(),
                            title: title.clone(),
                            status: "done".to_string(),
                            message: Some(path),
                        },
                    );
                }
            }
            Err(error) => {
                summary.failed += 1;
                let message = error.to_string();
                if let Some(app) = &app {
                    emit_progress(
                        app,
                        SyncProgressEvent {
                            clip_id: clip.id.clone(),
                            title: title.clone(),
                            status: "failed".to_string(),
                            message: Some(message.clone()),
                        },
                    );
                }

                if error.to_string().contains("セッション") {
                    return Err(message);
                }
            }
        }
    }

    Ok((summary, preview_items))
}

fn wav_conversion_delay_secs(conversions_done: u32, pending_estimate: u32) -> u64 {
    let nanos = std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .unwrap_or_default()
        .as_nanos() as u64;

    let batch_extra = match pending_estimate {
        0..=15 => 0,
        16..=40 => 3,
        41..=80 => 6,
        _ => 10,
    };
    let progress_extra = (u64::from(conversions_done) / 5).min(20);
    let extra = batch_extra + progress_extra;

    let min = 5 + extra;
    let max = 10 + extra + 5;
    let span = max - min + 1;
    min + (nanos % span)
}

async fn download_clip(
    session: &crate::suno::types::SessionData,
    clip: &crate::suno::types::Clip,
    title: &str,
    target_dir: &Path,
    output_dir: &Path,
    local_ids: &mut std::collections::HashSet<String>,
    conversions_done: u32,
    pending_estimate: u32,
) -> Result<(String, bool), AuthError> {
    let (_, needs_conversion) = resolve_wav_source(clip).await;
    if needs_conversion {
        let delay_secs = wav_conversion_delay_secs(conversions_done, pending_estimate);
        tokio::time::sleep(std::time::Duration::from_secs(delay_secs)).await;
    }

    let wav_data = fetch_wav_for_clip(session, clip).await?;
    tokio::fs::create_dir_all(target_dir)
        .await
        .map_err(|error| AuthError::Message(error.to_string()))?;

    let existing = list_wav_filenames(target_dir)
        .await
        .map_err(|error| AuthError::Message(error.to_string()))?;
    let filename = build_wav_filename(title, &existing);
    let output_path = clip_path(target_dir, &filename);
    save_wav_file(&output_path, &wav_data)
        .await
        .map_err(|error| AuthError::Message(error.to_string()))?;
    mark_clip_downloaded(output_dir, &clip.id, local_ids)
        .await
        .map_err(|error| AuthError::Message(error.to_string()))?;

    Ok((output_path.to_string_lossy().to_string(), needs_conversion))
}

pub fn request_sync_cancel(cancel_flag: &Arc<AtomicBool>) {
    cancel_flag.store(true, Ordering::Relaxed);
}
