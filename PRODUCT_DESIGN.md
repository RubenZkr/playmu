# Playmu Product Design Document

## 1. Product Summary

Playmu is a native desktop music player for Arch Linux focused exclusively on local audio files. The product goal is to deliver the familiarity, speed, and library ergonomics of Spotify while applying a more refined, Apple-like visual and interaction standard. It is not a streaming product, not a web app, and does not require any online account to function.

Core promise:

- Feels as fluid and organized as Spotify.
- Looks more premium, calm, and tactile, closer to Apple's product sensibility.
- Works entirely from local files stored on disk.
- Ships as a runnable Linux desktop application for Arch Linux.

## 2. Product Vision

Users with large local music libraries should get the same level of speed, discoverability, queue control, album browsing, and playback continuity that they expect from modern streaming apps, without giving up ownership of their files.

Playmu should make local music feel current again.

## 3. Goals

- Provide a Spotify-like desktop navigation model for local music.
- Support fast indexing of large local libraries.
- Make albums, artists, playlists, and queue management first-class.
- Deliver gapless playback, crossfade, replay gain, and robust metadata handling.
- Feel premium on Arch Linux with a polished desktop-native application shell.
- Launch fast and stay responsive with libraries of 100k+ tracks.

## 4. Non-Goals

- No music streaming.
- No cloud sync in v1.
- No social features.
- No podcast or video support.
- No browser-based delivery.
- No Electron-style heavy desktop shell for v1.

## 5. Target Users

### Primary

- Users who maintain curated local music libraries.
- Linux users who want a premium music experience instead of a utilitarian file browser.
- Former Spotify or Apple Music desktop users who want equivalent ergonomics for owned media.

### Secondary

- DJs and collectors with large FLAC, MP3, AAC, OGG, and WAV libraries.
- Users with external drives or NAS-mounted libraries.
- Users who care about album art, metadata cleanliness, and queue control.

## 6. Platform Strategy

### Target platform

- Arch Linux desktop.

### Delivery format

- Native desktop application.
- Primary package target: AUR package.
- Secondary package targets: native package build and AppImage for easy distribution.

### Technology recommendation

Use **Qt 6 + QML + Rust backend + GStreamer**.

Reasoning:

- Qt Quick provides the smooth, animated, high-density desktop UI needed to closely match Spotify's interaction model.
- QML is strong for polished transitions, layered layouts, album grids, sidebars, and responsive desktop panels.
- Rust provides safe, fast backend code for indexing, metadata handling, caching, and queue logic.
- GStreamer is mature on Linux for playback, codec support, gapless playback, and audio pipeline control.
- SQLite is sufficient for library indexing, search, playlists, play history, and cached metadata.

Why not a web app shell:

- The product should feel native on Linux, integrate cleanly with MPRIS, media keys, tray behavior, notifications, and filesystem watchers.
- Memory footprint and startup time should be materially better than typical webview-based desktop apps.

## 7. Design Principles

- **Spotify familiarity**: users should immediately understand navigation, queue behavior, and library structure.
- **Apple-like finish**: typography, spacing, animation, hierarchy, and surfaces should feel deliberate and premium.
- **Local-first truth**: files on disk are the source of truth.
- **Speed over decoration**: all animations must support clarity and never harm browsing speed.
- **Album-centric browsing**: preserve the emotional value of albums, not just loose tracks.
- **Power without clutter**: advanced controls are available without turning the product into a pro audio tool.

## 8. Product Positioning

"Spotify for your owned music collection, designed like a premium desktop media product."

## 9. Core Experience

The application should mirror Spotify's desktop mental model:

- Left navigation rail/sidebar.
- Central content canvas.
- Right-side contextual panel for queue, lyrics placeholder, or track info in future phases.
- Persistent bottom playback bar.
- Global search at the top.
- Library sections for Home, Search, Your Library, Liked Songs, Albums, Artists, Playlists, Recently Played.

The difference is that all content is derived from local music files and local metadata.

## 10. Information Architecture

### Primary navigation

- Home
- Search
- Your Library

### Library subsections

- Recently Added
- Recently Played
- Albums
- Artists
- Songs
- Genres
- Playlists
- Liked Songs
- Folders

### Detail pages

- Album page
- Artist page
- Playlist page
- Genre page
- Folder source page
- Search results page
- Queue page
- Settings page

## 11. Feature Set

### v1 Must-Have

- Recursive folder import.
- Background indexing.
- Automatic metadata extraction from tags.
- Embedded and sidecar album art support.
- Global search across songs, albums, artists, genres, and playlists.
- Spotify-style queue management.
- Play, pause, next, previous, seek, shuffle, repeat.
- Gapless playback.
- Crossfade.
- Replay gain normalization.
- Drag-and-drop playlist creation.
- Liked songs.
- Recently played and recently added.
- Sort, filter, and multi-select actions.
- File change detection with automatic re-scan.
- MPRIS and media key support.
- Desktop notifications.
- Mini-player mode.
- Remember last session, queue, and playback position.

### v1.1 Should-Have

- Smart playlists.
- Duplicate detection.
- Missing metadata repair flows.
- Better folder-source management.
- Waveform preview for track scrubbing.
- Last.fm scrobbling as optional integration.

### Later

- Local network remote control.
- Device sync.
- Lyrics from local files.
- Multi-user library profiles.

## 12. Functional Requirements

### Library ingestion

- User can add one or more root music folders.
- System recursively scans supported formats.
- System reads tags: title, artist, album, album artist, track number, disc number, year, genre, composer, replay gain, cover art.
- System fingerprints file identity using stable file path plus metadata checksum.
- System updates the library incrementally when files change.
- System gracefully handles broken tags and missing art.

### Supported formats

- MP3
- FLAC
- AAC/M4A
- OGG Vorbis
- Opus
- WAV
- ALAC if codec support is available in system pipeline

### Search

- Search starts returning results within 50 ms for common queries on warm cache.
- Results grouped into top result, songs, albums, artists, playlists, genres.
- Fuzzy matching with typo tolerance.
- Keyboard-first navigation.

### Playback

- Start playback in under 150 ms from warm state for cached local files.
- Seamless next-track transition for gapless-capable formats.
- Queue survives app restart.
- User can enqueue next, enqueue last, reorder queue, remove queue entries, and clear upcoming queue.

### Playlists

- Manual playlists.
- Drag tracks, albums, or artists into playlists.
- Playlist metadata stored locally in app database and exportable as M3U.

### Library views

- Album grid and list views.
- Artist view grouped by albums.
- Songs table view with dense columns.
- Sort by recently added, recently played, title, artist, album, duration, year.

### Desktop integration

- MPRIS transport controls.
- Global media keys.
- System notifications with album art.
- Optional tray icon.
- Deep filesystem integration for reveal in file manager.

## 13. User Experience Requirements

### UX mandate

The app should be **functionally as intuitive as Spotify desktop**, but with a more premium desktop feel.

### Interaction rules

- Single-click selects.
- Double-click plays.
- Enter plays current selection.
- Space toggles play/pause when not typing.
- Cmd-like quick command palette behavior via `Ctrl+K`.
- Context menus must expose all major actions without overwhelming the user.

### Keyboard shortcuts

- `Space`: play/pause
- `Ctrl+Right`: next track
- `Ctrl+Left`: previous track
- `Ctrl+L`: focus search
- `Ctrl+K`: open command palette
- `Ctrl+Shift+L`: like/unlike current track
- `Ctrl+Up/Down`: volume control
- `Ctrl+Enter`: play from selection

## 14. Visual Design Direction

### Design brief

The product should look like **Spotify's desktop structure filtered through Apple's restraint**.

### Style characteristics

- Dense but breathable layout.
- Large artwork moments on album and playlist pages.
- Soft material layering.
- Crisp typography with strong hierarchy.
- Controlled color extraction from album art.
- Smooth, quiet animations.
- Minimal chrome and high-content focus.

### Visual language

- Sidebar: dark translucent surface with clear active states.
- Main content: softly elevated surfaces with subtle gradients.
- Playback bar: solid anchored surface with premium transport controls.
- Cards: rounded, restrained, slightly tactile.
- Hover states: understated, never loud.
- Selection states: obvious and consistent.

### Typography

- Primary UI font: SF Pro equivalent where legally and technically appropriate is not guaranteed on Linux; use a high-quality alternative such as Inter Tight, Geist, or IBM Plex Sans.
- Display titles: slightly tighter tracking, large weight contrast.
- Metadata text: compact and legible for dense libraries.

### Color system

- Base palette should be neutral graphite, silver-gray, smoke, and warm white.
- Accent colors are derived from current album art but clamped to maintain contrast and avoid visual chaos.
- Avoid neon gamer aesthetics.

### Motion

- Navigation transitions under 220 ms.
- Queue and panel transitions under 180 ms.
- Use easing that feels physical, not bouncy.
- No gratuitous animation loops.

## 15. Benchmark: What “Exactly Like Spotify” Means

The request should be interpreted as matching **Spotify's product model and interaction patterns**, not copying protected branding.

Must match:

- Sidebar-first app structure.
- Content hierarchy.
- Queue behavior.
- Search behavior.
- Album, artist, playlist, and song browsing patterns.
- Dense desktop music management workflows.

Must not copy:

- Spotify brand assets.
- Logos.
- Exact iconography.
- Proprietary artwork.
- Trademarked visual identifiers.

## 16. Core Screens

### 1. Home

Purpose:

- Immediate return point.
- Surface recently played, pinned albums, popular artists in library, recent imports, and unfinished albums.

Sections:

- Jump back in
- Recently added
- Favorite artists
- Made for you locally: generated mixes based on play history

### 2. Search

Purpose:

- Fast, keyboard-first access to all library entities.

Behavior:

- Search field focused quickly.
- Live grouped results.
- Filter chips for songs, albums, artists, playlists, genres, folders.

### 3. Your Library

Purpose:

- Master organization view.

Behavior:

- Toggle between playlists, albums, artists, folders.
- Grid/list mode.
- Pin favorites.
- Sort and filter.

### 4. Album Page

Purpose:

- Showcase album as the core object.

Contents:

- Large artwork.
- Album metadata.
- Play and shuffle actions.
- Track list.
- Credits if available.
- Related albums from same artist in local library.

### 5. Artist Page

Purpose:

- Library view of an artist, grouped by albums.

Contents:

- Hero header.
- Popular tracks in user's library.
- Albums.
- Singles/EPs.
- Appears on if metadata allows.

### 6. Playlist Page

Purpose:

- Manual or smart grouping.

Contents:

- Artwork collage or custom image.
- Playlist description.
- Track list.
- Duration.
- Sort and reorder controls.

### 7. Bottom Playback Bar

Contents:

- Current artwork.
- Title and artist.
- Like action.
- Transport controls.
- Scrubber.
- Time elapsed and remaining.
- Volume.
- Queue toggle.
- Device output selector if available.

### 8. Right Panel

v1 behavior:

- Queue view.
- Up next and history.
- Current track metadata.

Future behavior:

- Lyrics.
- Credits.
- Related tracks.

## 17. User Flows

### First launch

1. User launches Playmu.
2. Empty-state onboarding asks for music folders.
3. Indexing begins immediately.
4. User sees progress and can start browsing imported items before scan completes.
5. App restores to Home after import.

### Play an album

1. User searches artist or opens album from library.
2. User opens album page.
3. User clicks play.
4. Album is queued in track order.
5. Queue panel reflects upcoming songs.

### Build a playlist

1. User multi-selects tracks or albums.
2. User drags selection to playlist in sidebar or uses context menu.
3. Playlist updates instantly.
4. Optional save/export to M3U.

### Resume listening

1. User reopens app.
2. Previous queue, current track, timestamp, repeat, and shuffle are restored.

## 18. Accessibility Requirements

- Full keyboard navigation.
- Screen reader labels for all transport and library controls.
- High-contrast mode support.
- Minimum touch target sizing even for pointer-first UI.
- Reduced motion option.
- Color contrast at WCAG AA minimum.

## 19. Performance Requirements

- Cold launch under 2.5 seconds on a typical Arch Linux desktop with SSD.
- Warm relaunch under 1.0 second.
- Library of 100k tracks remains navigable at 60 fps in primary views.
- Metadata indexing runs in background without blocking the UI thread.
- Scrolling large song lists must remain smooth through virtualization.

## 20. Technical Architecture

### Frontend

- Qt 6 / QML desktop shell.
- Componentized layout system for sidebar, content panels, cards, tables, and player controls.
- State model shared from Rust backend through typed IPC bindings.

### Backend

- Rust services for library indexing, metadata parsing, playback orchestration, queue engine, search engine, and settings.

### Playback engine

- GStreamer pipeline.
- Device selection and output management through Linux audio stack.
- Replay gain and gapless handling in playback layer.

### Database

- SQLite for tracks, albums, artists, playlists, sources, scan state, history, preferences.
- FTS5 for text search.

### Filesystem layer

- Watcher for monitored directories.
- Incremental re-index strategy.
- Path migration handling for moved libraries when possible.

### System integration

- MPRIS.
- Notifications.
- Media keys.
- File manager reveal.

## 21. Data Model Overview

### Core entities

- Track
- Album
- Artist
- AlbumArtist
- Genre
- Playlist
- PlaylistEntry
- SourceFolder
- PlaybackQueueEntry
- PlayHistoryEntry
- UserPreference
- ArtworkCacheEntry

### Key relationships

- An album contains many tracks.
- An artist can have many albums and tracks.
- A playlist contains ordered track references.
- A source folder contributes many tracks.
- A track can appear in many playlists.

## 22. Offline and Privacy Model

- Fully offline-capable after installation.
- No account required.
- No telemetry by default.
- Optional diagnostics must be explicit opt-in.
- All user data stored locally.

## 23. Arch Linux Packaging Requirements

- Buildable on current Arch stable toolchain.
- Package dependencies clearly declared for Qt 6, GStreamer, and media codecs.
- Provide `.desktop` entry, icons, MIME associations if relevant, and proper application metadata.
- Support installation through AUR.
- Avoid distro-specific hacks that prevent portability to other Linux distributions later.

## 24. Risks

- Matching Spotify's usability without accidentally copying brand-specific visual details.
- Metadata inconsistency in user libraries can undermine polish.
- Large library scanning can feel slow if incremental indexing is not carefully designed.
- Linux audio and codec environments vary across machines.
- High-polish desktop animation can hurt responsiveness if overused.

## 25. Open Questions

- Should smart playlists ship in v1 or wait until library fundamentals are solid?
- Should folder browsing be exposed as a first-class navigation item or remain secondary to albums/artists?
- Should the right panel default to queue or stay hidden until invoked?
- Should local lyrics files such as `.lrc` be supported in v1?
- Is tray behavior expected, or should the app behave like a standard single-window desktop app?

## 26. Success Metrics

- Time to first music after install: under 3 minutes.
- Search satisfaction: user finds intended item in first query for 90% of sampled tasks.
- Playback continuity: session restore succeeds reliably.
- Library scan stability: no crashes on malformed metadata.
- Perceived polish: users describe the app as modern, premium, and faster than typical Linux local players.

## 27. Recommended v1 Build Scope

Ship the following in the first public milestone:

- Native desktop shell.
- Folder import and indexing.
- Songs, albums, artists, playlists, liked songs.
- Search.
- Queue.
- Bottom playback bar.
- Gapless playback.
- MPRIS.
- Session restore.
- Premium visual foundation.

Defer everything else until this core loop feels excellent.

## 28. Summary

Playmu should be built as a **native Arch Linux desktop music player** that reproduces the organizational clarity and listening workflow of Spotify for users who own their music locally. The correct implementation direction is a polished Qt/QML desktop application backed by Rust, GStreamer, and SQLite. The product must prioritize speed, library scale, queue quality, and premium visual restraint over feature sprawl.
