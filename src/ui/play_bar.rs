use std::time::Duration;

use eframe::egui::{self, Color32, Margin, RichText, Stroke, Vec2};

use crate::{
    app::PlaymuApp,
    icons::{icon_widget, paint_equalizer_animated, paint_icon, Icon},
    models::RepeatMode,
    theme::{ACCENT_GREEN, ACCENT_GREEN_SOFT, CARD_STROKE, PANEL_DARK, TEXT_BRIGHT, TEXT_FAINT, TEXT_MUTED},
    ui::{
        paint_art,
        widgets::{draw_progress_bar, transport_button},
    },
    util::format_duration,
};

impl PlaymuApp {
    pub fn draw_bottom_bar(&mut self, ctx: &egui::Context) {
        let current = self.current_track().cloned();
        let art_tex = current.as_ref().and_then(|t| {
            self.get_art_texture(ctx, &t.artist, &t.album)
        });
        let (position, total, volume, has_track, is_paused, has_audio) =
            match self.audio_player.as_ref() {
                Some(p) => (p.position(), p.total(), p.volume(), p.has_track(), p.is_paused(), true),
                None => (Duration::ZERO, Duration::ZERO, 1.0, false, false, false),
            };

        let queue_pos = self.queue_position;
        let queue_len = self.queue.len();
        let anim_time = self.anim_time;
        let is_playing = has_track && !is_paused;
        let shuffle = self.shuffle;
        let repeat = self.repeat;

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
                    // ---- Left: animated art + now-playing metadata ----
                    ui.allocate_ui_with_layout(
                        Vec2::new(full_width * 0.28, 64.0),
                        egui::Layout::left_to_right(egui::Align::Center),
                        |ui| {
                            if let Some(track) = &current {
                                // Animated equalizer badge overlaid on art when playing.
                                let art_rect = {
                                    let (rect, _) = ui.allocate_exact_size(
                                        Vec2::splat(56.0),
                                        egui::Sense::hover(),
                                    );
                                    rect
                                };
                                // Draw art (real or gradient).
                                {
                                    let painter = ui.painter();
                                    let corner = eframe::egui::CornerRadius::same(8);
                                    if let Some(ref tex) = art_tex {
                                        let uv = eframe::egui::Rect::from_min_max(
                                            eframe::egui::pos2(0.0, 0.0),
                                            eframe::egui::pos2(1.0, 1.0),
                                        );
                                        painter.add(eframe::egui::Shape::image(
                                            tex.id(),
                                            art_rect,
                                            uv,
                                            Color32::WHITE,
                                        ));
                                    } else {
                                        use crate::ui::{art_color, lighten, paint_vertical_gradient};
                                        let base = art_color(&track.album);
                                        painter.rect_filled(art_rect, corner, base);
                                        let clip = painter.with_clip_rect(art_rect);
                                        paint_vertical_gradient(
                                            &clip,
                                            art_rect,
                                            lighten(base, 0.22),
                                            base,
                                        );
                                    }
                                    // Animated bars overlay when playing.
                                    if is_playing {
                                        let badge = eframe::egui::Rect::from_center_size(
                                            art_rect.center(),
                                            Vec2::splat(28.0),
                                        );
                                        painter.rect_filled(
                                            art_rect,
                                            corner,
                                            Color32::from_black_alpha(100),
                                        );
                                        paint_equalizer_animated(
                                            painter,
                                            badge,
                                            ACCENT_GREEN,
                                            anim_time,
                                        );
                                    }
                                    painter.rect_stroke(
                                        art_rect,
                                        corner,
                                        Stroke::new(1.0, Color32::from_white_alpha(20)),
                                        eframe::egui::StrokeKind::Inside,
                                    );
                                }

                                ui.add_space(12.0);
                                ui.vertical(|ui| {
                                    ui.add_space(6.0);
                                    ui.add(
                                        egui::Label::new(
                                            RichText::new(&track.title)
                                                .size(15.0)
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
                                paint_art(ui, "", None, 56.0, 8);
                                ui.add_space(12.0);
                                ui.vertical(|ui| {
                                    ui.add_space(8.0);
                                    ui.label(
                                        RichText::new("Nothing playing")
                                            .size(15.0)
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

                    // ---- Center: shuffle | prev | play | next | repeat + progress ----
                    ui.allocate_ui_with_layout(
                        Vec2::new(full_width * 0.44, 72.0),
                        egui::Layout::top_down(egui::Align::Center),
                        |ui| {
                            ui.horizontal(|ui| {
                                let center_w = ui.available_width();
                                ui.add_space((center_w - 230.0).max(0.0) / 2.0);

                                // Shuffle button
                                let sh_color = if shuffle { ACCENT_GREEN } else { TEXT_MUTED };
                                let (sh_rect, sh_resp) = ui
                                    .allocate_exact_size(Vec2::splat(30.0), egui::Sense::click());
                                paint_icon(ui.painter(), sh_rect, Icon::Shuffle, sh_color);
                                if sh_resp.clicked() {
                                    self.toggle_shuffle();
                                }
                                ui.add_space(8.0);

                                if transport_button(ui, Icon::Prev, false, has_audio).clicked() {
                                    self.play_queue_offset(-1);
                                }
                                ui.add_space(8.0);
                                let play_icon = if is_paused || !has_track {
                                    Icon::Play
                                } else {
                                    Icon::Pause
                                };
                                if transport_button(ui, play_icon, true, has_audio).clicked() {
                                    self.toggle_playback();
                                }
                                ui.add_space(8.0);
                                if transport_button(ui, Icon::Next, false, has_audio).clicked() {
                                    self.play_queue_offset(1);
                                }
                                ui.add_space(8.0);

                                // Repeat button cycles through Off → Queue → Track
                                let rep_icon = if repeat == RepeatMode::Track {
                                    Icon::RepeatOne
                                } else {
                                    Icon::Repeat
                                };
                                let rep_color = if repeat != RepeatMode::Off {
                                    ACCENT_GREEN
                                } else {
                                    TEXT_MUTED
                                };
                                let (rep_rect, rep_resp) = ui
                                    .allocate_exact_size(Vec2::splat(30.0), egui::Sense::click());
                                paint_icon(ui.painter(), rep_rect, rep_icon, rep_color);
                                if rep_resp.clicked() {
                                    self.cycle_repeat();
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
                                        // "3 / 47" queue position
                                        if let (Some(pos), true) = (queue_pos, queue_len > 0) {
                                            ui.label(
                                                RichText::new(format!("{} / {}", pos + 1, queue_len))
                                                    .color(TEXT_FAINT)
                                                    .small(),
                                            );
                                            ui.add_space(8.0);
                                        }
                                        let total_label = if total > Duration::ZERO {
                                            format_duration(total.as_secs() as i64)
                                        } else {
                                            "--:--".to_string()
                                        };
                                        ui.label(
                                            RichText::new(total_label).color(TEXT_FAINT).small(),
                                        );
                                        ui.add_space(8.0);
                                        draw_progress_bar(ui, fraction);
                                    },
                                );
                            });
                        },
                    );

                    // ---- Right: volume + queue toggle + quit ----
                    ui.allocate_ui_with_layout(
                        Vec2::new(ui.available_width(), 64.0),
                        egui::Layout::right_to_left(egui::Align::Center),
                        |ui| {
                            if ui
                                .add(
                                    egui::Button::new(RichText::new("Quit").color(TEXT_MUTED))
                                        .fill(Color32::TRANSPARENT),
                                )
                                .clicked()
                            {
                                ctx.send_viewport_cmd(egui::ViewportCommand::Close);
                            }
                            ui.add_space(10.0);

                            // Queue panel toggle
                            let q_icon = if self.queue_panel_open {
                                Icon::ChevronRight
                            } else {
                                Icon::ChevronLeft
                            };
                            let q_color = if self.queue_panel_open { ACCENT_GREEN } else { TEXT_MUTED };
                            let (q_rect, q_resp) =
                                ui.allocate_exact_size(Vec2::splat(22.0), egui::Sense::click());
                            paint_icon(ui.painter(), q_rect, q_icon, q_color);
                            if q_resp.clicked() {
                                self.queue_panel_open = !self.queue_panel_open;
                            }
                            ui.add_space(10.0);

                            // Volume
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
                            ui.add_space(4.0);
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
