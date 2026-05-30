use std::{
    collections::{HashMap, HashSet},
    path::PathBuf,
    sync::mpsc::{self, Receiver},
    thread,
    time::Duration,
};

use eframe::egui;

use crate::{
    audio::AudioPlayer,
    db::{self, Playlist, Track},
    library::{self, ScanSummary},
    models::{
        AlbumSummary, ArtistSummary, BrowseFocus, LibraryDensity, LibrarySortKey, LibraryStats,
        LibraryView, LyricsData, MixSummary, NavSection, PlaylistView, RepeatMode, SearchResults,
        SongViewTab,
    },
    theme::configure_theme,
    util::{summarize_albums, summarize_artists},
};

/// Actions that can be triggered from a track's context menu.
#[derive(Clone, Copy, PartialEq, Eq)]
pub enum TrackAction {
    PlayNow,
    AddToQueueEnd,
    GoToAlbum,
    GoToArtist,
}

pub struct PlaymuApp {
    pub(crate) db_path: PathBuf,
    pub(crate) source_input: String,
    pub(crate) search_query: String,
    pub(crate) tracks: Vec<Track>,
    pub(crate) selected_track_id: Option<i64>,
    pub(crate) now_playing_track_id: Option<i64>,
    pub(crate) queue: Vec<i64>,
    pub(crate) queue_position: Option<usize>,
    pub(crate) active_nav: NavSection,
    pub(crate) library_view: LibraryView,
    pub(crate) browse_focus: BrowseFocus,
    pub(crate) library_sort_key: LibrarySortKey,
    pub(crate) library_sort_ascending: bool,
    pub(crate) library_density: LibraryDensity,
    pub(crate) recently_played: Vec<Track>,
    pub(crate) pinned_albums: Vec<(String, String)>,
    pub(crate) search_input_has_focus: bool,
    pub(crate) status: String,
    pub(crate) scan_receiver: Option<Receiver<Result<ScanSummary, String>>>,
    pub(crate) is_scanning: bool,
    pub(crate) audio_player: Option<AudioPlayer>,
    // Playlists
    pub(crate) playlists: Vec<Playlist>,
    pub(crate) active_playlist_id: Option<i64>,
    pub(crate) new_playlist_name: String,
    pub(crate) show_create_playlist: bool,
    /// Track whose "add to playlist" popup is open (None = closed).
    pub(crate) add_to_playlist_track_id: Option<i64>,
    // Playback modes
    pub(crate) shuffle: bool,
    pub(crate) repeat: RepeatMode,
    // Layout
    pub(crate) queue_panel_open: bool,
    // Album art texture cache: "artist\x00album" → TextureHandle
    pub(crate) art_cache: HashMap<String, egui::TextureHandle>,
    // Current time for animations (seconds since startup)
    pub(crate) anim_time: f64,
    // Song View overlay
    pub(crate) song_view_open: bool,
    pub(crate) song_view_tab: SongViewTab,
    /// 0.0 = fully closed, 1.0 = fully open (drives the appear animation).
    pub(crate) song_view_anim: f32,
    pub(crate) waveform_data: Option<Vec<f32>>,
    pub(crate) waveform_receiver: Option<Receiver<Vec<f32>>>,
    pub(crate) lyrics_data: Option<LyricsData>,
}

impl PlaymuApp {
    pub fn new(cc: &eframe::CreationContext<'_>) -> Self {
        configure_theme(&cc.egui_ctx);

        let db_path = std::env::current_dir()
            .unwrap_or_else(|_| PathBuf::from("."))
            .join("playmu.db");

        let mut status =
            String::from("Add a music folder path, import it, then select a track to play.");

        if let Err(e) = db::init_database(&db_path) {
            status = format!("Database initialization failed: {e}");
        }

        let tracks = db::list_tracks(&db_path).unwrap_or_default();
        let audio_player = match AudioPlayer::new() {
            Ok(player) => Some(player),
            Err(e) => {
                status = format!("Audio output unavailable: {e}");
                None
            }
        };

        let mut app = Self {
            db_path,
            source_input: String::new(),
            search_query: String::new(),
            tracks,
            selected_track_id: None,
            now_playing_track_id: None,
            queue: Vec::new(),
            queue_position: None,
            active_nav: NavSection::Home,
            library_view: LibraryView::Songs,
            browse_focus: BrowseFocus::All,
            library_sort_key: LibrarySortKey::Artist,
            library_sort_ascending: true,
            library_density: LibraryDensity::Dense,
            recently_played: Vec::new(),
            pinned_albums: Vec::new(),
            search_input_has_focus: false,
            status,
            scan_receiver: None,
            is_scanning: false,
            audio_player,
            playlists: Vec::new(),
            active_playlist_id: None,
            new_playlist_name: String::new(),
            show_create_playlist: false,
            add_to_playlist_track_id: None,
            shuffle: false,
            repeat: RepeatMode::Off,
            queue_panel_open: true,
            art_cache: HashMap::new(),
            anim_time: 0.0,
            song_view_open: false,
            song_view_tab: SongViewTab::Cover,
            song_view_anim: 0.0,
            waveform_data: None,
            waveform_receiver: None,
            lyrics_data: None,
        };
        app.refresh_home_personalization();
        app.refresh_playlists();
        app.restore_session();
        app
    }

    // -------------------------------------------------------------------------
    // Scan / import
    // -------------------------------------------------------------------------

    pub fn start_scan(&mut self) {
        let source_folder = self.source_input.trim();
        if source_folder.is_empty() {
            self.status = "Enter an absolute music-folder path before importing.".to_string();
            return;
        }
        let source_path = PathBuf::from(source_folder);
        if !source_path.exists() || !source_path.is_dir() {
            self.status = "The path does not exist or is not a directory.".to_string();
            return;
        }

        let db_path = self.db_path.clone();
        let (tx, rx) = mpsc::channel();
        self.scan_receiver = Some(rx);
        self.is_scanning = true;
        self.status = format!("Scanning {} ...", source_path.display());

        thread::spawn(move || {
            let result = library::scan_music_folder(&db_path, &source_path)
                .map_err(|e| e.to_string());
            let _ = tx.send(result);
        });
    }

    fn refresh_after_scan(&mut self, summary: ScanSummary) {
        self.tracks = db::list_tracks(&self.db_path).unwrap_or_default();
        self.refresh_home_personalization();
        if self.selected_track_id.is_none() {
            self.selected_track_id = self.tracks.first().map(|t| t.id);
        }
        self.status = format!(
            "Imported {} tracks from {}. Removed {} stale entries.",
            summary.imported_tracks, summary.source_folder, summary.removed_tracks
        );
        self.is_scanning = false;
    }

    pub fn process_background_events(&mut self) {
        if let Some(receiver) = &self.scan_receiver {
            match receiver.try_recv() {
                Ok(Ok(summary)) => {
                    self.scan_receiver = None;
                    self.refresh_after_scan(summary);
                }
                Ok(Err(e)) => {
                    self.scan_receiver = None;
                    self.is_scanning = false;
                    self.status = format!("Scan failed: {e}");
                }
                Err(mpsc::TryRecvError::Empty) => {}
                Err(mpsc::TryRecvError::Disconnected) => {
                    self.scan_receiver = None;
                    self.is_scanning = false;
                    self.status = "Scan worker disconnected unexpectedly.".to_string();
                }
            }
        }
    }

    // -------------------------------------------------------------------------
    // Track queries
    // -------------------------------------------------------------------------

    pub fn selected_track(&self) -> Option<&Track> {
        self.tracks.iter().find(|t| Some(t.id) == self.selected_track_id)
    }

    pub fn current_track(&self) -> Option<&Track> {
        self.tracks.iter().find(|t| Some(t.id) == self.now_playing_track_id)
    }

    pub fn filtered_tracks(&self) -> Vec<&Track> {
        let query = self.search_query.trim().to_ascii_lowercase();
        self.tracks
            .iter()
            .filter(|track| {
                let focus_ok = match &self.browse_focus {
                    BrowseFocus::All => true,
                    BrowseFocus::Artist(a) => track.artist.eq_ignore_ascii_case(a),
                    BrowseFocus::Album { artist, album } => {
                        track.artist.eq_ignore_ascii_case(artist)
                            && track.album.eq_ignore_ascii_case(album)
                    }
                };
                if !focus_ok {
                    return false;
                }
                if query.is_empty() {
                    return true;
                }
                let haystack = format!(
                    "{} {} {}",
                    track.title.to_ascii_lowercase(),
                    track.artist.to_ascii_lowercase(),
                    track.album.to_ascii_lowercase()
                );
                haystack.contains(&query)
            })
            .collect()
    }

    pub fn sorted_library_tracks(&self) -> Vec<Track> {
        let mut tracks: Vec<Track> = self.filtered_tracks().into_iter().cloned().collect();
        tracks.sort_by(|a, b| {
            let ord = match self.library_sort_key {
                LibrarySortKey::Title => a.title.cmp(&b.title),
                LibrarySortKey::Artist => a.artist.cmp(&b.artist),
                LibrarySortKey::Album => a.album.cmp(&b.album),
                LibrarySortKey::Duration => a.duration_seconds.cmp(&b.duration_seconds),
            };
            let tiebreak = ord
                .then_with(|| a.artist.cmp(&b.artist))
                .then_with(|| a.album.cmp(&b.album))
                .then_with(|| a.title.cmp(&b.title));
            if self.library_sort_ascending { tiebreak } else { tiebreak.reverse() }
        });
        tracks
    }

    pub fn library_albums(&self) -> Vec<AlbumSummary> {
        summarize_albums(self.filtered_tracks())
    }

    pub fn library_artists(&self) -> Vec<ArtistSummary> {
        summarize_artists(self.filtered_tracks())
    }

    pub fn queue_tracks(&self) -> Vec<&Track> {
        self.queue
            .iter()
            .filter_map(|id| self.tracks.iter().find(|t| t.id == *id))
            .collect()
    }

    // -------------------------------------------------------------------------
    // Home personalisation
    // -------------------------------------------------------------------------

    pub fn refresh_home_personalization(&mut self) {
        self.recently_played =
            db::list_recently_played_tracks(&self.db_path, 12).unwrap_or_default();
        self.pinned_albums = db::list_pinned_albums(&self.db_path).unwrap_or_default();
    }

    pub fn library_stats(&self) -> LibraryStats {
        let artist_count = self
            .tracks
            .iter()
            .map(|t| t.artist.to_ascii_lowercase())
            .collect::<HashSet<_>>()
            .len();
        let album_count = self
            .tracks
            .iter()
            .map(|t| {
                format!("{}::{}", t.artist.to_ascii_lowercase(), t.album.to_ascii_lowercase())
            })
            .collect::<HashSet<_>>()
            .len();

        let mut artist_totals = HashMap::<String, usize>::new();
        for track in &self.tracks {
            *artist_totals.entry(track.artist.clone()).or_default() += 1;
        }
        let mut top_artists: Vec<(String, usize)> = artist_totals.into_iter().collect();
        top_artists.sort_by(|a, b| b.1.cmp(&a.1).then_with(|| a.0.cmp(&b.0)));
        top_artists.truncate(5);

        let mut recent_tracks = self.tracks.clone();
        recent_tracks.sort_by(|a, b| b.created_at.cmp(&a.created_at));
        recent_tracks.truncate(8);

        let recent_albums = summarize_albums(recent_tracks.iter())
            .into_iter()
            .take(6)
            .collect();

        LibraryStats {
            track_count: self.tracks.len(),
            artist_count,
            album_count,
            top_artists,
            recent_albums,
        }
    }

    pub fn search_results(&self) -> SearchResults {
        let tracks: Vec<Track> = self.filtered_tracks().into_iter().cloned().collect();
        SearchResults::from_tracks(tracks)
    }

    pub fn selected_album_key(&self) -> Option<(String, String)> {
        self.selected_track()
            .map(|t| (t.artist.clone(), t.album.clone()))
    }

    pub fn is_album_pinned(&self, artist: &str, album: &str) -> bool {
        self.pinned_albums
            .iter()
            .any(|(a, b)| a.eq_ignore_ascii_case(artist) && b.eq_ignore_ascii_case(album))
    }

    pub fn toggle_selected_album_pin(&mut self) {
        let Some((artist, album)) = self.selected_album_key() else {
            self.status = "Select a track first to pin or unpin its album.".to_string();
            return;
        };
        let should_pin = !self.is_album_pinned(&artist, &album);
        match db::set_album_pinned(&self.db_path, &artist, &album, should_pin) {
            Ok(()) => {
                self.refresh_home_personalization();
                self.status = if should_pin {
                    format!("Pinned album: {artist} - {album}")
                } else {
                    format!("Unpinned album: {artist} - {album}")
                };
            }
            Err(e) => self.status = format!("Unable to update pin state: {e}"),
        }
    }

    pub fn pinned_album_summaries(&self) -> Vec<AlbumSummary> {
        let all_albums = summarize_albums(self.tracks.iter());
        self.pinned_albums
            .iter()
            .filter_map(|(artist, album)| {
                all_albums.iter().find(|c| {
                    c.artist.eq_ignore_ascii_case(artist) && c.title.eq_ignore_ascii_case(album)
                })
            })
            .cloned()
            .collect()
    }

    pub fn generated_mixes(&self) -> Vec<MixSummary> {
        let mut mixes = Vec::new();
        if self.recently_played.is_empty() {
            return mixes;
        }

        let mut seen = HashSet::new();
        let replay_ids: Vec<i64> = self
            .recently_played
            .iter()
            .filter(|t| seen.insert(t.id))
            .map(|t| t.id)
            .collect();
        if !replay_ids.is_empty() {
            mixes.push(MixSummary {
                name: "Replay Mix".to_string(),
                description: "Your most recently played songs.".to_string(),
                track_ids: replay_ids,
            });
        }

        let mut artist_scores: HashMap<String, usize> = HashMap::new();
        for track in &self.recently_played {
            *artist_scores.entry(track.artist.clone()).or_default() += 1;
        }
        if let Some((artist_name, _)) = artist_scores.into_iter().max_by_key(|(_, c)| *c) {
            let ids: Vec<i64> = self
                .tracks
                .iter()
                .filter(|t| t.artist.eq_ignore_ascii_case(&artist_name))
                .take(30)
                .map(|t| t.id)
                .collect();
            if !ids.is_empty() {
                mixes.push(MixSummary {
                    name: format!("{artist_name} Mix"),
                    description: "Generated from your top local-history artist.".to_string(),
                    track_ids: ids,
                });
            }
        }

        let album_keys: HashSet<(String, String)> = self
            .recently_played
            .iter()
            .map(|t| (t.artist.clone(), t.album.clone()))
            .collect();
        let mut bounce_ids = Vec::new();
        let mut bounce_seen = HashSet::new();
        for track in &self.tracks {
            if album_keys
                .iter()
                .any(|(a, b)| track.artist.eq_ignore_ascii_case(a) && track.album.eq_ignore_ascii_case(b))
                && bounce_seen.insert(track.id)
            {
                bounce_ids.push(track.id);
            }
            if bounce_ids.len() >= 40 {
                break;
            }
        }
        if !bounce_ids.is_empty() {
            mixes.push(MixSummary {
                name: "Album Bounce".to_string(),
                description: "Songs from albums you recently touched.".to_string(),
                track_ids: bounce_ids,
            });
        }

        mixes
    }

    // -------------------------------------------------------------------------
    // Session persistence
    // -------------------------------------------------------------------------

    pub fn save_session(&self) {
        let volume = self
            .audio_player
            .as_ref()
            .map(|p| p.volume())
            .unwrap_or(1.0);
        let _ = db::set_setting(&self.db_path, "volume", &volume.to_string());
        let _ = db::set_setting(
            &self.db_path,
            "shuffle",
            if self.shuffle { "1" } else { "0" },
        );
        let repeat_str = match self.repeat {
            RepeatMode::Off => "off",
            RepeatMode::Queue => "queue",
            RepeatMode::Track => "track",
        };
        let _ = db::set_setting(&self.db_path, "repeat", repeat_str);
        let nav_str = match self.active_nav {
            NavSection::Home => "home",
            NavSection::Search => "search",
            NavSection::Library => "library",
        };
        let _ = db::set_setting(&self.db_path, "active_nav", nav_str);
        if let Some(id) = self.now_playing_track_id {
            let _ = db::set_setting(&self.db_path, "last_track_id", &id.to_string());
        }
    }

    fn restore_session(&mut self) {
        if let Ok(Some(vol)) = db::get_setting(&self.db_path, "volume") {
            if let Ok(v) = vol.parse::<f32>() {
                if let Some(player) = self.audio_player.as_mut() {
                    player.set_volume(v);
                }
            }
        }
        if let Ok(Some(s)) = db::get_setting(&self.db_path, "shuffle") {
            self.shuffle = s == "1";
        }
        if let Ok(Some(r)) = db::get_setting(&self.db_path, "repeat") {
            self.repeat = match r.as_str() {
                "queue" => RepeatMode::Queue,
                "track" => RepeatMode::Track,
                _ => RepeatMode::Off,
            };
        }
        if let Ok(Some(nav)) = db::get_setting(&self.db_path, "active_nav") {
            self.active_nav = match nav.as_str() {
                "search" => NavSection::Search,
                "library" => NavSection::Library,
                _ => NavSection::Home,
            };
        }
        if let Ok(Some(id_str)) = db::get_setting(&self.db_path, "last_track_id") {
            if let Ok(id) = id_str.parse::<i64>() {
                if self.tracks.iter().any(|t| t.id == id) {
                    self.selected_track_id = Some(id);
                }
            }
        }
    }

    // -------------------------------------------------------------------------
    // Album art (lazy texture loading)
    // -------------------------------------------------------------------------

    /// Returns a reference to a loaded texture, loading from DB + decoding on first access.
    pub fn get_art_texture(
        &mut self,
        ctx: &egui::Context,
        artist: &str,
        album: &str,
    ) -> Option<egui::TextureHandle> {
        let key = format!("{artist}\x00{album}");
        if self.art_cache.contains_key(&key) {
            return self.art_cache.get(&key).cloned();
        }
        // Not cached — try loading from the database.
        let bytes = db::get_album_art(&self.db_path, artist, album).ok().flatten()?;
        let img = image::load_from_memory(&bytes).ok()?;
        let rgba = img.to_rgba8();
        let (w, h) = rgba.dimensions();
        let pixels: Vec<egui::Color32> = rgba
            .pixels()
            .map(|p| egui::Color32::from_rgba_unmultiplied(p[0], p[1], p[2], p[3]))
            .collect();
        let color_image = egui::ColorImage::new([w as usize, h as usize], pixels);
        let handle = ctx.load_texture(&key, color_image, egui::TextureOptions::LINEAR);
        self.art_cache.insert(key.clone(), handle);
        self.art_cache.get(&key).cloned()
    }

    // -------------------------------------------------------------------------
    // Song View
    // -------------------------------------------------------------------------

    pub fn open_song_view(&mut self) {
        if self.now_playing_track_id.is_none() {
            return;
        }
        self.song_view_open = true;
        self.load_lyrics_for_current_track();
    }

    pub fn close_song_view(&mut self) {
        self.song_view_open = false;
    }

    pub fn request_waveform(&mut self) {
        if self.waveform_data.is_some() || self.waveform_receiver.is_some() {
            return; // already loading or loaded
        }
        let Some(track) = self.current_track().cloned() else { return };
        let path = track.file_path.clone();
        let (tx, rx) = mpsc::channel();
        self.waveform_receiver = Some(rx);
        thread::spawn(move || {
            let _ = tx.send(compute_waveform(&path));
        });
    }

    pub fn load_lyrics_for_current_track(&mut self) {
        self.lyrics_data = None;
        let Some(track) = self.current_track().cloned() else { return };

        // 1. Look for a .lrc sidecar file next to the audio file.
        let lrc_path = std::path::PathBuf::from(&track.file_path)
            .with_extension("lrc");
        if lrc_path.exists() {
            if let Ok(lrc_text) = std::fs::read_to_string(&lrc_path) {
                let parsed = parse_lrc(&lrc_text);
                if !parsed.is_empty() {
                    self.lyrics_data = Some(LyricsData::Synced(parsed));
                    return;
                }
            }
        }

        // 2. Fall back to embedded tag lyrics via lofty.
        use lofty::{file::TaggedFileExt, prelude::Accessor, probe::Probe, tag::ItemKey};
        if let Ok(probe) = Probe::open(&track.file_path) {
            if let Some(tagged) = probe.guess_file_type().ok().and_then(|p| p.read().ok()) {
                if let Some(tag) = tagged.primary_tag() {
                    let text = tag
                        .get_string(ItemKey::UnsyncLyrics)
                        .or_else(|| tag.get_string(ItemKey::Lyrics))
                        .map(str::to_owned);
                    if let Some(t) = text {
                        if !t.trim().is_empty() {
                            self.lyrics_data = Some(LyricsData::Plain(t));
                        }
                    }
                }
            }
        }
    }

    pub fn poll_waveform(&mut self) {
        if let Some(rx) = &self.waveform_receiver {
            if let Ok(data) = rx.try_recv() {
                self.waveform_data = Some(data);
                self.waveform_receiver = None;
            }
        }
    }

    // -------------------------------------------------------------------------
    // Global keyboard shortcuts
    // -------------------------------------------------------------------------

    pub fn handle_global_shortcuts(&mut self, ctx: &egui::Context) {
        // Don't steal keys while a text field is focused.
        if ctx.wants_keyboard_input() {
            return;
        }
        let (space, left, right, m_key, l_key, s_key) = ctx.input(|i| {
            (
                i.key_pressed(egui::Key::Space),
                i.key_pressed(egui::Key::ArrowLeft),
                i.key_pressed(egui::Key::ArrowRight),
                i.key_pressed(egui::Key::M),
                i.key_pressed(egui::Key::L),
                i.key_pressed(egui::Key::S),
            )
        });
        if space { self.toggle_playback(); }
        if left  { self.play_queue_offset(-1); }
        if right { self.play_queue_offset(1); }
        if m_key { self.toggle_mute(); }
        if l_key { self.cycle_repeat(); }
        if s_key { self.toggle_shuffle(); }
    }

    pub fn toggle_mute(&mut self) {
        let Some(player) = self.audio_player.as_mut() else { return };
        if player.volume() > 0.0 {
            player.set_volume(0.0);
        } else {
            player.set_volume(0.7);
        }
    }

    pub fn toggle_shuffle(&mut self) {
        self.shuffle = !self.shuffle;
    }

    pub fn cycle_repeat(&mut self) {
        self.repeat = self.repeat.next();
    }

    // -------------------------------------------------------------------------
    // Context-menu track actions
    // -------------------------------------------------------------------------

    pub fn handle_track_action(&mut self, action: TrackAction, track_id: i64) {
        match action {
            TrackAction::PlayNow => {
                self.selected_track_id = Some(track_id);
                self.play_selected_from_visible();
            }
            TrackAction::AddToQueueEnd => {
                self.queue.push(track_id);
                if self.queue_position.is_none() && !self.queue.is_empty() {
                    self.queue_position = Some(0);
                    self.play_track(track_id);
                } else {
                    self.status = "Added to end of queue.".to_string();
                }
            }
            TrackAction::GoToAlbum => {
                if let Some(track) = self.tracks.iter().find(|t| t.id == track_id).cloned() {
                    self.open_album(&track.artist, &track.album);
                }
            }
            TrackAction::GoToArtist => {
                if let Some(track) = self.tracks.iter().find(|t| t.id == track_id).cloned() {
                    self.open_artist(&track.artist);
                }
            }
        }
    }

    // -------------------------------------------------------------------------
    // Playlist management
    // -------------------------------------------------------------------------

    pub fn refresh_playlists(&mut self) {
        self.playlists = db::list_playlists(&self.db_path).unwrap_or_default();
    }

    pub fn playlist_views(&self) -> Vec<PlaylistView> {
        self.playlists
            .iter()
            .map(|pl| {
                let track_ids =
                    db::list_playlist_track_ids(&self.db_path, pl.id).unwrap_or_default();
                PlaylistView { playlist: pl.clone(), track_ids }
            })
            .collect()
    }

    pub fn active_playlist_view(&self) -> Option<PlaylistView> {
        let id = self.active_playlist_id?;
        let pl = self.playlists.iter().find(|p| p.id == id)?.clone();
        let track_ids = db::list_playlist_track_ids(&self.db_path, id).unwrap_or_default();
        Some(PlaylistView { playlist: pl, track_ids })
    }

    pub fn create_playlist(&mut self) {
        let name = self.new_playlist_name.trim().to_string();
        if name.is_empty() {
            self.status = "Enter a playlist name first.".to_string();
            return;
        }
        match db::create_playlist(&self.db_path, &name) {
            Ok(id) => {
                self.refresh_playlists();
                self.active_playlist_id = Some(id);
                self.library_view = LibraryView::Playlists;
                self.new_playlist_name.clear();
                self.show_create_playlist = false;
                self.status = format!("Created playlist \"{name}\".");
            }
            Err(e) => self.status = format!("Failed to create playlist: {e}"),
        }
    }

    pub fn delete_active_playlist(&mut self) {
        let Some(id) = self.active_playlist_id else { return };
        let name = self
            .playlists
            .iter()
            .find(|p| p.id == id)
            .map(|p| p.name.clone())
            .unwrap_or_default();
        if db::delete_playlist(&self.db_path, id).is_ok() {
            self.refresh_playlists();
            self.active_playlist_id = self.playlists.first().map(|p| p.id);
            self.status = format!("Deleted playlist \"{name}\".");
        }
    }

    pub fn add_track_to_playlist(&mut self, playlist_id: i64, track_id: i64) {
        match db::add_track_to_playlist(&self.db_path, playlist_id, track_id) {
            Ok(()) => {
                let pl_name = self
                    .playlists
                    .iter()
                    .find(|p| p.id == playlist_id)
                    .map(|p| p.name.as_str())
                    .unwrap_or("playlist");
                let track_title = self
                    .tracks
                    .iter()
                    .find(|t| t.id == track_id)
                    .map(|t| t.title.as_str())
                    .unwrap_or("track");
                self.status = format!("Added \"{track_title}\" to \"{pl_name}\".");
            }
            Err(e) => self.status = format!("Could not add track: {e}"),
        }
        self.add_to_playlist_track_id = None;
    }

    pub fn remove_track_from_active_playlist(&mut self, track_id: i64) {
        let Some(pl_id) = self.active_playlist_id else { return };
        let _ = db::remove_track_from_playlist(&self.db_path, pl_id, track_id);
    }

    // -------------------------------------------------------------------------
    // Playback control
    // -------------------------------------------------------------------------

    pub fn play_track(&mut self, track_id: i64) {
        let track = self.tracks.iter().find(|t| t.id == track_id).cloned();
        let Some(track) = track else {
            self.status = "The selected track no longer exists in the library.".to_string();
            return;
        };
        let Some(player) = self.audio_player.as_mut() else {
            self.status = "Audio output is not available on this system.".to_string();
            return;
        };
        match player.play_file(&track.file_path, track.duration_seconds) {
            Ok(()) => {
                self.now_playing_track_id = Some(track.id);
                self.selected_track_id = Some(track.id);
                // Reset per-track song-view data.
                self.waveform_data = None;
                self.waveform_receiver = None;
                self.lyrics_data = None;
                if self.song_view_open {
                    self.load_lyrics_for_current_track();
                }
                let _ = db::record_track_play(&self.db_path, track.id);
                self.refresh_home_personalization();
                self.status = format!("Playing {} - {}", track.artist, track.title);
            }
            Err(e) => self.status = format!("Playback failed: {e}"),
        }
    }

    pub fn play_track_list(&mut self, mut track_ids: Vec<i64>, start_index: usize) {
        if track_ids.is_empty() {
            self.status = "No tracks available to play in this selection.".to_string();
            return;
        }
        let idx = if self.shuffle {
            // Fisher-Yates: put the clicked track first, then shuffle the rest.
            let clicked = track_ids.remove(start_index.min(track_ids.len() - 1));
            use std::collections::hash_map::DefaultHasher;
            use std::hash::{Hash, Hasher};
            // Deterministic-ish shuffle seeded by track id (no rand dep needed).
            let seed = clicked as u64;
            for i in (1..track_ids.len()).rev() {
                let mut h = DefaultHasher::new();
                (seed ^ i as u64).hash(&mut h);
                let j = (h.finish() as usize) % (i + 1);
                track_ids.swap(i, j);
            }
            track_ids.insert(0, clicked);
            0
        } else {
            start_index.min(track_ids.len() - 1)
        };
        let id = track_ids[idx];
        self.queue = track_ids;
        self.queue_position = Some(idx);
        self.play_track(id);
    }

    pub fn play_selected_from_visible(&mut self) {
        let Some(track_id) = self.selected_track_id else {
            self.status = "Select a track before playing.".to_string();
            return;
        };

        use crate::models::{LibraryView, NavSection};
        let visible: Vec<i64> = if self.active_nav == NavSection::Library
            && self.library_view == LibraryView::Songs
        {
            self.sorted_library_tracks().into_iter().map(|t| t.id).collect()
        } else if self.active_nav == NavSection::Search {
            self.search_results().tracks.into_iter().map(|t| t.id).collect()
        } else {
            self.filtered_tracks().into_iter().map(|t| t.id).collect()
        };

        if visible.is_empty() {
            self.status = "No visible tracks are available to queue.".to_string();
            return;
        }
        let pos = visible.iter().position(|id| *id == track_id).unwrap_or(0);
        self.queue = visible;
        self.queue_position = Some(pos);
        self.play_track(track_id);
    }

    pub fn play_queue_offset(&mut self, offset: isize) {
        let Some(pos) = self.queue_position else {
            self.status = "Start playback from the library to build a queue first.".to_string();
            return;
        };
        let next = pos as isize + offset;
        let queue_len = self.queue.len() as isize;

        let resolved = if next < 0 || next >= queue_len {
            match self.repeat {
                RepeatMode::Queue if !self.queue.is_empty() => {
                    // Wrap around.
                    ((next % queue_len + queue_len) % queue_len) as usize
                }
                RepeatMode::Track => pos, // stay on current
                _ => {
                    self.status = "Queue boundary reached.".to_string();
                    return;
                }
            }
        } else {
            next as usize
        };

        self.queue_position = Some(resolved);
        let id = self.queue[resolved];
        self.play_track(id);
    }

    pub fn toggle_playback(&mut self) {
        let Some(player) = self.audio_player.as_mut() else {
            self.status = "Audio output is not available on this system.".to_string();
            return;
        };
        match player.toggle_pause() {
            Some(true) => self.status = "Playback paused.".to_string(),
            Some(false) => self.status = "Playback resumed.".to_string(),
            None => self.play_selected_from_visible(),
        }
    }

    // -------------------------------------------------------------------------
    // Navigation helpers
    // -------------------------------------------------------------------------

    pub fn open_artist(&mut self, artist: &str) {
        self.browse_focus = BrowseFocus::Artist(artist.to_string());
        self.library_view = LibraryView::Songs;
        self.active_nav = NavSection::Library;
        self.status = format!("Browsing artist: {artist}");
    }

    pub fn open_album(&mut self, artist: &str, album: &str) {
        self.browse_focus = BrowseFocus::Album {
            artist: artist.to_string(),
            album: album.to_string(),
        };
        self.library_view = LibraryView::Songs;
        self.active_nav = NavSection::Library;
        self.status = format!("Browsing album: {artist} - {album}");
    }

    pub fn clear_browse_focus(&mut self) {
        self.browse_focus = BrowseFocus::All;
    }
}

impl eframe::App for PlaymuApp {
    fn update(&mut self, ctx: &egui::Context, _frame: &mut eframe::Frame) {
        self.anim_time = ctx.input(|i| i.time);
        self.process_background_events();
        self.poll_waveform();

        // Close song view with Escape.
        if self.song_view_open && ctx.input(|i| i.key_pressed(egui::Key::Escape)) {
            self.close_song_view();
        }

        // Auto-advance when the current track ends.
        if self.audio_player.as_mut().is_some_and(AudioPlayer::take_finished) {
            if self.repeat == RepeatMode::Track {
                // Re-play the same track.
                if let Some(id) = self.queue_position.and_then(|p| self.queue.get(p)).copied() {
                    self.play_track(id);
                }
            } else {
                self.play_queue_offset(1);
            }
        }

        // Live-repaint the progress bar while playing.
        if self
            .audio_player
            .as_ref()
            .is_some_and(|p| p.has_track() && !p.is_paused())
        {
            ctx.request_repaint_after(Duration::from_millis(250));
        }

        self.handle_global_shortcuts(ctx);
        self.draw_sidebar(ctx);
        if self.queue_panel_open {
            self.draw_right_panel(ctx);
        }

        egui::CentralPanel::default().show(ctx, |ui| {
            ui.add_space(12.0);
            self.draw_top_bar(ui);
            self.handle_search_shortcuts(ctx);
            ui.add_space(16.0);

            match self.active_nav {
                NavSection::Home => self.draw_home(ui),
                NavSection::Search => self.draw_search(ui),
                NavSection::Library => self.draw_library(ui),
            }
        });

        self.draw_bottom_bar(ctx);

        // Song view overlay — drawn last so it sits on top of everything.
        self.draw_song_view_overlay(ctx);
    }

    fn save(&mut self, _storage: &mut dyn eframe::Storage) {
        self.save_session();
    }
}

/// Internal helper: session persistence uses `image` for art decoding.
#[allow(unused_imports)]
use image as _image;

// ---------------------------------------------------------------------------
// Waveform & lyrics helpers (called from background threads)
// ---------------------------------------------------------------------------

/// Decode the audio file and return ~800 normalised RMS amplitude values.
fn compute_waveform(path: &str) -> Vec<f32> {
    use std::fs::File;
    use std::io::BufReader;
    use rodio::{Decoder, Source};

    const BARS: usize = 800;
    const MAX_SAMPLES: usize = 10_000_000; // cap memory

    let fallback = vec![0.0f32; BARS];

    let Ok(file) = File::open(path) else { return fallback };
    let Ok(decoder) = Decoder::new(BufReader::new(file)) else { return fallback };
    let channels = decoder.channels() as usize;

    // Collect mono-mixed samples, capped.
    let samples: Vec<f32> = decoder
        .take(MAX_SAMPLES)
        .collect::<Vec<i16>>()
        .chunks(channels)
        .map(|frame| {
            let sum: i32 = frame.iter().map(|&s| s as i32).sum();
            (sum as f32 / (channels as f32 * 32768.0)).clamp(-1.0, 1.0)
        })
        .collect();

    if samples.is_empty() {
        return fallback;
    }

    let window = (samples.len() / BARS).max(1);
    let mut result: Vec<f32> = samples
        .chunks(window)
        .take(BARS)
        .map(|chunk| {
            let rms = (chunk.iter().map(|s| s * s).sum::<f32>() / chunk.len() as f32).sqrt();
            rms
        })
        .collect();

    // Normalize to [0, 1].
    let peak = result.iter().cloned().fold(0.0f32, f32::max).max(0.001);
    for v in &mut result {
        *v /= peak;
    }
    // Pad if necessary.
    while result.len() < BARS {
        result.push(0.0);
    }
    result
}

/// Parse LRC-format lyrics into (timestamp_seconds, line) pairs.
fn parse_lrc(text: &str) -> Vec<(f64, String)> {
    let mut lines = Vec::new();
    for raw in text.lines() {
        let raw = raw.trim();
        if !raw.starts_with('[') {
            continue;
        }
        // Multiple timestamps per line are supported: [mm:ss.xx][mm:ss.xx]text
        let mut rest = raw;
        let mut timestamps: Vec<f64> = Vec::new();
        loop {
            let Some(close) = rest.find(']') else { break };
            let tag = &rest[1..close];
            rest = &rest[close + 1..];
            // Try to parse as mm:ss.xx
            if let Some(ts) = parse_lrc_timestamp(tag) {
                timestamps.push(ts);
            } else {
                break; // Not a timestamp tag — metadata tag, skip.
            }
        }
        let lyric_line = rest.trim().to_string();
        for ts in timestamps {
            lines.push((ts, lyric_line.clone()));
        }
    }
    lines.sort_by(|a, b| a.0.partial_cmp(&b.0).unwrap_or(std::cmp::Ordering::Equal));
    lines
}

fn parse_lrc_timestamp(s: &str) -> Option<f64> {
    // "mm:ss.xx" or "mm:ss.xxx"
    let colon = s.find(':')?;
    let minutes: f64 = s[..colon].parse().ok()?;
    let seconds: f64 = s[colon + 1..].parse().ok()?;
    Some(minutes * 60.0 + seconds)
}
