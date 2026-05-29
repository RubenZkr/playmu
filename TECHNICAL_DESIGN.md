# Playmu Technical Design Document

## 1. Purpose

This document defines the technical architecture for Playmu v1, a native Arch Linux desktop music player for local music files only. It translates the product requirements into an implementable system design.

Playmu v1 must:

- run as a native desktop application on Arch Linux
- store application state locally
- index and play local audio files
- provide a Spotify-like library and queue model
- avoid any streaming or cloud dependency

## 2. Technology Decisions

### Primary stack

- Rust for application core and services
- Qt 6 + QML for the desktop UI
- GStreamer for playback
- SQLite for local storage in a single `.db` file

### Why this stack

- Rust is appropriate for concurrent scanning, metadata parsing, state management, and search orchestration.
- Qt/QML provides the desktop-grade UI control needed for dense library browsing and polished animations.
- GStreamer is the most practical Linux-native playback layer for codec support and audio pipeline control.
- SQLite gives a simple, reliable, transactional local store that is sufficient for the full v1 product.

## 3. v1 Architecture Overview

Playmu is a single-process desktop application with a layered architecture.

### Layers

1. Presentation layer
2. Application layer
3. Domain services layer
4. Infrastructure layer

### High-level flow

1. QML renders the UI and emits user intents.
2. Rust application controllers receive intents and coordinate services.
3. Services read and update state in SQLite, the playback engine, and the filesystem.
4. The Rust state layer publishes view models back to QML.

## 4. Runtime Components

### UI shell

Responsible for:

- window lifecycle
- navigation state
- rendering lists, grids, detail pages, and player controls
- search input and results presentation
- queue and context menus

Implementation:

- Qt 6
- Qt Quick / QML
- Rust-to-QML bindings for commands and state exposure

### App core

Responsible for:

- startup and shutdown flow
- service orchestration
- command handling
- event dispatch
- session restore
- state synchronization to UI

### Library service

Responsible for:

- source folder registration
- recursive scanning
- metadata extraction
- album and artist normalization
- incremental updates after filesystem changes

### Playback service

Responsible for:

- play, pause, seek, next, previous
- queue execution
- shuffle and repeat
- gapless playback
- crossfade settings
- replay gain application
- playback state events

### Search service

Responsible for:

- fast query execution over tracks, albums, artists, genres, and playlists
- ranking and grouped result formatting
- warm-cache performance targets

### Playlist service

Responsible for:

- manual playlist creation and editing
- liked songs management
- playlist ordering
- export to M3U

### System integration service

Responsible for:

- MPRIS
- media keys
- desktop notifications
- reveal in file manager
- optional tray support

## 5. Recommended Repository Structure

Suggested structure for implementation:

```text
playmu/
  app/
    Cargo.toml
    src/
      main.rs
      app/
      domain/
      infra/
      ui/
      db/
  ui/
    qml/
      App.qml
      components/
      pages/
      theme/
  assets/
    icons/
  packaging/
    PKGBUILD
    desktop/
  docs/
    PRODUCT_DESIGN.md
    TECHNICAL_DESIGN.md
```

If a single-root layout is preferred initially, the same logical module boundaries should still be preserved.

## 6. Data Storage Strategy

### v1 decision

All application data should be stored locally in a single SQLite database file.

Recommended file location:

- `~/.local/share/playmu/playmu.db`

### Data kept in the database

- library index
- normalized track metadata
- albums
- artists
- genres
- playlists
- playlist ordering
- liked songs
- play history
- queue snapshot
- app settings
- source folders
- scan checkpoints
- artwork metadata and cache references

### Data kept outside the database

- actual audio files remain in their original locations
- optional artwork cache files may live in a local cache directory
- logs remain as plain text files

### SQLite configuration

- WAL mode enabled
- foreign keys enabled
- migrations required from day one
- FTS5 enabled for search

## 7. Database Schema

The schema should remain normalized enough for correctness, but practical enough for fast v1 iteration.

### `source_folders`

- `id` INTEGER PRIMARY KEY
- `path` TEXT NOT NULL UNIQUE
- `display_name` TEXT
- `is_enabled` INTEGER NOT NULL DEFAULT 1
- `created_at` TEXT NOT NULL
- `updated_at` TEXT NOT NULL
- `last_scan_at` TEXT
- `scan_cursor` TEXT

### `tracks`

- `id` INTEGER PRIMARY KEY
- `source_folder_id` INTEGER NOT NULL
- `file_path` TEXT NOT NULL UNIQUE
- `file_name` TEXT NOT NULL
- `file_size` INTEGER NOT NULL
- `file_mtime` INTEGER NOT NULL
- `content_hash` TEXT
- `title` TEXT NOT NULL
- `sort_title` TEXT
- `album_id` INTEGER
- `primary_artist_id` INTEGER
- `track_number` INTEGER
- `disc_number` INTEGER
- `duration_ms` INTEGER NOT NULL
- `year` INTEGER
- `bitrate` INTEGER
- `sample_rate` INTEGER
- `codec` TEXT
- `replay_gain_track` REAL
- `replay_gain_album` REAL
- `is_liked` INTEGER NOT NULL DEFAULT 0
- `play_count` INTEGER NOT NULL DEFAULT 0
- `last_played_at` TEXT
- `date_added` TEXT NOT NULL
- `date_modified` TEXT NOT NULL

### `albums`

- `id` INTEGER PRIMARY KEY
- `title` TEXT NOT NULL
- `sort_title` TEXT
- `album_artist_id` INTEGER
- `year` INTEGER
- `artwork_id` INTEGER
- `track_count` INTEGER NOT NULL DEFAULT 0
- `duration_ms` INTEGER NOT NULL DEFAULT 0
- `date_added` TEXT NOT NULL

### `artists`

- `id` INTEGER PRIMARY KEY
- `name` TEXT NOT NULL UNIQUE
- `sort_name` TEXT
- `artwork_id` INTEGER

### `track_artists`

- `track_id` INTEGER NOT NULL
- `artist_id` INTEGER NOT NULL
- `role` TEXT NOT NULL DEFAULT 'primary'

### `genres`

- `id` INTEGER PRIMARY KEY
- `name` TEXT NOT NULL UNIQUE

### `track_genres`

- `track_id` INTEGER NOT NULL
- `genre_id` INTEGER NOT NULL

### `artworks`

- `id` INTEGER PRIMARY KEY
- `source_kind` TEXT NOT NULL
- `source_path` TEXT
- `mime_type` TEXT
- `cache_key` TEXT
- `dominant_color` TEXT
- `width` INTEGER
- `height` INTEGER
- `updated_at` TEXT NOT NULL

### `playlists`

- `id` INTEGER PRIMARY KEY
- `name` TEXT NOT NULL
- `description` TEXT
- `artwork_id` INTEGER
- `is_smart` INTEGER NOT NULL DEFAULT 0
- `created_at` TEXT NOT NULL
- `updated_at` TEXT NOT NULL

### `playlist_entries`

- `id` INTEGER PRIMARY KEY
- `playlist_id` INTEGER NOT NULL
- `track_id` INTEGER NOT NULL
- `position` INTEGER NOT NULL
- `added_at` TEXT NOT NULL

### `play_history`

- `id` INTEGER PRIMARY KEY
- `track_id` INTEGER NOT NULL
- `started_at` TEXT NOT NULL
- `completed_at` TEXT
- `played_ms` INTEGER NOT NULL DEFAULT 0
- `source_context` TEXT
- `source_context_id` INTEGER

### `queue_entries`

- `id` INTEGER PRIMARY KEY
- `position` INTEGER NOT NULL
- `track_id` INTEGER NOT NULL
- `source_context` TEXT
- `source_context_id` INTEGER
- `enqueued_at` TEXT NOT NULL

### `app_state`

- `key` TEXT PRIMARY KEY
- `value` TEXT NOT NULL
- `updated_at` TEXT NOT NULL

### `settings`

- `key` TEXT PRIMARY KEY
- `value` TEXT NOT NULL
- `updated_at` TEXT NOT NULL

### `search_index`

FTS5 virtual table populated from tracks, albums, artists, and playlists.

Suggested columns:

- `entity_type`
- `entity_id`
- `title`
- `subtitle`
- `keywords`

## 8. Domain Model

### Primary entities

- Track
- Album
- Artist
- Genre
- Playlist
- QueueEntry
- SourceFolder
- PlaybackState
- SessionState

### Domain rules

- A track belongs to one source folder and may belong to one normalized album.
- A track may have multiple artists and genres.
- A playlist contains ordered track references.
- The queue is persisted and restored on launch.
- The database is the app's local state source of truth, but the music files remain the media source of truth.

## 9. Library Scanning and Indexing

### Supported sources

- local directories
- mounted drives
- mounted network paths if exposed as normal filesystem paths

### Scan pipeline

1. Enumerate enabled source folders.
2. Walk directories recursively.
3. Filter by supported extensions.
4. Detect file changes using path, size, and mtime.
5. Parse metadata only for new or changed files.
6. Normalize album and artist records.
7. Update search index.
8. Emit progress events to the UI.

### Metadata parser requirements

- support ID3, Vorbis comments, and common MP4 tags
- extract embedded cover art when available
- fall back to folder artwork files when present
- tolerate invalid or partial tags
- generate safe display titles for malformed files

### File watching

Use a filesystem watcher to detect changes after the initial scan.

Watcher actions:

- add newly discovered files
- refresh changed files
- remove missing files from active library views
- preserve playlists and history records for removed tracks where possible

## 10. Search Architecture

### v1 approach

Use SQLite FTS5 for indexed search, with a Rust ranking layer for grouped results.

### Search requirements

- prefix matching for fast initial results
- typo tolerance through lightweight fallback matching in Rust when exact FTS ranking is insufficient
- grouped results for top hit, songs, albums, artists, playlists, genres
- keyboard-first result navigation

### Search flow

1. User types in the search field.
2. UI debounces input lightly.
3. Rust search service queries FTS5.
4. Search service ranks and groups results.
5. UI renders grouped sections.

## 11. Playback Architecture

### Playback engine

GStreamer should own decoding and audio output.

### Playback responsibilities

- load track URI from local file path
- maintain transport state
- emit position and state updates
- prepare next item for seamless transition
- apply replay gain where supported
- honor shuffle and repeat logic from the queue service

### Queue model

The queue must behave like a modern streaming desktop player:

- explicit queue order
- current track pointer
- up next section
- played history section
- enqueue next
- enqueue last
- drag reorder
- remove from queue
- clear future queue

### Session restore

Persist:

- current track id
- playback position
- queue order
- shuffle state
- repeat mode
- volume

## 12. UI Architecture

### Main regions

- left sidebar
- top search and navigation header
- central content area
- right contextual panel
- bottom playback bar

### Major pages

- HomePage
- SearchPage
- LibraryPage
- AlbumPage
- ArtistPage
- PlaylistPage
- SettingsPage

### View model strategy

The Rust core should expose purpose-built UI models rather than raw tables.

Examples:

- `HomeViewModel`
- `AlbumDetailViewModel`
- `ArtistDetailViewModel`
- `QueueViewModel`
- `PlaybackBarViewModel`

This keeps the QML side simple and avoids business logic leakage into the UI layer.

## 13. Application Commands

The UI should send explicit commands to the app core.

Examples:

- `ImportSourceFolder(path)`
- `PlayTrack(track_id)`
- `PlayAlbum(album_id)`
- `TogglePlayPause`
- `SeekTo(position_ms)`
- `SetVolume(percent)`
- `QueueNext(track_id)`
- `QueueLast(track_id)`
- `RemoveQueueEntry(queue_entry_id)`
- `CreatePlaylist(name)`
- `AddTracksToPlaylist(playlist_id, track_ids)`
- `ToggleLikedTrack(track_id)`
- `Search(query)`

## 14. Inter-Component Communication

### Preferred model

- command-based requests from UI to app core
- event-based updates from app core to UI

### Event examples

- `ScanStarted`
- `ScanProgress`
- `LibraryUpdated`
- `PlaybackStateChanged`
- `PlaybackPositionChanged`
- `QueueUpdated`
- `SearchResultsUpdated`
- `NotificationRequested`

## 15. Error Handling Strategy

### Principles

- invalid media files must not crash the app
- partial scan failures should be isolated and reported
- playback failures should skip gracefully when possible
- UI errors should surface actionable messages

### Error categories

- filesystem errors
- metadata parse errors
- database errors
- playback pipeline errors
- integration errors

### User-facing behavior

- show non-blocking errors for bad files
- allow retry for scan failures
- log full technical diagnostics locally

## 16. Performance Strategy

### Required outcomes

- cold launch under 2.5 seconds on SSD-backed systems
- smooth scrolling for large lists
- search results on warm cache within tens of milliseconds for common queries

### Implementation tactics

- async scanning off the UI thread
- batched writes to SQLite
- prepared statements for hot paths
- list virtualization in QML views
- artwork caching by stable keys
- incremental scan updates instead of full rebuilds

## 17. Concurrency Model

Rust services should run with clear separation between UI-facing state and worker tasks.

### Suggested task groups

- main application thread
- scan worker pool
- metadata extraction workers
- playback event listener
- artwork decode workers

### Rules

- QML must never block on filesystem or database work
- database writes should be serialized or coordinated through a repository layer
- playback callbacks should emit state updates through a safe event channel

## 18. Security and Privacy

### Privacy model

- no account system
- no remote telemetry by default
- no external dependency required to use the app

### Security considerations

- treat all tag metadata as untrusted input
- sanitize file paths before shell or desktop integration calls
- validate exported playlist paths
- avoid arbitrary command execution for file actions

## 19. Logging and Diagnostics

### Local diagnostics

Recommended log directory:

- `~/.local/state/playmu/`

Recommended files:

- `app.log`
- `scan.log`
- `playback.log`

### Log content

- startup and shutdown events
- scan summaries
- scan failures per file
- playback failures
- database migration status

## 20. Packaging and Linux Integration

### Arch Linux requirements

- package as an AUR-installable desktop app
- include `.desktop` file
- include app icons
- register MIME and desktop metadata where useful
- declare runtime dependencies explicitly

### Runtime integrations

- MPRIS support
- media key support
- desktop notifications with artwork
- reveal current file in the file manager

## 21. Testing Strategy

### Unit tests

- metadata normalization
- queue behavior
- playlist ordering
- scan diff logic
- search ranking helpers

### Integration tests

- SQLite migrations
- import of fixture libraries
- playback state transitions with mocked pipeline hooks where possible
- session restore behavior

### UI tests

- smoke coverage for main navigation
- playback bar behavior
- queue interactions
- search result rendering

### Manual test fixtures

Maintain a fixture library containing:

- well-tagged albums
- broken tags
- missing artwork
- multi-disc albums
- compilation albums
- large library subsets

## 22. v1 Milestones

### Milestone 1: application skeleton

- app startup
- QML shell
- Rust command bridge
- SQLite initialization
- settings load and save

### Milestone 2: library ingestion

- source folder management
- recursive scan
- metadata extraction
- basic album, artist, and songs views

### Milestone 3: playback

- GStreamer integration
- transport controls
- queue persistence
- session restore

### Milestone 4: product workflow

- playlists
- liked songs
- search
- right-side queue panel
- notifications and MPRIS

### Milestone 5: polish

- performance work
- artwork caching
- animation tuning
- packaging and desktop integration

## 23. Initial Crate and Module Plan

Suggested Rust module layout:

```text
src/
  main.rs
  app/
    mod.rs
    commands.rs
    events.rs
    state.rs
  domain/
    mod.rs
    playback.rs
    queue.rs
    library.rs
    playlists.rs
    search.rs
  infra/
    mod.rs
    db.rs
    migrations.rs
    scanner.rs
    metadata.rs
    artwork.rs
    watcher.rs
    player.rs
    mpris.rs
    notifications.rs
  ui/
    mod.rs
    bindings.rs
    view_models.rs
```

## 24. Rust Library Recommendations

Candidate crates for evaluation:

- `rusqlite` for SQLite
- `notify` for filesystem watching
- `lofty` or `symphonia` for metadata parsing evaluation
- `gstreamer` crate family for playback integration
- `serde` and `serde_json` for persisted app state values
- `tracing` and `tracing-subscriber` for logging
- `anyhow` and `thiserror` for error handling

Final crate selection should prioritize Linux stability over novelty.

## 25. Known Tradeoffs

- Qt/QML plus Rust is more complex than a pure Electron implementation, but it better satisfies the native desktop requirement.
- A single SQLite database simplifies shipping, but artwork blobs should be monitored carefully to avoid oversized database growth.
- Gapless playback and crossfade details may vary by format and pipeline behavior.

## 26. Recommended v1 Technical Scope

Build the smallest complete system that proves the core loop:

- import folders
- build library index
- browse albums, artists, and songs
- search locally
- play music reliably
- persist queue and settings in one local database
- expose Linux desktop playback integration

Everything else should be deferred until that path is solid.
