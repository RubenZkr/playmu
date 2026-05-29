use std::path::{Path, PathBuf};

use anyhow::{Context, Result};
use lofty::{
    file::TaggedFileExt,
    picture::PictureType,
    prelude::Accessor,
    probe::Probe,
    tag::ItemKey,
};
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

        let file_meta = std::fs::metadata(path)
            .with_context(|| format!("failed to read metadata for {}", path.display()))?;
        let file_mtime = file_meta
            .modified()?
            .duration_since(std::time::UNIX_EPOCH)?
            .as_secs() as i64;

        // Try to read real tag data; fall back to path-derived heuristics.
        let tag_data = read_tag_data(path);
        let fallback = normalize_from_path(path, source_folder);

        let title = tag_data
            .as_ref()
            .and_then(|t| t.title.clone())
            .unwrap_or(fallback.title);
        let artist = tag_data
            .as_ref()
            .and_then(|t| t.artist.clone())
            .unwrap_or(fallback.artist);
        let album = tag_data
            .as_ref()
            .and_then(|t| t.album.clone())
            .unwrap_or(fallback.album);
        let duration = tag_data
            .as_ref()
            .and_then(|t| t.duration_seconds)
            .unwrap_or(0);

        let file_path = path.to_string_lossy().to_string();
        discovered_paths.push(file_path.clone());

        db::upsert_track(
            &transaction,
            source_folder_id,
            &file_path,
            &title,
            &artist,
            &album,
            duration,
            file_mtime,
        )?;

        // Store embedded album art once per unique artist+album pair.
        if let Some(art_bytes) = tag_data.and_then(|t| t.cover_art) {
            let _ = db::set_album_art_in_tx(&transaction, &artist, &album, &art_bytes);
        }

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

// ---------------------------------------------------------------------------
// Tag reading
// ---------------------------------------------------------------------------

struct TagData {
    title: Option<String>,
    artist: Option<String>,
    album: Option<String>,
    duration_seconds: Option<i64>,
    cover_art: Option<Vec<u8>>,
}

fn read_tag_data(path: &Path) -> Option<TagData> {
    let tagged = Probe::open(path)
        .ok()?
        .guess_file_type()
        .ok()?
        .read()
        .ok()?;

    let properties = tagged.properties();
    let duration_seconds = Some(properties.duration().as_secs() as i64).filter(|&d| d > 0);

    let tag = tagged.primary_tag()?;

    let title = tag
        .get_string(ItemKey::TrackTitle)
        .or_else(|| tag.get_string(ItemKey::OriginalAlbumTitle))
        .map(clean_tag_string);

    let artist = tag
        .get_string(ItemKey::TrackArtist)
        .or_else(|| tag.get_string(ItemKey::AlbumArtist))
        .map(clean_tag_string);

    let album = tag
        .get_string(ItemKey::AlbumTitle)
        .map(clean_tag_string);

    // Prefer CoverFront, fall back to first available picture.
    let cover_art = tag
        .pictures()
        .iter()
        .find(|p| matches!(p.pic_type(), PictureType::CoverFront))
        .or_else(|| tag.pictures().first())
        .map(|p| p.data().to_vec());

    Some(TagData { title, artist, album, duration_seconds, cover_art })
}

fn clean_tag_string(s: &str) -> String {
    s.trim().to_string()
}

// ---------------------------------------------------------------------------
// Path-derived fallback metadata
// ---------------------------------------------------------------------------

struct NormalizedTrack {
    title: String,
    artist: String,
    album: String,
}

fn normalize_from_path(path: &Path, source_folder: &Path) -> NormalizedTrack {
    let title = path
        .file_stem()
        .and_then(|s| s.to_str())
        .unwrap_or("Unknown Title")
        .replace('_', " ");

    let relative_parent = path
        .strip_prefix(source_folder)
        .ok()
        .and_then(|r| r.parent())
        .unwrap_or(Path::new(""));

    let components: Vec<String> = relative_parent
        .components()
        .filter_map(|c| c.as_os_str().to_str().map(ToOwned::to_owned))
        .collect();

    let album = components.last().cloned().unwrap_or_else(|| "Unknown Album".to_string());
    let artist = if components.len() >= 2 {
        components[components.len() - 2].clone()
    } else {
        "Unknown Artist".to_string()
    };

    NormalizedTrack { title, artist, album }
}

fn is_supported_audio_file(path: &Path) -> bool {
    matches!(
        path.extension()
            .and_then(|e| e.to_str())
            .map(|e| e.to_ascii_lowercase())
            .as_deref(),
        Some("mp3" | "flac" | "m4a" | "aac" | "ogg" | "opus" | "wav" | "wv" | "ape")
    )
}

#[allow(dead_code)]
fn _pathbuf(_value: PathBuf) {}
