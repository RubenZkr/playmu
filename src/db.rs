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
    pub created_at: String,
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

        CREATE TABLE IF NOT EXISTS play_history (
            id INTEGER PRIMARY KEY,
            track_id INTEGER NOT NULL,
            played_at TEXT NOT NULL DEFAULT CURRENT_TIMESTAMP,
            FOREIGN KEY(track_id) REFERENCES tracks(id)
        );

        CREATE TABLE IF NOT EXISTS pinned_albums (
            artist TEXT NOT NULL,
            album TEXT NOT NULL,
            pinned_at TEXT NOT NULL DEFAULT CURRENT_TIMESTAMP,
            PRIMARY KEY(artist, album)
        );

        CREATE INDEX IF NOT EXISTS idx_play_history_track ON play_history(track_id);
        CREATE INDEX IF NOT EXISTS idx_play_history_played_at ON play_history(played_at DESC);
        CREATE INDEX IF NOT EXISTS idx_pinned_albums_pinned_at ON pinned_albums(pinned_at DESC);

        CREATE TABLE IF NOT EXISTS playlists (
            id INTEGER PRIMARY KEY,
            name TEXT NOT NULL,
            created_at TEXT NOT NULL DEFAULT CURRENT_TIMESTAMP
        );

        CREATE TABLE IF NOT EXISTS playlist_tracks (
            playlist_id INTEGER NOT NULL,
            track_id    INTEGER NOT NULL,
            position    INTEGER NOT NULL DEFAULT 0,
            added_at    TEXT NOT NULL DEFAULT CURRENT_TIMESTAMP,
            PRIMARY KEY (playlist_id, track_id),
            FOREIGN KEY (playlist_id) REFERENCES playlists(id) ON DELETE CASCADE,
            FOREIGN KEY (track_id)    REFERENCES tracks(id)    ON DELETE CASCADE
        );

        CREATE INDEX IF NOT EXISTS idx_playlist_tracks_playlist ON playlist_tracks(playlist_id, position);

        CREATE TABLE IF NOT EXISTS settings (
            key   TEXT NOT NULL PRIMARY KEY,
            value TEXT NOT NULL
        );

        CREATE TABLE IF NOT EXISTS album_art (
            artist TEXT NOT NULL COLLATE NOCASE,
            album  TEXT NOT NULL COLLATE NOCASE,
            data   BLOB NOT NULL,
            PRIMARY KEY (artist, album)
        );
        ",
    )?;

    Ok(())
}

pub fn record_track_play(db_path: &Path, track_id: i64) -> Result<()> {
    let connection = Connection::open(db_path)?;
    connection.execute(
        "INSERT INTO play_history (track_id) VALUES (?1)",
        [track_id],
    )?;
    Ok(())
}

pub fn list_recently_played_tracks(db_path: &Path, limit: usize) -> Result<Vec<Track>> {
    let connection = Connection::open(db_path)?;
    let mut statement = connection.prepare(
        "
        SELECT
            t.id,
            t.file_path,
            t.title,
            t.artist,
            t.album,
            t.duration_seconds,
            t.created_at
        FROM tracks t
        INNER JOIN (
            SELECT track_id, MAX(played_at) AS last_played
            FROM play_history
            GROUP BY track_id
            ORDER BY last_played DESC
            LIMIT ?1
        ) h ON h.track_id = t.id
        ORDER BY h.last_played DESC
        ",
    )?;

    let rows = statement.query_map([limit as i64], |row| {
        Ok(Track {
            id: row.get(0)?,
            file_path: row.get(1)?,
            title: row.get(2)?,
            artist: row.get(3)?,
            album: row.get(4)?,
            duration_seconds: row.get(5)?,
            created_at: row.get(6)?,
        })
    })?;

    let mut tracks = Vec::new();
    for row in rows {
        tracks.push(row?);
    }

    Ok(tracks)
}

pub fn list_pinned_albums(db_path: &Path) -> Result<Vec<(String, String)>> {
    let connection = Connection::open(db_path)?;
    let mut statement = connection.prepare(
        "
        SELECT artist, album
        FROM pinned_albums
        ORDER BY pinned_at DESC
        ",
    )?;

    let rows = statement.query_map([], |row| Ok((row.get(0)?, row.get(1)?)))?;
    let mut pinned = Vec::new();
    for row in rows {
        pinned.push(row?);
    }

    Ok(pinned)
}

pub fn set_album_pinned(db_path: &Path, artist: &str, album: &str, pinned: bool) -> Result<()> {
    let connection = Connection::open(db_path)?;
    if pinned {
        connection.execute(
            "
            INSERT INTO pinned_albums (artist, album)
            VALUES (?1, ?2)
            ON CONFLICT(artist, album) DO UPDATE SET pinned_at = CURRENT_TIMESTAMP
            ",
            params![artist, album],
        )?;
    } else {
        connection.execute(
            "DELETE FROM pinned_albums WHERE artist = ?1 AND album = ?2",
            params![artist, album],
        )?;
    }

    Ok(())
}

pub fn list_tracks(db_path: &Path) -> Result<Vec<Track>> {
    let connection = Connection::open(db_path)?;
    let mut statement = connection.prepare(
        "
        SELECT id, file_path, title, artist, album, duration_seconds, created_at
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
            created_at: row.get(6)?,
        })
    })?;

    let mut tracks = Vec::new();
    for row in rows {
        tracks.push(row?);
    }

    Ok(tracks)
}

// ---------------------------------------------------------------------------
// Settings (key/value persistence)
// ---------------------------------------------------------------------------

pub fn get_setting(db_path: &Path, key: &str) -> Result<Option<String>> {
    let conn = Connection::open(db_path)?;
    let val = conn
        .query_row("SELECT value FROM settings WHERE key = ?1", [key], |r| {
            r.get::<_, String>(0)
        })
        .optional()?;
    Ok(val)
}

pub fn set_setting(db_path: &Path, key: &str, value: &str) -> Result<()> {
    let conn = Connection::open(db_path)?;
    conn.execute(
        "INSERT INTO settings (key, value) VALUES (?1, ?2)
         ON CONFLICT(key) DO UPDATE SET value = excluded.value",
        params![key, value],
    )?;
    Ok(())
}

// ---------------------------------------------------------------------------
// Source folder management
// ---------------------------------------------------------------------------

pub fn list_source_folders(db_path: &Path) -> Result<Vec<(i64, String)>> {
    let conn = Connection::open(db_path)?;
    let mut stmt = conn.prepare("SELECT id, path FROM source_folders ORDER BY created_at")?;
    let rows = stmt.query_map([], |r| Ok((r.get::<_, i64>(0)?, r.get::<_, String>(1)?)))?;
    rows.collect::<rusqlite::Result<Vec<_>>>().map_err(Into::into)
}

pub fn remove_source_folder(db_path: &Path, folder_id: i64) -> Result<()> {
    let conn = Connection::open(db_path)?;
    conn.execute("DELETE FROM tracks WHERE source_folder_id = ?1", [folder_id])?;
    conn.execute("DELETE FROM source_folders WHERE id = ?1", [folder_id])?;
    Ok(())
}

// ---------------------------------------------------------------------------
// Album art
// ---------------------------------------------------------------------------

pub fn get_album_art(db_path: &Path, artist: &str, album: &str) -> Result<Option<Vec<u8>>> {
    let conn = Connection::open(db_path)?;
    let val = conn
        .query_row(
            "SELECT data FROM album_art WHERE artist = ?1 AND album = ?2",
            params![artist, album],
            |r| r.get::<_, Vec<u8>>(0),
        )
        .optional()?;
    Ok(val)
}

pub fn set_album_art(db_path: &Path, artist: &str, album: &str, data: &[u8]) -> Result<()> {
    let conn = Connection::open(db_path)?;
    set_album_art_in_tx(&conn, artist, album, data)
}

/// Like set_album_art but works inside an existing transaction/connection.
pub fn set_album_art_in_tx(conn: &Connection, artist: &str, album: &str, data: &[u8]) -> Result<()> {
    conn.execute(
        "INSERT INTO album_art (artist, album, data) VALUES (?1, ?2, ?3)
         ON CONFLICT(artist, album) DO NOTHING",
        params![artist, album, data],
    )?;
    Ok(())
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

// ---------------------------------------------------------------------------
// Playlists
// ---------------------------------------------------------------------------

#[derive(Clone, Debug)]
pub struct Playlist {
    pub id: i64,
    pub name: String,
    pub created_at: String,
}

pub fn create_playlist(db_path: &Path, name: &str) -> Result<i64> {
    let connection = Connection::open(db_path)?;
    connection.execute("INSERT INTO playlists (name) VALUES (?1)", [name])?;
    Ok(connection.last_insert_rowid())
}

pub fn rename_playlist(db_path: &Path, playlist_id: i64, name: &str) -> Result<()> {
    let connection = Connection::open(db_path)?;
    connection.execute(
        "UPDATE playlists SET name = ?1 WHERE id = ?2",
        params![name, playlist_id],
    )?;
    Ok(())
}

pub fn delete_playlist(db_path: &Path, playlist_id: i64) -> Result<()> {
    let connection = Connection::open(db_path)?;
    connection.execute("DELETE FROM playlists WHERE id = ?1", [playlist_id])?;
    Ok(())
}

pub fn list_playlists(db_path: &Path) -> Result<Vec<Playlist>> {
    let connection = Connection::open(db_path)?;
    let mut stmt = connection.prepare(
        "SELECT id, name, created_at FROM playlists ORDER BY created_at",
    )?;
    let rows = stmt.query_map([], |row| {
        Ok(Playlist { id: row.get(0)?, name: row.get(1)?, created_at: row.get(2)? })
    })?;
    rows.collect::<rusqlite::Result<Vec<_>>>().map_err(Into::into)
}

pub fn list_playlist_track_ids(db_path: &Path, playlist_id: i64) -> Result<Vec<i64>> {
    let connection = Connection::open(db_path)?;
    let mut stmt = connection.prepare(
        "SELECT track_id FROM playlist_tracks WHERE playlist_id = ?1 ORDER BY position, added_at",
    )?;
    let rows = stmt.query_map([playlist_id], |row| row.get::<_, i64>(0))?;
    rows.collect::<rusqlite::Result<Vec<_>>>().map_err(Into::into)
}

pub fn add_track_to_playlist(db_path: &Path, playlist_id: i64, track_id: i64) -> Result<()> {
    let connection = Connection::open(db_path)?;
    let max_pos: i64 = connection
        .query_row(
            "SELECT COALESCE(MAX(position), -1) FROM playlist_tracks WHERE playlist_id = ?1",
            [playlist_id],
            |row| row.get(0),
        )
        .unwrap_or(-1);
    connection.execute(
        "INSERT OR IGNORE INTO playlist_tracks (playlist_id, track_id, position) VALUES (?1, ?2, ?3)",
        params![playlist_id, track_id, max_pos + 1],
    )?;
    Ok(())
}

pub fn remove_track_from_playlist(db_path: &Path, playlist_id: i64, track_id: i64) -> Result<()> {
    let connection = Connection::open(db_path)?;
    connection.execute(
        "DELETE FROM playlist_tracks WHERE playlist_id = ?1 AND track_id = ?2",
        params![playlist_id, track_id],
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