use std::time::Duration;

use eframe::egui::{self, Margin, RichText, Stroke, Vec2};

use crate::{
    app::PlaymuApp,
    icons::{icon_widget, Icon},
    theme::{CARD_STROKE, PANEL_DARK, TEXT_BRIGHT, TEXT_FAINT, TEXT_MUTED},
    ui::{
        paint_art,
        widgets::{draw_progress_bar, transport_button},
    },
    util::format_duration,
};

impl PlaymuApp {
    pub fn draw_bottom_bar(&mut self, ctx: &egui::Context) {
        let current = self.current_track().cloned();
        let (position, total, volume, has_track, is_paused, has_audio) =
            match self.audio_player.as_ref() {
                Some(p) => (
                    p.position(),
                    p.total(),
                    p.volume(),
                    p.has_track(),
                    p.is_paused(),
                    true,
                ),
                None => (Duration::ZERO, Duration::ZERO, 1.0, false, false, false),
            };

        egui::TopBottomPanel::bottom("playback_bar")
            .min_height(92.0)
            .frame(
                egui::Frame::new()
                    .fill(PANEL_DARK)
                    .inner_margin(Margin::symmetric(18, 12))
                    .stroke(Stroke::new(1.0, CARD_STROKE)),
            )
            .show(ctx, |ui| {
                let full_width = ui.available_width();
                ui.horizontal(|ui| {
                    // Left — art + now-playing metadata
                    ui.allocate_ui_with_layout(
                        Vec2::new(full_width * 0.30, 64.0),
                        egui::Layout::left_to_right(egui::Align::Center),
                        |ui| {
                            if let Some(track) = &current {
                                paint_art(ui, &track.album, 56.0, 8);
                                ui.add_space(12.0);
                                ui.vertical(|ui| {
                                    ui.add_space(6.0);
                                    ui.add(
                                        egui::Label::new(
                                            RichText::new(&track.title)
                                                .size(16.0)
                                                .strong()
                                                .color(TEXT_BRIGHT),
                                        )
                                        .truncate(),
                                    );
                                    ui.add(
                                        egui::Label::new(
                                            RichText::new(format!(
                                                "{} — {}",
                                                track.artist, track.album
                                            ))
                                            .color(TEXT_MUTED)
                                            .small(),
                                        )
                                        .truncate(),
                                    );
                                });
                            } else {
                                paint_art(ui, "", 56.0, 8);
                                ui.add_space(12.0);
                                ui.vertical(|ui| {
                                    ui.add_space(8.0);
                                    ui.label(
                                        RichText::new("Nothing playing")
                                            .size(16.0)
                                            .strong()
                                            .color(TEXT_BRIGHT),
                                    );
                                    ui.label(
                                        RichText::new("Select a track and hit play.")
                                            .color(TEXT_MUTED)
                                            .small(),
                                    );
                                });
                            }
                        },
                    );

                    // Center — transport + progress bar
                    ui.allocate_ui_with_layout(
                        Vec2::new(full_width * 0.42, 72.0),
                        egui::Layout::top_down(egui::Align::Center),
                        |ui| {
                            ui.horizontal(|ui| {
                                ui.add_space((ui.available_width() - 150.0).max(0.0) / 2.0);
                                if transport_button(ui, Icon::Prev, false, has_audio).clicked() {
                                    self.play_queue_offset(-1);
                                }
                                ui.add_space(8.0);
                                let play_icon =
                                    if is_paused || !has_track { Icon::Play } else { Icon::Pause };
                                if transport_button(ui, play_icon, true, has_audio).clicked() {
                                    self.toggle_playback();
                                }
                                ui.add_space(8.0);
                                if transport_button(ui, Icon::Next, false, has_audio).clicked() {
                                    self.play_queue_offset(1);
                                }
                            });

                            ui.add_space(6.0);
                            let fraction = if total > Duration::ZERO {
                                position.as_secs_f32() / total.as_secs_f32()
                            } else {
                                0.0
                            };
                            ui.horizontal(|ui| {
                                ui.label(
                                    RichText::new(format_duration(position.as_secs() as i64))
                                        .color(TEXT_FAINT)
                                        .small(),
                                );
                                ui.with_layout(
                                    egui::Layout::right_to_left(egui::Align::Center),
                                    |ui| {
                                        let total_label = if total > Duration::ZERO {
                                            format_duration(total.as_secs() as i64)
                                        } else {
                                            "--:--".to_string()
                                        };
                                        ui.label(
                                            RichText::new(total_label)
                                                .color(TEXT_FAINT)
                                                .small(),
                                        );
                                        ui.add_space(8.0);
                                        draw_progress_bar(ui, fraction);
                                    },
                                );
                            });
                        },
                    );

                    // Right — volume icon + slider + quit
                    ui.allocate_ui_with_layout(
                        Vec2::new(ui.available_width(), 64.0),
                        egui::Layout::right_to_left(egui::Align::Center),
                        |ui| {
                            if ui
                                .add(
                                    egui::Button::new(
                                        RichText::new("Quit").color(TEXT_MUTED),
                                    )
                                    .fill(egui::Color32::TRANSPARENT),
                                )
                                .clicked()
                            {
                                ctx.send_viewport_cmd(egui::ViewportCommand::Close);
                            }
                            ui.add_space(12.0);

                            let mut new_volume = volume;
                            let slider = ui.add_enabled(
                                has_audio,
                                egui::Slider::new(&mut new_volume, 0.0..=1.0)
                                    .show_value(false)
                                    .handle_shape(egui::style::HandleShape::Circle),
                            );
                            if slider.changed() {
                                if let Some(player) = self.audio_player.as_mut() {
                                    player.set_volume(new_volume);
                                }
                            }
                            ui.add_space(6.0);
                            let vol_icon = if volume <= 0.001 {
                                Icon::Mute
                            } else if volume < 0.5 {
                                Icon::VolumeLow
                            } else {
                                Icon::Volume
                            };
                            icon_widget(ui, 20.0, vol_icon, TEXT_MUTED);
                        },
                    );
                });
            });
    }
}
