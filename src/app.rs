use std::{
    collections::{HashMap, HashSet},
    fs::File,
    io::BufReader,
    path::PathBuf,
    sync::mpsc::{self, Receiver},
    thread,
};

use eframe::egui::{self, Color32, RichText, TextEdit};
use rodio::{Decoder, OutputStream, OutputStreamHandle, Sink};

use crate::{
    db::{self, Track},
    library::{self, ScanSummary},
};

const ACCENT_GREEN: Color32 = Color32::from_rgb(29, 185, 84);
const ACCENT_GREEN_SOFT: Color32 = Color32::from_rgb(37, 102, 62);
const PANEL_DARK: Color32 = Color32::from_rgb(15, 18, 18);
const PANEL_SOFT: Color32 = Color32::from_rgb(24, 28, 29);
const SURFACE: Color32 = Color32::from_rgb(31, 36, 38);
const SURFACE_HOVER: Color32 = Color32::from_rgb(39, 46, 48);
const TEXT_MUTED: Color32 = Color32::from_rgb(155, 163, 166);

#[derive(Clone, Copy, PartialEq, Eq)]
enum NavSection {
    Home,
    Search,
    Library,
}

impl NavSection {
    fn label(self) -> &'static str {
        match self {
            Self::Home => "Home",
            Self::Search => "Search",
            Self::Library => "Your Library",
        }
    }

    fn subtitle(self) -> &'static str {
        match self {
            Self::Home => "A focused landing space for your own collection.",
            Self::Search => "Jump to any song, artist, or album in your local library.",
            Self::Library => "Dense browsing for tracks, artists, and albums.",
        }
    }
}

#[derive(Clone)]
struct LibraryStats {
    track_count: usize,
    artist_count: usize,
    album_count: usize,
    top_artists: Vec<(String, usize)>,
    recent_tracks: Vec<Track>,
}

pub struct PlaymuApp {
    db_path: PathBuf,
    source_input: String,
    search_query: String,
    tracks: Vec<Track>,
    selected_track_id: Option<i64>,
    now_playing_track_id: Option<i64>,
    queue: Vec<i64>,
    queue_position: Option<usize>,
    active_nav: NavSection,
    status: String,
    scan_receiver: Option<Receiver<Result<ScanSummary, String>>>,
    is_scanning: bool,
    audio_player: Option<AudioPlayer>,
}

impl PlaymuApp {
    pub fn new(cc: &eframe::CreationContext<'_>) -> Self {
        configure_theme(&cc.egui_ctx);

        let db_path = std::env::current_dir()
            .unwrap_or_else(|_| PathBuf::from("."))
            .join("playmu.db");

        let mut status = String::from("Add a music folder path, import it, then select a track to play.");

        if let Err(error) = db::init_database(&db_path) {
            status = format!("Database initialization failed: {error}");
        }

        let tracks = db::list_tracks(&db_path).unwrap_or_default();
        let audio_player = match AudioPlayer::new() {
            Ok(player) => Some(player),
            Err(error) => {
                status = format!("Audio output unavailable: {error}");
                None
            }
        };

        Self {
            db_path,
            source_input: String::new(),
            search_query: String::new(),
            tracks,
            selected_track_id: None,
            now_playing_track_id: None,
            queue: Vec::new(),
            queue_position: None,
            active_nav: NavSection::Home,
            status,
            scan_receiver: None,
            is_scanning: false,
            audio_player,
        }
    }

    fn start_scan(&mut self) {
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
                .map_err(|error| error.to_string());
            let _ = tx.send(result);
        });
    }

    fn refresh_after_scan(&mut self, summary: ScanSummary) {
        self.tracks = db::list_tracks(&self.db_path).unwrap_or_default();
        if self.selected_track_id.is_none() {
            self.selected_track_id = self.tracks.first().map(|track| track.id);
        }
        self.status = format!(
            "Imported {} tracks from {}. Removed {} stale entries.",
            summary.imported_tracks, summary.source_folder, summary.removed_tracks
        );
        self.is_scanning = false;
    }

    fn process_background_events(&mut self) {
        if let Some(receiver) = &self.scan_receiver {
            match receiver.try_recv() {
                Ok(Ok(summary)) => {
                    self.scan_receiver = None;
                    self.refresh_after_scan(summary);
                }
                Ok(Err(error)) => {
                    self.scan_receiver = None;
                    self.is_scanning = false;
                    self.status = format!("Scan failed: {error}");
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

    fn selected_track(&self) -> Option<&Track> {
        let selected_id = self.selected_track_id?;
        self.tracks.iter().find(|track| track.id == selected_id)
    }

    fn current_track(&self) -> Option<&Track> {
        let current_id = self.now_playing_track_id?;
        self.tracks.iter().find(|track| track.id == current_id)
    }

    fn visible_tracks(&self) -> Vec<&Track> {
        let query = self.search_query.trim().to_ascii_lowercase();
        if query.is_empty() {
            return self.tracks.iter().collect();
        }

        self.tracks
            .iter()
            .filter(|track| {
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

    fn library_stats(&self) -> LibraryStats {
        let artist_count = self
            .tracks
            .iter()
            .map(|track| track.artist.to_ascii_lowercase())
            .collect::<HashSet<_>>()
            .len();
        let album_count = self
            .tracks
            .iter()
            .map(|track| format!("{}::{}", track.artist.to_ascii_lowercase(), track.album.to_ascii_lowercase()))
            .collect::<HashSet<_>>()
            .len();

        let mut artist_totals = HashMap::<String, usize>::new();
        for track in &self.tracks {
            *artist_totals.entry(track.artist.clone()).or_default() += 1;
        }

        let mut top_artists: Vec<(String, usize)> = artist_totals.into_iter().collect();
        top_artists.sort_by(|left, right| right.1.cmp(&left.1).then_with(|| left.0.cmp(&right.0)));
        top_artists.truncate(5);

        let mut recent_tracks = self.tracks.clone();
        recent_tracks.sort_by(|left, right| right.created_at.cmp(&left.created_at));
        recent_tracks.truncate(8);

        LibraryStats {
            track_count: self.tracks.len(),
            artist_count,
            album_count,
            top_artists,
            recent_tracks,
        }
    }

    fn play_selected_from_visible(&mut self) {
        let Some(track_id) = self.selected_track_id else {
            self.status = "Select a track before playing.".to_string();
            return;
        };

        let visible_track_ids: Vec<i64> = self.visible_tracks().into_iter().map(|track| track.id).collect();
        if visible_track_ids.is_empty() {
            self.status = "No visible tracks are available to queue.".to_string();
            return;
        }

        let queue_position = visible_track_ids.iter().position(|id| *id == track_id).unwrap_or(0);
        self.queue = visible_track_ids;
        self.queue_position = Some(queue_position);
        self.play_track(track_id);
    }

    fn play_track(&mut self, track_id: i64) {
        let track = self.tracks.iter().find(|track| track.id == track_id).cloned();
        let Some(track) = track else {
            self.status = "The selected track no longer exists in the library.".to_string();
            return;
        };

        let Some(player) = self.audio_player.as_mut() else {
            self.status = "Audio output is not available on this system.".to_string();
            return;
        };

        match player.play_file(&track.file_path) {
            Ok(()) => {
                self.now_playing_track_id = Some(track.id);
                self.selected_track_id = Some(track.id);
                self.status = format!("Playing {} - {}", track.artist, track.title);
            }
            Err(error) => {
                self.status = format!("Playback failed: {error}");
            }
        }
    }

    fn play_queue_offset(&mut self, offset: isize) {
        let Some(position) = self.queue_position else {
            self.status = "Start playback from the library to build a queue first.".to_string();
            return;
        };

        let next_position = position as isize + offset;
        if next_position < 0 || next_position >= self.queue.len() as isize {
            self.status = "Queue boundary reached.".to_string();
            return;
        }

        let next_position = next_position as usize;
        self.queue_position = Some(next_position);
        let next_track_id = self.queue[next_position];
        self.play_track(next_track_id);
    }

    fn toggle_playback(&mut self) {
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

    fn queue_tracks(&self) -> Vec<&Track> {
        self.queue
            .iter()
            .filter_map(|track_id| self.tracks.iter().find(|track| track.id == *track_id))
            .collect()
    }

    fn nav_button(ui: &mut egui::Ui, selected: bool, label: &str) -> egui::Response {
        let fill = if selected { ACCENT_GREEN_SOFT } else { SURFACE };
        let text = if selected { Color32::WHITE } else { TEXT_MUTED };
        ui.add_sized(
            [ui.available_width(), 40.0],
            egui::Button::new(RichText::new(label).color(text).strong()).fill(fill),
        )
    }

    fn draw_sidebar(&mut self, ctx: &egui::Context) {
        egui::SidePanel::left("sidebar")
            .resizable(false)
            .min_width(250.0)
            .show(ctx, |ui| {
                ui.add_space(12.0);
                ui.label(RichText::new("PLAYMU").size(28.0).strong().color(Color32::WHITE));
                ui.label(RichText::new("Spotify-like local music for Arch Linux").color(TEXT_MUTED));
                ui.add_space(16.0);

                for nav in [NavSection::Home, NavSection::Search, NavSection::Library] {
                    if Self::nav_button(ui, self.active_nav == nav, nav.label()).clicked() {
                        self.active_nav = nav;
                    }
                    ui.add_space(6.0);
                }

                ui.add_space(18.0);
                ui.label(RichText::new("Import Local Music").strong());
                ui.label(RichText::new("Point Playmu at a folder on this machine.").color(TEXT_MUTED));
                ui.add(
                    TextEdit::singleline(&mut self.source_input)
                        .hint_text("/home/you/Music")
                        .desired_width(f32::INFINITY),
                );

                if ui
                    .add_enabled(!self.is_scanning, egui::Button::new("Import Folder").fill(ACCENT_GREEN))
                    .clicked()
                {
                    self.start_scan();
                }

                if self.is_scanning {
                    ui.add_space(8.0);
                    ui.horizontal(|ui| {
                        ui.spinner();
                        ui.label("Scanning library...");
                    });
                }

                ui.add_space(20.0);
                ui.separator();
                ui.add_space(10.0);
                ui.label(RichText::new("Database").strong());
                ui.small(self.db_path.display().to_string());
                ui.add_space(10.0);
                ui.label(RichText::new("Status").strong());
                ui.label(RichText::new(&self.status).color(TEXT_MUTED));
            });
    }

    fn draw_right_panel(&mut self, ctx: &egui::Context) {
        egui::SidePanel::right("queue_panel")
            .default_width(300.0)
            .resizable(false)
            .show(ctx, |ui| {
                ui.add_space(12.0);
                ui.label(RichText::new("Queue").size(22.0).strong());
                ui.label(RichText::new("Local playback queue, no streaming required.").color(TEXT_MUTED));
                ui.add_space(12.0);

                let queue_tracks: Vec<Track> = self.queue_tracks().into_iter().cloned().collect();
                egui::ScrollArea::vertical().show(ui, |ui| {
                    for (index, track) in queue_tracks.iter().enumerate() {
                        let active = self.queue_position == Some(index);
                        let button = egui::Button::new(
                            RichText::new(format!("{}\n{}", track.title, track.artist)).color(Color32::WHITE),
                        )
                        .fill(if active { ACCENT_GREEN_SOFT } else { SURFACE });

                        if ui.add_sized([ui.available_width(), 52.0], button).clicked() {
                            self.queue_position = Some(index);
                            self.play_track(track.id);
                        }
                        ui.add_space(4.0);
                    }

                    if self.queue.is_empty() {
                        ui.add_space(16.0);
                        ui.label(RichText::new("Queue is empty.").strong());
                        ui.label(RichText::new("Play a track from the visible results to build the queue.").color(TEXT_MUTED));
                    }
                });

                ui.add_space(12.0);
                ui.separator();
                ui.add_space(10.0);
                ui.label(RichText::new("Selection").strong());
                if let Some(track) = self.selected_track() {
                    ui.label(RichText::new(&track.title).size(18.0).strong());
                    ui.label(RichText::new(format!("{} - {}", track.artist, track.album)).color(TEXT_MUTED));
                    ui.small(&track.file_path);
                } else {
                    ui.label(RichText::new("Nothing selected.").color(TEXT_MUTED));
                }
            });
    }

    fn draw_top_bar(&mut self, ui: &mut egui::Ui) {
        ui.horizontal(|ui| {
            ui.vertical(|ui| {
                ui.label(RichText::new(self.active_nav.label()).size(30.0).strong());
                ui.label(RichText::new(self.active_nav.subtitle()).color(TEXT_MUTED));
            });

            ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
                ui.add_sized(
                    [340.0, 36.0],
                    TextEdit::singleline(&mut self.search_query)
                        .hint_text("What do you want to play?")
                        .desired_width(340.0),
                );
            });
        });
    }

    fn draw_stat_card(ui: &mut egui::Ui, title: &str, value: String, subtitle: &str) {
        egui::Frame::group(ui.style()).fill(SURFACE).show(ui, |ui| {
            ui.set_min_size(egui::vec2(210.0, 118.0));
            ui.label(RichText::new(title).color(TEXT_MUTED).strong());
            ui.add_space(8.0);
            ui.label(RichText::new(value).size(32.0).strong());
            ui.add_space(8.0);
            ui.label(RichText::new(subtitle).color(TEXT_MUTED));
        });
    }

    fn draw_track_row(&mut self, ui: &mut egui::Ui, track: &Track, highlighted: bool) {
        let duration_label = if track.duration_seconds > 0 {
            format_duration(track.duration_seconds)
        } else {
            "--:--".to_string()
        };

        let response = egui::Frame::group(ui.style())
            .fill(if highlighted { ACCENT_GREEN_SOFT } else { SURFACE })
            .show(ui, |ui| {
                ui.set_min_height(56.0);
                ui.horizontal(|ui| {
                    ui.vertical(|ui| {
                        ui.label(RichText::new(&track.title).color(Color32::WHITE).strong());
                        ui.label(
                            RichText::new(format!("{} - {}", track.artist, track.album)).color(TEXT_MUTED),
                        );
                    });
                    ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
                        ui.label(RichText::new(duration_label).color(TEXT_MUTED));
                    });
                });
            })
            .response;
        if response.clicked() {
            self.selected_track_id = Some(track.id);
        }
        if response.double_clicked() {
            self.selected_track_id = Some(track.id);
            self.play_selected_from_visible();
        }
        ui.add_space(6.0);
    }

    fn draw_home(&mut self, ui: &mut egui::Ui) {
        let stats = self.library_stats();

        egui::Frame::group(ui.style()).fill(PANEL_SOFT).show(ui, |ui| {
            ui.add_space(4.0);
            ui.label(RichText::new("Built for owned music, not streaming catalogs.").color(TEXT_MUTED));
            ui.label(
                RichText::new("Make the local library feel premium again.")
                    .size(34.0)
                    .strong(),
            );
            ui.add_space(8.0);
            ui.horizontal(|ui| {
                if ui
                    .add_enabled(
                        self.selected_track_id.is_some(),
                        egui::Button::new("Play Selection").fill(ACCENT_GREEN),
                    )
                    .clicked()
                {
                    self.play_selected_from_visible();
                }
                if ui.button("Go To Library").clicked() {
                    self.active_nav = NavSection::Library;
                }
            });
        });

        ui.add_space(16.0);
        ui.horizontal(|ui| {
            Self::draw_stat_card(ui, "Tracks", stats.track_count.to_string(), "Indexed from local source folders");
            Self::draw_stat_card(ui, "Artists", stats.artist_count.to_string(), "Normalized from imported folders");
            Self::draw_stat_card(ui, "Albums", stats.album_count.to_string(), "Ready for album-centric browsing");
        });

        ui.add_space(18.0);
        ui.columns(2, |columns| {
            columns[0].label(RichText::new("Recently Added").size(22.0).strong());
            columns[0].add_space(8.0);
            egui::ScrollArea::vertical().max_height(360.0).show(&mut columns[0], |ui| {
                for track in stats.recent_tracks {
                    let highlighted = self.selected_track_id == Some(track.id);
                    self.draw_track_row(ui, &track, highlighted);
                }
                if self.tracks.is_empty() {
                    ui.label(RichText::new("Import a folder to populate Home.").color(TEXT_MUTED));
                }
            });

            columns[1].label(RichText::new("Top Artists In Your Library").size(22.0).strong());
            columns[1].add_space(8.0);
            egui::ScrollArea::vertical().max_height(360.0).show(&mut columns[1], |ui| {
                for (artist, count) in stats.top_artists {
                    egui::Frame::group(ui.style()).fill(SURFACE).show(ui, |ui| {
                        ui.horizontal(|ui| {
                            ui.label(RichText::new(artist).size(18.0).strong());
                            ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
                                ui.label(RichText::new(format!("{} tracks", count)).color(TEXT_MUTED));
                            });
                        });
                    });
                    ui.add_space(6.0);
                }
                if self.tracks.is_empty() {
                    ui.label(RichText::new("Artist summaries appear after import.").color(TEXT_MUTED));
                }
            });
        });
    }

    fn draw_search(&mut self, ui: &mut egui::Ui) {
        let visible_tracks: Vec<Track> = self.visible_tracks().into_iter().cloned().collect();
        ui.label(RichText::new(format!("{} results", visible_tracks.len())).color(TEXT_MUTED));
        ui.add_space(8.0);

        if self.search_query.trim().is_empty() {
            egui::Frame::group(ui.style()).fill(PANEL_SOFT).show(ui, |ui| {
                ui.set_min_height(220.0);
                ui.vertical_centered(|ui| {
                    ui.add_space(48.0);
                    ui.label(RichText::new("Search your local collection").size(28.0).strong());
                    ui.label(RichText::new("Try a title, artist, or album in the search field above.").color(TEXT_MUTED));
                });
            });
            return;
        }

        egui::ScrollArea::vertical().show(ui, |ui| {
            for track in visible_tracks {
                let highlighted = self.selected_track_id == Some(track.id);
                self.draw_track_row(ui, &track, highlighted);
            }
            if self.visible_tracks().is_empty() {
                ui.label(RichText::new("No results matched that search.").color(TEXT_MUTED));
            }
        });
    }

    fn draw_library(&mut self, ui: &mut egui::Ui) {
        let visible_tracks: Vec<Track> = self.visible_tracks().into_iter().cloned().collect();
        ui.horizontal(|ui| {
            ui.label(RichText::new("Tracks").size(22.0).strong());
            ui.label(RichText::new(format!("{} visible", visible_tracks.len())).color(TEXT_MUTED));
        });
        ui.add_space(10.0);

        egui::ScrollArea::vertical().show(ui, |ui| {
            for track in visible_tracks {
                let highlighted = self.selected_track_id == Some(track.id)
                    || self.now_playing_track_id == Some(track.id);
                self.draw_track_row(ui, &track, highlighted);
            }

            if self.tracks.is_empty() {
                egui::Frame::group(ui.style()).fill(PANEL_SOFT).show(ui, |ui| {
                    ui.set_min_height(240.0);
                    ui.vertical_centered(|ui| {
                        ui.add_space(48.0);
                        ui.label(RichText::new("No music indexed yet").size(28.0).strong());
                        ui.label(RichText::new("Import a local music folder from the left sidebar to get started.").color(TEXT_MUTED));
                    });
                });
            }
        });
    }

    fn draw_bottom_bar(&mut self, ctx: &egui::Context) {
        egui::TopBottomPanel::bottom("playback_bar")
            .min_height(98.0)
            .show(ctx, |ui| {
                ui.add_space(8.0);
                ui.horizontal(|ui| {
                    ui.vertical(|ui| {
                        ui.label(RichText::new("Now Playing").color(TEXT_MUTED).strong());
                        if let Some(track) = self.current_track() {
                            ui.label(RichText::new(&track.title).size(24.0).strong());
                            ui.label(RichText::new(format!("{} - {}", track.artist, track.album)).color(TEXT_MUTED));
                        } else {
                            ui.label(RichText::new("Nothing playing yet").size(24.0).strong());
                            ui.label(RichText::new("Select a track and hit play.").color(TEXT_MUTED));
                        }
                    });

                    ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
                        if ui.button("Quit").clicked() {
                            ctx.send_viewport_cmd(egui::ViewportCommand::Close);
                        }

                        if ui.button("Next").clicked() {
                            self.play_queue_offset(1);
                        }

                        let transport_label = if self.audio_player.as_ref().is_some_and(AudioPlayer::is_paused) {
                            "Resume"
                        } else {
                            "Play / Pause"
                        };
                        if ui
                            .add_enabled(
                                self.audio_player.is_some(),
                                egui::Button::new(transport_label).fill(ACCENT_GREEN),
                            )
                            .clicked()
                        {
                            self.toggle_playback();
                        }

                        if ui.button("Previous").clicked() {
                            self.play_queue_offset(-1);
                        }
                    });
                });
                ui.add_space(8.0);
            });
    }
}

impl eframe::App for PlaymuApp {
    fn update(&mut self, ctx: &egui::Context, _frame: &mut eframe::Frame) {
        self.process_background_events();

        self.draw_sidebar(ctx);
        self.draw_right_panel(ctx);

        egui::CentralPanel::default().show(ctx, |ui| {
            ui.add_space(12.0);
            self.draw_top_bar(ui);
            ui.add_space(16.0);

            match self.active_nav {
                NavSection::Home => self.draw_home(ui),
                NavSection::Search => self.draw_search(ui),
                NavSection::Library => self.draw_library(ui),
            }
        });

        self.draw_bottom_bar(ctx);
    }
}

struct AudioPlayer {
    _stream: OutputStream,
    handle: OutputStreamHandle,
    current_sink: Option<Sink>,
}

impl AudioPlayer {
    fn new() -> anyhow::Result<Self> {
        let (stream, handle) = OutputStream::try_default()?;
        Ok(Self {
            _stream: stream,
            handle,
            current_sink: None,
        })
    }

    fn play_file(&mut self, file_path: &str) -> anyhow::Result<()> {
        if let Some(sink) = self.current_sink.take() {
            sink.stop();
        }

        let file = File::open(file_path)?;
        let source = Decoder::new(BufReader::new(file))?;
        let sink = Sink::try_new(&self.handle)?;
        sink.append(source);
        sink.play();
        self.current_sink = Some(sink);

        Ok(())
    }

    fn toggle_pause(&mut self) -> Option<bool> {
        let sink = self.current_sink.as_ref()?;
        if sink.is_paused() {
            sink.play();
            Some(false)
        } else {
            sink.pause();
            Some(true)
        }
    }

    fn is_paused(&self) -> bool {
        match &self.current_sink {
            Some(sink) => sink.is_paused(),
            None => false,
        }
    }
}

fn configure_theme(ctx: &egui::Context) {
    let mut visuals = egui::Visuals::dark();
    visuals.override_text_color = Some(Color32::from_rgb(242, 245, 246));
    visuals.panel_fill = PANEL_DARK;
    visuals.faint_bg_color = PANEL_SOFT;
    visuals.extreme_bg_color = PANEL_SOFT;
    visuals.window_fill = PANEL_DARK;
    visuals.widgets.noninteractive.bg_fill = PANEL_SOFT;
    visuals.widgets.inactive.bg_fill = SURFACE;
    visuals.widgets.hovered.bg_fill = SURFACE_HOVER;
    visuals.widgets.active.bg_fill = ACCENT_GREEN_SOFT;
    visuals.selection.bg_fill = ACCENT_GREEN;
    ctx.set_visuals(visuals);

    let mut style = (*ctx.style()).clone();
    style.spacing.item_spacing = egui::vec2(10.0, 10.0);
    style.spacing.button_padding = egui::vec2(14.0, 12.0);
    style.spacing.window_margin = egui::Margin::same(14);
    ctx.set_style(style);
}

fn format_duration(duration_seconds: i64) -> String {
    let minutes = duration_seconds / 60;
    let seconds = duration_seconds % 60;
    format!("{minutes}:{seconds:02}")
}