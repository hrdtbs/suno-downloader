use std::fs::File;
use std::io::{Read, Write};
use std::path::{Path, PathBuf};

use tauri::path::BaseDirectory;
use tauri::Manager;
use walkdir::WalkDir;
use zip::write::SimpleFileOptions;
use zip::ZipWriter;

fn is_extension_dir(path: &Path) -> bool {
    path.join("manifest.json").is_file()
}

fn normalize_path(path: PathBuf) -> PathBuf {
    path.canonicalize().unwrap_or(path)
}

pub fn resolve_dir(app: &tauri::AppHandle) -> Result<PathBuf, String> {
    let mut candidates = Vec::new();

    if let Ok(path) = app
        .path()
        .resolve("../chrome-extension", BaseDirectory::Resource)
    {
        candidates.push(path);
    }

    if let Ok(path) = app.path().resolve("chrome-extension", BaseDirectory::Resource) {
        candidates.push(path);
    }

    if let Ok(resource) = app.path().resource_dir() {
        candidates.push(resource.join("chrome-extension"));
        candidates.push(resource.join("_up_").join("chrome-extension"));
    }

    candidates.push(
        PathBuf::from(env!("CARGO_MANIFEST_DIR"))
            .join("..")
            .join("chrome-extension"),
    );

    if let Ok(cwd) = std::env::current_dir() {
        candidates.push(cwd.join("chrome-extension"));
        if let Some(parent) = cwd.parent() {
            candidates.push(parent.join("chrome-extension"));
        }
    }

    for candidate in candidates {
        if is_extension_dir(&candidate) {
            return Ok(normalize_path(candidate));
        }
    }

    Err("Chrome extension files not found.".to_string())
}

pub fn zip_directory(source_dir: &Path, dest_file: &Path) -> Result<(), String> {
    let file = File::create(dest_file).map_err(|error| error.to_string())?;
    let mut zip = ZipWriter::new(file);
    let options = SimpleFileOptions::default().compression_method(zip::CompressionMethod::Deflated);

    for entry in WalkDir::new(source_dir)
        .into_iter()
        .filter_map(Result::ok)
        .filter(|entry| entry.path() != source_dir)
    {
        let path = entry.path();
        let name = path
            .strip_prefix(source_dir)
            .map_err(|error| error.to_string())?;
        let name = name.to_string_lossy().replace('\\', "/");

        if path.is_file() {
            zip.start_file(name, options)
                .map_err(|error| error.to_string())?;
            let mut source = File::open(path).map_err(|error| error.to_string())?;
            let mut buffer = Vec::new();
            source
                .read_to_end(&mut buffer)
                .map_err(|error| error.to_string())?;
            zip.write_all(&buffer)
                .map_err(|error| error.to_string())?;
        } else if path.is_dir() {
            zip.add_directory(format!("{name}/"), options)
                .map_err(|error| error.to_string())?;
        }
    }

    zip.finish().map_err(|error| error.to_string())?;
    Ok(())
}
