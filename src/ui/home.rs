use eframe::egui::{self, RichText};

use crate::{
    app::PlaymuApp,
    models::NavSection,
    theme::{ACCENT_GREEN, PANEL_SOFT, SURFACE, TEXT_MUTED},
    ui::widgets::{draw_collection_card, draw_stat_card},
};

impl PlaymuApp {
    pub fn draw_home(&mut self, ui: &mut egui::Ui) {
        let stats = self.library_stats();
        let pinned_summaries = self.pinned_album_summaries();
        let generated_mixes = self.generated_mixes();
        let recently_played = self.recently_played.clone();

        egui::Frame::group(ui.style()).fill(PANEL_SOFT).show(ui, |ui| {
            ui.add_space(4.0);
            ui.label(
                RichText::new("Built for owned music, not streaming catalogs.").color(TEXT_MUTED),
            );
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
                let pin_label = if let Some((artist, album)) = self.selected_album_key() {
                    if self.is_album_pinned(&artist, &album) {
                        "Unpin Selected Album"
                    } else {
                        "Pin Selected Album"
                    }
                } else {
                    "Pin Selected Album"
                };
                if ui
                    .add_enabled(self.selected_album_key().is_some(), egui::Button::new(pin_label))
                    .clicked()
                {
                    self.toggle_selected_album_pin();
                }
            });
        });

        ui.add_space(16.0);
        ui.horizontal(|ui| {
            draw_stat_card(
                ui,
                "Tracks",
                stats.track_count.to_string(),
                "Indexed from local source folders",
            );
            draw_stat_card(
                ui,
                "Artists",
                stats.artist_count.to_string(),
                "Normalized from imported folders",
            );
            draw_stat_card(
                ui,
                "Albums",
                stats.album_count.to_string(),
                "Ready for album-centric browsing",
            );
        });

        ui.add_space(18.0);
        ui.label(RichText::new("Jump Back In").size(22.0).strong());
        ui.add_space(8.0);
        egui::ScrollArea::horizontal().show(ui, |ui| {
            ui.horizontal(|ui| {
                for album in stats.recent_albums.clone() {
                    let response = draw_collection_card(
                        ui,
                        &album.title,
                        &album.artist,
                        &format!("{} tracks", album.track_count),
                        PANEL_SOFT,
                    );
                    if response.clicked() {
                        self.open_album(&album.artist, &album.title);
                    }
                    if response.double_clicked() {
                        self.play_track_list(album.track_ids.clone(), 0);
                    }
                    ui.add_space(8.0);
                }
            });
        });

        ui.add_space(18.0);
        ui.label(RichText::new("Pinned Albums").size(22.0).strong());
        ui.add_space(8.0);
        egui::ScrollArea::horizontal().show(ui, |ui| {
            ui.horizontal(|ui| {
                for album in pinned_summaries.clone() {
                    let response = draw_collection_card(
                        ui,
                        &album.title,
                        &album.artist,
                        &format!("{} tracks", album.track_count),
                        SURFACE,
                    );
                    if response.clicked() {
                        self.open_album(&album.artist, &album.title);
                    }
                    if response.double_clicked() {
                        self.play_track_list(album.track_ids.clone(), 0);
                    }
                    ui.add_space(8.0);
                }
                if pinned_summaries.is_empty() {
                    ui.label(
                        RichText::new("No pinned albums yet. Select a track and pin its album.")
                            .color(TEXT_MUTED),
                    );
                }
            });
        });

        ui.add_space(18.0);
        ui.label(RichText::new("Made For You Locally").size(22.0).strong());
        ui.add_space(8.0);
        ui.horizontal_wrapped(|ui| {
            for mix in generated_mixes.clone() {
                egui::Frame::group(ui.style()).fill(PANEL_SOFT).show(ui, |ui| {
                    ui.set_min_size(egui::vec2(280.0, 128.0));
                    ui.label(RichText::new(&mix.name).size(20.0).strong());
                    ui.label(RichText::new(&mix.description).color(TEXT_MUTED));
                    ui.add_space(8.0);
                    ui.label(
                        RichText::new(format!("{} tracks", mix.track_ids.len())).color(TEXT_MUTED),
                    );
                    if ui.button("Play Mix").clicked() {
                        self.play_track_list(mix.track_ids.clone(), 0);
                    }
                });
                ui.add_space(10.0);
            }
            if generated_mixes.is_empty() {
                ui.label(
                    RichText::new("Play a few songs to generate local-history mixes.")
                        .color(TEXT_MUTED),
                );
            }
        });

        ui.add_space(18.0);
        ui.columns(2, |columns| {
            columns[0].label(RichText::new("Recently Played").size(22.0).strong());
            columns[0].add_space(8.0);
            egui::ScrollArea::vertical()
                .max_height(360.0)
                .show(&mut columns[0], |ui| {
                    for track in recently_played {
                        let highlighted = self.selected_track_id == Some(track.id);
                        self.draw_track_row(ui, &track, highlighted);
                    }
                    if self.recently_played.is_empty() {
                        ui.label(
                            RichText::new(
                                "Nothing played yet. Start playback to build history.",
                            )
                            .color(TEXT_MUTED),
                        );
                    }
                });

            columns[1].label(RichText::new("Top Artists In Your Library").size(22.0).strong());
            columns[1].add_space(8.0);
            egui::ScrollArea::vertical()
                .max_height(360.0)
                .show(&mut columns[1], |ui| {
                    for (artist, count) in stats.top_artists {
                        let response = draw_collection_card(
                            ui,
                            &artist,
                            "Open artist view in your library",
                            &format!("{} tracks", count),
                            SURFACE,
                        );
                        if response.clicked() {
                            self.open_artist(&artist);
                        }
                        ui.add_space(6.0);
                    }
                    if self.tracks.is_empty() {
                        ui.label(
                            RichText::new("Artist summaries appear after import.")
                                .color(TEXT_MUTED),
                        );
                    }
                });
        });
    }
}
