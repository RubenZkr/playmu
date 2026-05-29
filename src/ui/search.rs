use eframe::egui::{self, RichText};

use crate::{
    app::PlaymuApp,
    models::SearchResults,
    theme::{ACCENT_GREEN_SOFT, PANEL_SOFT, SURFACE, TEXT_MUTED},
    ui::widgets::draw_collection_card,
};

impl PlaymuApp {
    pub fn draw_search(&mut self, ui: &mut egui::Ui) {
        let results = self.search_results();
        self.ensure_search_selection(&results);

        ui.label(
            RichText::new(format!(
                "{} songs, {} albums, {} artists",
                results.tracks.len(),
                results.albums.len(),
                results.artists.len()
            ))
            .color(TEXT_MUTED),
        );
        ui.label(
            RichText::new(
                "Use Up/Down to navigate songs, Enter to play. \
                 If the search box is focused, hold Alt + Up/Down.",
            )
            .color(TEXT_MUTED),
        );
        ui.add_space(8.0);

        if self.search_query.trim().is_empty() {
            egui::Frame::group(ui.style()).fill(PANEL_SOFT).show(ui, |ui| {
                ui.set_min_height(220.0);
                ui.vertical_centered(|ui| {
                    ui.add_space(48.0);
                    ui.label(
                        RichText::new("Search your local collection")
                            .size(28.0)
                            .strong(),
                    );
                    ui.label(
                        RichText::new(
                            "Try a title, artist, or album in the search field above.",
                        )
                        .color(TEXT_MUTED),
                    );
                });
            });
            return;
        }

        if let Some(top_track) = results.tracks.first().cloned() {
            ui.label(RichText::new("Top Result").size(22.0).strong());
            ui.add_space(8.0);
            let top_result = draw_collection_card(
                ui,
                &top_track.title,
                &format!("{} - {}", top_track.artist, top_track.album),
                "Double click a song row below to play a search queue",
                ACCENT_GREEN_SOFT,
            );
            if top_result.clicked() {
                self.selected_track_id = Some(top_track.id);
            }
            ui.add_space(18.0);
        }

        ui.columns(2, |columns| {
            columns[0].label(RichText::new("Albums").size(22.0).strong());
            columns[0].add_space(8.0);
            egui::ScrollArea::vertical()
                .max_height(220.0)
                .show(&mut columns[0], |ui| {
                    for album in results.albums.clone() {
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
                        ui.add_space(6.0);
                    }
                });

            columns[1].label(RichText::new("Artists").size(22.0).strong());
            columns[1].add_space(8.0);
            egui::ScrollArea::vertical()
                .max_height(220.0)
                .show(&mut columns[1], |ui| {
                    for artist in results.artists.clone() {
                        let response = draw_collection_card(
                            ui,
                            &artist.name,
                            "Open in Library",
                            &format!(
                                "{} tracks across {} albums",
                                artist.track_count, artist.album_count
                            ),
                            SURFACE,
                        );
                        if response.clicked() {
                            self.open_artist(&artist.name);
                        }
                        ui.add_space(6.0);
                    }
                });
        });

        ui.add_space(18.0);
        ui.label(RichText::new("Songs").size(22.0).strong());
        ui.add_space(8.0);

        egui::ScrollArea::vertical().show(ui, |ui| {
            let tracks = results.tracks;
            let is_empty = tracks.is_empty();
            for track in tracks {
                let highlighted = self.selected_track_id == Some(track.id);
                self.draw_track_row(ui, &track, highlighted);
            }
            if is_empty {
                ui.label(
                    RichText::new("No results matched that search.").color(TEXT_MUTED),
                );
            }
        });
    }

    pub fn handle_search_shortcuts(&mut self, ctx: &egui::Context) {
        use crate::models::NavSection;
        if self.active_nav != NavSection::Search || self.search_query.trim().is_empty() {
            return;
        }

        let results = self.search_results();
        if results.tracks.is_empty() {
            return;
        }

        self.ensure_search_selection(&results);
        let track_ids: Vec<i64> = results.tracks.iter().map(|t| t.id).collect();
        let current_index = self
            .selected_track_id
            .and_then(|id| track_ids.iter().position(|tid| *tid == id))
            .unwrap_or(0);

        let (move_up, move_down, play_pressed, alt_held) = ctx.input(|input| {
            (
                input.key_pressed(egui::Key::ArrowUp),
                input.key_pressed(egui::Key::ArrowDown),
                input.key_pressed(egui::Key::Enter),
                input.modifiers.alt,
            )
        });

        let can_navigate = !self.search_input_has_focus || alt_held;
        if can_navigate {
            if move_up {
                self.selected_track_id = Some(track_ids[current_index.saturating_sub(1)]);
            }
            if move_down {
                self.selected_track_id =
                    Some(track_ids[(current_index + 1).min(track_ids.len() - 1)]);
            }
        }

        if play_pressed {
            let selected_id = self
                .selected_track_id
                .filter(|id| track_ids.iter().any(|tid| tid == id))
                .unwrap_or(track_ids[0]);
            let start_index = track_ids
                .iter()
                .position(|tid| *tid == selected_id)
                .unwrap_or(0);
            self.selected_track_id = Some(selected_id);
            self.play_track_list(track_ids, start_index);
        }
    }

    pub fn ensure_search_selection(&mut self, results: &SearchResults) {
        if results.tracks.is_empty() {
            return;
        }
        let selected_in_results = self
            .selected_track_id
            .is_some_and(|id| results.tracks.iter().any(|t| t.id == id));
        if !selected_in_results {
            self.selected_track_id = Some(results.tracks[0].id);
        }
    }
}
