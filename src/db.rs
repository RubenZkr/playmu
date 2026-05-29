use std::path::Path;

use anyhow::Result;
use rusqlite::{Connection, OptionalExtension, params};

#[derive(Clone, Debug)]
pub struct Track {
    pub id: i64,
    pub file_path: String,
    pub title: String,
    pub artist: String,
    pub album: String,
    pub duration_seconds: i64,
}

pub fn init_database(db_path: &Path) -> Result<()> {
    let connection = Connection::open(db_path)?;

    connection.execute_batch(
        "
        PRAGMA journal_mode = WAL;
        PRAGMA foreign_keys = ON;

        CREATE TABLE IF NOT EXISTS source_folders (
            id INTEGER PRIMARY KEY,
            path TEXT NOT NULL UNIQUE,
            created_at TEXT NOT NULL DEFAULT CURRENT_TIMESTAMP,
            updated_at TEXT NOT NULL DEFAULT CURRENT_TIMESTAMP
        );

        CREATE TABLE IF NOT EXISTS tracks (
            id INTEGER PRIMARY KEY,
            source_folder_id INTEGER NOT NULL,
            file_path TEXT NOT NULL UNIQUE,
            title TEXT NOT NULL,
            artist TEXT NOT NULL,
            album TEXT NOT NULL,
            duration_seconds INTEGER NOT NULL DEFAULT 0,
            file_mtime INTEGER NOT NULL,
            created_at TEXT NOT NULL DEFAULT CURRENT_TIMESTAMP,
            updated_at TEXT NOT NULL DEFAULT CURRENT_TIMESTAMP,
            FOREIGN KEY(source_folder_id) REFERENCES source_folders(id)
        );

        CREATE INDEX IF NOT EXISTS idx_tracks_artist ON tracks(artist);
        CREATE INDEX IF NOT EXISTS idx_tracks_album ON tracks(album);
        CREATE INDEX IF NOT EXISTS idx_tracks_title ON tracks(title);
        ",
    )?;

    Ok(())
}

pub fn list_tracks(db_path: &Path) -> Result<Vec<Track>> {
    let connection = Connection::open(db_path)?;
    let mut statement = connection.prepare(
        "
        SELECT id, file_path, title, artist, album, duration_seconds
        FROM tracks
        ORDER BY artist COLLATE NOCASE, album COLLATE NOCASE, title COLLATE NOCASE
        ",
    )?;

    let rows = statement.query_map([], |row| {
        Ok(Track {
            id: row.get(0)?,
            file_path: row.get(1)?,
            title: row.get(2)?,
            artist: row.get(3)?,
            album: row.get(4)?,
            duration_seconds: row.get(5)?,
        })
    })?;

    let mut tracks = Vec::new();
    for row in rows {
        tracks.push(row?);
    }

    Ok(tracks)
}

pub fn ensure_source_folder(connection: &Connection, path: &str) -> Result<i64> {
    let existing = connection
        .query_row(
            "SELECT id FROM source_folders WHERE path = ?1",
            [path],
            |row| row.get::<_, i64>(0),
        )
        .optional()?;

    if let Some(id) = existing {
        connection.execute(
            "UPDATE source_folders SET updated_at = CURRENT_TIMESTAMP WHERE id = ?1",
            [id],
        )?;
        return Ok(id);
    }

    connection.execute(
        "INSERT INTO source_folders (path) VALUES (?1)",
        [path],
    )?;

    Ok(connection.last_insert_rowid())
}

pub fn upsert_track(
    connection: &Connection,
    source_folder_id: i64,
    file_path: &str,
    title: &str,
    artist: &str,
    album: &str,
    duration_seconds: i64,
    file_mtime: i64,
) -> Result<()> {
    connection.execute(
        "
        INSERT INTO tracks (
            source_folder_id,
            file_path,
            title,
            artist,
            album,
            duration_seconds,
            file_mtime
        ) VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7)
        ON CONFLICT(file_path) DO UPDATE SET
            source_folder_id = excluded.source_folder_id,
            title = excluded.title,
            artist = excluded.artist,
            album = excluded.album,
            duration_seconds = excluded.duration_seconds,
            file_mtime = excluded.file_mtime,
            updated_at = CURRENT_TIMESTAMP
        ",
        params![
            source_folder_id,
            file_path,
            title,
            artist,
            album,
            duration_seconds,
            file_mtime
        ],
    )?;

    Ok(())
}

pub fn remove_tracks_missing_from_scan(
    connection: &Connection,
    source_folder_id: i64,
    present_paths: &[String],
) -> Result<usize> {
    let mut statement = connection.prepare(
        "SELECT file_path FROM tracks WHERE source_folder_id = ?1",
    )?;

    let existing_paths = statement.query_map([source_folder_id], |row| row.get::<_, String>(0))?;
    let present_lookup: std::collections::HashSet<&str> =
        present_paths.iter().map(String::as_str).collect();

    let mut removed = 0;
    for file_path in existing_paths {
        let file_path = file_path?;
        if !present_lookup.contains(file_path.as_str()) {
            removed += connection.execute(
                "DELETE FROM tracks WHERE source_folder_id = ?1 AND file_path = ?2",
                params![source_folder_id, file_path],
            )?;
        }
    }

    Ok(removed)
}