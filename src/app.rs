use std::{
    fs::File,
    io::BufReader,
    path::PathBuf,
    sync::mpsc::{self, Receiver},
    thread,
};

use eframe::egui::{self, RichText};
use rodio::{Decoder, OutputStream, OutputStreamHandle, Sink};

use crate::{
    db::{self, Track},
    library::{self, ScanSummary},
};

pub struct PlaymuApp {
    db_path: PathBuf,
    source_input: String,
    tracks: Vec<Track>,
    selected_track_id: Option<i64>,
    status: String,
    scan_receiver: Option<Receiver<Result<ScanSummary, String>>>,
    is_scanning: bool,
    audio_player: Option<AudioPlayer>,
}

impl PlaymuApp {
    pub fn new(cc: &eframe::CreationContext<'_>) -> Self {
        cc.egui_ctx.set_visuals(egui::Visuals::dark());

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
            tracks,
            selected_track_id: None,
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
}

impl eframe::App for PlaymuApp {
    fn update(&mut self, ctx: &egui::Context, _frame: &mut eframe::Frame) {
        self.process_background_events();

        egui::TopBottomPanel::bottom("playback_bar")
            .min_height(86.0)
            .show(ctx, |ui| {
                ui.add_space(8.0);
                ui.horizontal(|ui| {
                    ui.vertical(|ui| {
                        ui.label(RichText::new("Now Playing").strong().size(16.0));
                        if let Some(track) = self.selected_track() {
                            ui.label(RichText::new(&track.title).size(20.0));
                            ui.label(format!("{} - {}", track.artist, track.album));
                        } else {
                            ui.label("No track selected");
                            ui.label("Import a folder, then choose a track.");
                        }
                    });

                    ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
                        if ui.button("Quit").clicked() {
                            ctx.send_viewport_cmd(egui::ViewportCommand::Close);
                        }

                        let stop_enabled = self.audio_player.is_some();
                        if ui
                            .add_enabled(stop_enabled, egui::Button::new("Stop"))
                            .clicked()
                        {
                            if let Some(player) = &mut self.audio_player {
                                player.stop();
                                self.status = "Playback stopped.".to_string();
                            }
                        }

                        let play_enabled = self.selected_track().is_some() && self.audio_player.is_some();
                        if ui
                            .add_enabled(play_enabled, egui::Button::new("Play Selected"))
                            .clicked()
                        {
                            if let (Some(track), Some(player)) =
                                (self.selected_track().cloned(), self.audio_player.as_mut())
                            {
                                match player.play_file(&track.file_path) {
                                    Ok(()) => {
                                        self.status = format!(
                                            "Playing {} - {}",
                                            track.artist, track.title
                                        );
                                    }
                                    Err(error) => {
                                        self.status = format!("Playback failed: {error}");
                                    }
                                }
                            }
                        }
                    });
                });
                ui.add_space(8.0);
            });

        egui::SidePanel::left("sidebar")
            .resizable(false)
            .min_width(260.0)
            .show(ctx, |ui| {
                ui.add_space(8.0);
                ui.heading("Playmu");
                ui.label("Local library player");
                ui.separator();

                ui.label(RichText::new("Library Import").strong());
                ui.label("Paste a music-folder path on this machine.");
                ui.text_edit_singleline(&mut self.source_input);

                if ui
                    .add_enabled(!self.is_scanning, egui::Button::new("Import Folder"))
                    .clicked()
                {
                    self.start_scan();
                }

                if self.is_scanning {
                    ui.add_space(8.0);
                    ui.spinner();
                }

                ui.separator();
                ui.label(RichText::new("Database").strong());
                ui.monospace(self.db_path.display().to_string());
                ui.separator();
                ui.label(RichText::new("Status").strong());
                ui.label(&self.status);
            });

        egui::CentralPanel::default().show(ctx, |ui| {
            ui.add_space(8.0);
            ui.horizontal(|ui| {
                ui.heading("Your Library");
                ui.label(format!("{} tracks indexed", self.tracks.len()));
            });
            ui.separator();

            egui::ScrollArea::vertical().show(ui, |ui| {
                for track in &self.tracks {
                    let is_selected = self.selected_track_id == Some(track.id);
                    let label = format!("{} - {}    [{}]", track.artist, track.title, track.album);
                    if ui.selectable_label(is_selected, label).clicked() {
                        self.selected_track_id = Some(track.id);
                    }
                    if track.duration_seconds > 0 {
                        ui.small(format!("{} seconds", track.duration_seconds));
                    }
                    ui.add_space(4.0);
                }

                if self.tracks.is_empty() {
                    ui.add_space(40.0);
                    ui.label(
                        RichText::new("No music indexed yet.")
                            .size(24.0)
                            .strong(),
                    );
                    ui.label(
                        "Import a local folder to create the first version of your library database.",
                    );
                }
            });
        });
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

    fn stop(&mut self) {
        if let Some(sink) = self.current_sink.take() {
            sink.stop();
        }
    }
}