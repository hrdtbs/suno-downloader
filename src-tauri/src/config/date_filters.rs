use chrono::{DateTime, Utc};

use crate::config::date_folders::parse_created_at;
use crate::suno::types::Clip;

const MS_PER_DAY: i64 = 86_400_000;

pub fn parse_since_duration(value: &str) -> anyhow::Result<i64> {
    let trimmed = value.trim();
    let re = regex::Regex::new(r"(?i)^(\d+)([dw])$")?;
    let captures = re
        .captures(trimmed)
        .ok_or_else(|| anyhow::anyhow!("Invalid since value: {value}"))?;

    let amount: i64 = captures[1].parse()?;
    if amount <= 0 {
        anyhow::bail!("Duration must be greater than 0");
    }

    let days = match captures[2].to_ascii_lowercase().as_str() {
        "w" => amount * 7,
        _ => amount,
    };

    Ok(days * MS_PER_DAY)
}

pub fn resolve_since_cutoff(since: &str) -> anyhow::Result<DateTime<Utc>> {
    let duration_ms = parse_since_duration(since)?;
    let cutoff = Utc::now() - chrono::Duration::milliseconds(duration_ms);
    Ok(cutoff)
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
