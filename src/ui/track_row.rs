use eframe::egui::{self, CornerRadius, Margin, RichText, Sense};

use crate::{
    app::PlaymuApp,
    db::Track,
    icons::{icon_widget, Icon},
    models::NavSection,
    theme::{ACCENT_GREEN, ACCENT_GREEN_SOFT, SURFACE, TEXT_BRIGHT, TEXT_MUTED},
    ui::{paint_art_play, widgets::{add_to_playlist_button, hover_highlight_row}},
    util::{format_duration, highlight_match_job},
};

impl PlaymuApp {
    pub fn draw_track_row(&mut self, ui: &mut egui::Ui, track: &Track, highlighted: bool) {
        let duration_label = if track.duration_seconds > 0 {
            format_duration(track.duration_seconds)
        } else {
            "--:--".to_string()
        };
        let search_q = if self.active_nav == NavSection::Search {
            self.search_query.trim()
        } else {
            ""
        };
        let is_now_playing = self.now_playing_track_id == Some(track.id);
        let fill = if highlighted { ACCENT_GREEN_SOFT } else { SURFACE };
        let title_color = if is_now_playing { ACCENT_GREEN } else { TEXT_BRIGHT };
        let playlists_snap = self.playlists.clone();

        let mut play_art_clicked = false;
        let mut playlist_chosen: Option<i64> = None;

        let response = egui::Frame::new()
            .fill(fill)
            .corner_radius(CornerRadius::same(10))
            .inner_margin(Margin::symmetric(12, 8))
            .show(ui, |ui| {
                ui.set_min_height(52.0);
                ui.horizontal(|ui| {
                    let (_, clicked) = paint_art_play(ui, &track.album, 42.0, 8);
                    play_art_clicked = clicked;

                    ui.add_space(10.0);
                    ui.vertical(|ui| {
                        ui.label(highlight_match_job(&track.title, search_q, title_color, true));
                        ui.label(highlight_match_job(
                            &format!("{} - {}", track.artist, track.album),
                            search_q,
                            TEXT_MUTED,
                            false,
                        ));
                    });

                    ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
                        ui.label(RichText::new(&duration_label).color(TEXT_MUTED).small());
                        if is_now_playing {
                            ui.add_space(8.0);
                            icon_widget(ui, 14.0, Icon::Equalizer, ACCENT_GREEN);
                        }
                        if let Some(id) =
                            add_to_playlist_button(ui, track.id, &playlists_snap, false)
                        {
                            playlist_chosen = Some(id);
                        }
                    });
                });
            })
            .response
            .interact(Sense::click());

        if response.hovered() {
            hover_highlight_row(ui, response.rect, 10);
        }

        if play_art_clicked {
            self.selected_track_id = Some(track.id);
            self.play_selected_from_visible();
        } else if response.double_clicked() {
            self.selected_track_id = Some(track.id);
            self.play_selected_from_visible();
        } else if response.clicked() {
            self.selected_track_id = Some(track.id);
        }

        if let Some(pl_id) = playlist_chosen {
            self.add_track_to_playlist(pl_id, track.id);
        }

        ui.add_space(6.0);
    }
}
