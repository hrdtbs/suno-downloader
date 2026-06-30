use std::collections::HashSet;

const INVALID_CHARS: &str = r#"/<>:"/\|?*"#;
const MAX_FILENAME_LENGTH: usize = 120;

pub fn sanitize_filename(title: &str) -> String {
    let mut cleaned = String::new();
    for ch in title.chars() {
        if ch.is_control() || INVALID_CHARS.contains(ch) {
            continue;
        }
        cleaned.push(ch);
    }

    let cleaned = cleaned.split_whitespace().collect::<Vec<_>>().join(" ");
    let cleaned = cleaned.trim();

    if cleaned.is_empty() {
        return "untitled".to_string();
    }

    if cleaned.chars().count() <= MAX_FILENAME_LENGTH {
        return cleaned.to_string();
    }

    cleaned
        .chars()
        .take(MAX_FILENAME_LENGTH)
        .collect::<String>()
        .trim()
        .to_string()
}

pub fn build_wav_filename(title: &str, existing_filenames: &HashSet<String>) -> String {
    let base = sanitize_filename(title);
    let filename = format!("{base}.wav");

    if !existing_filenames.contains(&filename) {
        return filename;
    }

    let mut counter = 2;
    loop {
        let candidate = format!("{base} ({counter}).wav");
        if !existing_filenames.contains(&candidate) {
            return candidate;
        }
        counter += 1;
    }
}
