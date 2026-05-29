use std::path::{Path, PathBuf};

use anyhow::{Context, Result};
use rusqlite::Connection;
use walkdir::WalkDir;

use crate::db;

#[derive(Clone, Debug)]
pub struct ScanSummary {
    pub imported_tracks: usize,
    pub removed_tracks: usize,
    pub source_folder: String,
}

pub fn scan_music_folder(db_path: &Path, source_folder: &Path) -> Result<ScanSummary> {
    let mut connection = Connection::open(db_path)?;
    let transaction = connection.transaction()?;
    let source_folder_string = source_folder.to_string_lossy().to_string();
    let source_folder_id = db::ensure_source_folder(&transaction, &source_folder_string)?;

    let mut discovered_paths = Vec::new();
    let mut imported_tracks = 0;

    for entry in WalkDir::new(source_folder)
        .follow_links(true)
        .into_iter()
        .filter_map(Result::ok)
    {
        let path = entry.path();
        if !path.is_file() || !is_supported_audio_file(path) {
            continue;
        }

        let metadata = std::fs::metadata(path)
            .with_context(|| format!("failed to read metadata for {}", path.display()))?;

        let file_mtime = metadata
            .modified()?
            .duration_since(std::time::UNIX_EPOCH)?
            .as_secs() as i64;

        let normalized = normalize_track(path, source_folder);
        let file_path = path.to_string_lossy().to_string();
        discovered_paths.push(file_path.clone());

        db::upsert_track(
            &transaction,
            source_folder_id,
            &file_path,
            &normalized.title,
            &normalized.artist,
            &normalized.album,
            0,
            file_mtime,
        )?;

        imported_tracks += 1;
    }

    let removed_tracks =
        db::remove_tracks_missing_from_scan(&transaction, source_folder_id, &discovered_paths)?;

    transaction.commit()?;

    Ok(ScanSummary {
        imported_tracks,
        removed_tracks,
        source_folder: source_folder_string,
    })
}

#[derive(Debug)]
struct NormalizedTrack {
    title: String,
    artist: String,
    album: String,
}

fn normalize_track(path: &Path, source_folder: &Path) -> NormalizedTrack {
    let file_stem = path
        .file_stem()
        .and_then(|stem| stem.to_str())
        .unwrap_or("Unknown Title")
        .replace('_', " ");

    let relative_parent = path
        .strip_prefix(source_folder)
        .ok()
        .and_then(|relative| relative.parent())
        .unwrap_or_else(|| Path::new(""));

    let parent_components: Vec<String> = relative_parent
        .components()
        .filter_map(|component| component.as_os_str().to_str().map(ToOwned::to_owned))
        .collect();

    let album = parent_components
        .last()
        .cloned()
        .unwrap_or_else(|| "Unknown Album".to_string());

    let artist = if parent_components.len() >= 2 {
        parent_components[parent_components.len() - 2].clone()
    } else {
        "Unknown Artist".to_string()
    };

    NormalizedTrack {
        title: file_stem,
        artist,
        album,
    }
}

fn is_supported_audio_file(path: &Path) -> bool {
    matches!(
        path.extension()
            .and_then(|extension| extension.to_str())
            .map(|extension| extension.to_ascii_lowercase()),
        Some(extension)
            if matches!(
                extension.as_str(),
                "mp3" | "flac" | "m4a" | "aac" | "ogg" | "opus" | "wav"
            )
    )
}

#[allow(dead_code)]
fn _pathbuf(_value: PathBuf) {}