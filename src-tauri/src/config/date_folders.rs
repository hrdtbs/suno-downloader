use std::path::{Path, PathBuf};

use chrono::{DateTime, Datelike, Utc};

use crate::suno::types::{Clip, OrganizeMode};

const UNKNOWN_FOLDER: &str = "unknown";

pub fn parse_created_at(clip: &Clip) -> Option<DateTime<Utc>> {
    let raw = clip.created_at.as_ref()?;
    DateTime::parse_from_rfc3339(raw)
        .ok()
        .map(|dt| dt.with_timezone(&Utc))
        .or_else(|| {
            chrono::DateTime::parse_from_str(raw, "%Y-%m-%dT%H:%M:%S%.fZ")
                .ok()
                .map(|dt| dt.with_timezone(&Utc))
        })
}

pub fn format_month_key(date: DateTime<Utc>) -> String {
    format!("{}-{:02}", date.year(), date.month())
}

pub fn format_iso_week_key(date: DateTime<Utc>) -> String {
    let (iso_year, iso_week) = iso_week_parts(date);
    format!("{iso_year}-W{iso_week:02}")
}

fn iso_week_parts(date: DateTime<Utc>) -> (i32, u32) {
    let iso = date.date_naive().iso_week();
    (iso.year(), iso.week())
}

pub fn resolve_clip_output_dir(
    base_dir: &Path,
    created_at: Option<&str>,
    mode: &OrganizeMode,
) -> PathBuf {
    if matches!(mode, OrganizeMode::Flat) {
        return base_dir.to_path_buf();
    }

    let date = created_at
        .and_then(|raw| DateTime::parse_from_rfc3339(raw).ok())
        .map(|dt| dt.with_timezone(&Utc));

    let Some(date) = date else {
        return base_dir.join(UNKNOWN_FOLDER);
    };

    let month_key = format_month_key(date);
    let week_key = format_iso_week_key(date);

    match mode {
        OrganizeMode::Month => base_dir.join(month_key),
        OrganizeMode::Week => base_dir.join(week_key),
        OrganizeMode::MonthWeek => base_dir.join(month_key).join(week_key),
        OrganizeMode::Flat => base_dir.to_path_buf(),
    }
}
