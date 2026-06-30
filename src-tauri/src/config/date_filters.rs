use chrono::{DateTime, Utc};

use crate::config::date_folders::parse_created_at;
use crate::suno::types::Clip;

const MS_PER_HOUR: i64 = 3_600_000;
const MS_PER_DAY: i64 = 86_400_000;

pub fn parse_since_duration(value: &str) -> anyhow::Result<i64> {
    let trimmed = value.trim();
    if trimmed.is_empty() {
        anyhow::bail!("Since value must not be empty");
    }

    let re = regex::Regex::new(r"(?i)^(\d+)([dwmh])$")?;
    let captures = re
        .captures(trimmed)
        .ok_or_else(|| anyhow::anyhow!("Invalid since value: {value}"))?;

    let amount: i64 = captures[1].parse()?;
    if amount <= 0 {
        anyhow::bail!("Duration must be greater than 0");
    }

    let duration_ms = match captures[2].to_ascii_lowercase().as_str() {
        "h" => amount * MS_PER_HOUR,
        "w" => amount * 7 * MS_PER_DAY,
        "m" => amount * 30 * MS_PER_DAY,
        _ => amount * MS_PER_DAY,
    };

    Ok(duration_ms)
}

pub fn resolve_since_cutoff(since: &str) -> anyhow::Result<DateTime<Utc>> {
    let duration_ms = parse_since_duration(since)?;
    let cutoff = Utc::now() - chrono::Duration::milliseconds(duration_ms);
    Ok(cutoff)
}

pub fn resolve_since_cutoff_optional(
    since: Option<&str>,
) -> anyhow::Result<Option<DateTime<Utc>>> {
    match since.map(str::trim).filter(|value| !value.is_empty()) {
        None => Ok(None),
        Some(value) => resolve_since_cutoff(value).map(Some),
    }
}

pub fn is_clip_created_after(clip: &Clip, cutoff: &DateTime<Utc>) -> bool {
    parse_created_at(clip).is_some_and(|created| created >= *cutoff)
}

pub fn should_stop_feed_paging(clips: &[Clip], created_after: &DateTime<Utc>) -> bool {
    let mut has_valid_date = false;

    for clip in clips {
        let Some(created_at) = parse_created_at(clip) else {
            continue;
        };
        has_valid_date = true;
        if created_at >= *created_after {
            return false;
        }
    }

    has_valid_date
}
