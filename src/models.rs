use crate::{db::{Playlist, Track}, util::{summarize_albums, summarize_artists}};

#[derive(Clone, Copy, PartialEq, Eq)]
pub enum NavSection {
    Home,
    Search,
    Library,
}

impl NavSection {
    pub fn label(self) -> &'static str {
        match self {
            Self::Home => "Home",
            Self::Search => "Search",
            Self::Library => "Your Library",
        }
    }

    pub fn subtitle(self) -> &'static str {
        match self {
            Self::Home => "A focused landing space for your own collection.",
            Self::Search => "Jump to any song, artist, or album in your local library.",
            Self::Library => "Dense browsing for tracks, artists, and albums.",
        }
    }
}

#[derive(Clone, PartialEq, Eq)]
pub enum BrowseFocus {
    All,
    Artist(String),
    Album { artist: String, album: String },
}

impl BrowseFocus {
    pub fn label(&self) -> String {
        match self {
            Self::All => "All music".to_string(),
            Self::Artist(artist) => format!("Artist: {artist}"),
            Self::Album { artist, album } => format!("Album: {artist} - {album}"),
        }
    }
}

#[derive(Clone, Copy, PartialEq, Eq)]
pub enum LibraryView {
    Songs,
    Albums,
    Artists,
    Playlists,
}

impl LibraryView {
    pub fn label(self) -> &'static str {
        match self {
            Self::Songs => "Songs",
            Self::Albums => "Albums",
            Self::Artists => "Artists",
            Self::Playlists => "Playlists",
        }
    }
}

/// Which tab is active in the Song View overlay.
#[derive(Clone, Copy, PartialEq, Eq, Default)]
pub enum SongViewTab {
    #[default]
    Cover,
    Lyrics,
    Waves,
}

/// Lyrics content, either plain text or LRC-synced lines.
#[derive(Clone)]
pub enum LyricsData {
    /// Unsynchronized plain text (from embedded tags).
    Plain(String),
    /// LRC-synchronized: (timestamp_seconds, line_text).
    Synced(Vec<(f64, String)>),
}

/// Full playlist including its resolved track list.
#[derive(Clone)]
pub struct PlaylistView {
    pub playlist: Playlist,
    pub track_ids: Vec<i64>,
}

#[derive(Clone, Copy, PartialEq, Eq)]
pub enum LibrarySortKey {
    Title,
    Artist,
    Album,
    Duration,
}

impl LibrarySortKey {
    pub fn label(self) -> &'static str {
        match self {
            Self::Title => "Title",
            Self::Artist => "Artist",
            Self::Album => "Album",
            Self::Duration => "Duration",
        }
    }
}

#[derive(Clone, Copy, PartialEq, Eq, Default)]
pub enum RepeatMode {
    #[default]
    Off,
    Queue,
    Track,
}

impl RepeatMode {
    pub fn next(self) -> Self {
        match self {
            Self::Off => Self::Queue,
            Self::Queue => Self::Track,
            Self::Track => Self::Off,
        }
    }
}

#[derive(Clone, Copy, PartialEq, Eq)]
pub enum LibraryDensity {
    Compact,
    Dense,
}

impl LibraryDensity {
    pub fn row_height(self) -> f32 {
        match self {
            Self::Compact => 40.0,
            Self::Dense => 62.0,
        }
    }

    pub fn vertical_gap(self) -> f32 {
        match self {
            Self::Compact => 2.0,
            Self::Dense => 6.0,
        }
    }
}

#[derive(Clone)]
pub struct LibraryStats {
    pub track_count: usize,
    pub artist_count: usize,
    pub album_count: usize,
    pub top_artists: Vec<(String, usize)>,
    pub recent_albums: Vec<AlbumSummary>,
}

#[derive(Clone)]
pub struct AlbumSummary {
    pub title: String,
    pub artist: String,
    pub track_count: usize,
    pub track_ids: Vec<i64>,
}

#[derive(Clone)]
pub struct ArtistSummary {
    pub name: String,
    pub track_count: usize,
    pub album_count: usize,
    pub track_ids: Vec<i64>,
}

pub struct SearchResults {
    pub tracks: Vec<Track>,
    pub albums: Vec<AlbumSummary>,
    pub artists: Vec<ArtistSummary>,
}

impl SearchResults {
    pub fn from_tracks(tracks: Vec<Track>) -> Self {
        let albums = summarize_albums(tracks.iter()).into_iter().take(8).collect();
        let artists = summarize_artists(tracks.iter()).into_iter().take(8).collect();
        Self { tracks, albums, artists }
    }
}

#[derive(Clone)]
pub struct MixSummary {
    pub name: String,
    pub description: String,
    pub track_ids: Vec<i64>,
}
