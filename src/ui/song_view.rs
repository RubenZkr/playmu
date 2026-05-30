use std::time::Duration;

use eframe::egui::{
    self, Align2, Color32, CornerRadius, FontId, Margin, Pos2, Rect, RichText, ScrollArea, Sense,
    Shape, Stroke, StrokeKind, Vec2,
};

use crate::{
    app::PlaymuApp,
    icons::{paint_icon, Icon},
    models::{LyricsData, SongViewTab},
    theme::{
        ACCENT_GREEN, ACCENT_GREEN_SOFT, BG_BASE, PANEL_DARK, PANEL_SOFT, SURFACE, SURFACE_HOVER,
        TEXT_BRIGHT, TEXT_FAINT, TEXT_MUTED,
    },
    ui::{art_color, lighten, paint_art},
    util::format_duration,
};

impl PlaymuApp {
    /// Drive the appear/disappear animation. Returns current alpha (0–255).
    pub fn update_song_view_anim(&mut self, ctx: &egui::Context) -> u8 {
        let dt = ctx.input(|i| i.unstable_dt).clamp(0.0, 0.1);
        let target = if self.song_view_open { 1.0f32 } else { 0.0 };
        let speed = 9.0f32;
        self.song_view_anim += (target - self.song_view_anim) * speed * dt;
        self.song_view_anim = self.song_view_anim.clamp(0.0, 1.0);

        if (self.song_view_anim - target).abs() > 0.005 {
            ctx.request_repaint();
        }
        (self.song_view_anim * 255.0) as u8
    }

    pub fn draw_song_view_overlay(&mut self, ctx: &egui::Context) {
        let alpha = self.update_song_view_anim(ctx);
        if alpha == 0 {
            return;
        }

        // Render on top of everything (Foreground layer).
        egui::Area::new(egui::Id::new("song_view_overlay"))
            .order(egui::Order::Foreground)
            .fixed_pos(egui::pos2(0.0, 0.0))
            .show(ctx, |ui| {
                // Fill the entire screen.
                let screen = ctx.screen_rect();
                ui.set_clip_rect(screen);

                // --- Background ---
                let track = self.current_track().cloned();
                let bg_tint = track
                    .as_ref()
                    .map(|t| art_color(&t.album))
                    .unwrap_or(Color32::from_rgb(20, 20, 30));
                let bg_alpha = alpha;

                // Dark base.
                ui.painter()
                    .rect_filled(screen, CornerRadius::ZERO, Color32::from_black_alpha(bg_alpha));

                // Atmospheric color glow (top-center halo).
                let glow_center = egui::pos2(screen.center().x, screen.top() - 40.0);
                let glow_r = screen.width() * 0.55;
                for i in (0..6).rev() {
                    let r = glow_r * (1.0 - i as f32 * 0.12);
                    let a = ((15 - i * 2) as u32 * bg_alpha as u32 / 255) as u8;
                    ui.painter().circle_filled(
                        glow_center,
                        r,
                        Color32::from_rgba_unmultiplied(
                            bg_tint.r(),
                            bg_tint.g(),
                            bg_tint.b(),
                            a,
                        ),
                    );
                }

                // Allocate the full-screen frame.
                let resp = ui.allocate_rect(screen, Sense::click());
                let _ = resp; // absorb stray clicks so they don't fall through.

                // --- Layout in a vertical stack inside the screen ---
                let content_rect = screen.shrink2(Vec2::new(0.0, 0.0));

                // Use a child Ui for structured layout.
                ui.allocate_new_ui(egui::UiBuilder::new().max_rect(content_rect), |ui| {
                    ui.vertical(|ui| {
                        self.draw_song_view_header(ui, alpha);

                        // Tabs
                        self.draw_song_view_tabs(ui, alpha);

                        // Content area fills the rest.
                        let remaining = ui.available_rect_before_wrap();
                        ui.allocate_new_ui(
                            egui::UiBuilder::new().max_rect(remaining),
                            |ui| {
                                match self.song_view_tab {
                                    SongViewTab::Cover => self.draw_cover_tab(ui, alpha),
                                    SongViewTab::Lyrics => self.draw_lyrics_tab(ui, alpha),
                                    SongViewTab::Waves => self.draw_waves_tab(ui, alpha),
                                }
                            },
                        );
                    });
                });
            });
    }

    // -----------------------------------------------------------------------
    // Header: back button + track title + artist
    // -----------------------------------------------------------------------

    fn draw_song_view_header(&mut self, ui: &mut egui::Ui, alpha: u8) {
        let fade = |c: Color32| {
            Color32::from_rgba_unmultiplied(c.r(), c.g(), c.b(), (c.a() as u32 * alpha as u32 / 255) as u8)
        };
        ui.horizontal(|ui| {
            ui.add_space(24.0);
            // Back / close button.
            let (back_rect, back_resp) =
                ui.allocate_exact_size(Vec2::splat(36.0), Sense::click());
            let back_color = if back_resp.hovered() { TEXT_BRIGHT } else { TEXT_MUTED };
            paint_icon(ui.painter(), back_rect, Icon::ChevronLeft, fade(back_color));
            if back_resp.clicked() {
                self.close_song_view();
            }
            if back_resp.hovered() {
                ui.ctx().set_cursor_icon(egui::CursorIcon::PointingHand);
            }

            ui.add_space(8.0);
            ui.vertical(|ui| {
                ui.add_space(6.0);
                if let Some(track) = self.current_track() {
                    ui.label(
                        RichText::new(&track.title)
                            .size(14.0)
                            .strong()
                            .color(fade(TEXT_BRIGHT)),
                    );
                    ui.label(
                        RichText::new(format!("{} — {}", track.artist, track.album))
                            .size(12.0)
                            .color(fade(TEXT_MUTED)),
                    );
                } else {
                    ui.label(
                        RichText::new("Now Playing").size(14.0).color(fade(TEXT_MUTED)),
                    );
                }
            });
        });
        ui.add_space(8.0);
    }

    // -----------------------------------------------------------------------
    // Tab selector
    // -----------------------------------------------------------------------

    fn draw_song_view_tabs(&mut self, ui: &mut egui::Ui, alpha: u8) {
        let fade = |c: Color32| {
            Color32::from_rgba_unmultiplied(c.r(), c.g(), c.b(), (c.a() as u32 * alpha as u32 / 255) as u8)
        };
        ui.horizontal(|ui| {
            let available = ui.available_width();
            let tab_w = 110.0f32;
            let total = tab_w * 3.0 + 16.0 * 2.0;
            ui.add_space((available - total).max(0.0) / 2.0);

            for (tab, label) in [
                (SongViewTab::Cover, "Album"),
                (SongViewTab::Lyrics, "Songtekst"),
                (SongViewTab::Waves, "Waveform"),
            ] {
                let selected = self.song_view_tab == tab;
                let fill = if selected {
                    fade(ACCENT_GREEN_SOFT)
                } else {
                    fade(Color32::from_rgba_unmultiplied(40, 44, 50, 180))
                };
                let text_color = if selected { fade(ACCENT_GREEN) } else { fade(TEXT_MUTED) };
                let (rect, resp) = ui.allocate_exact_size(Vec2::new(tab_w, 34.0), Sense::click());

                ui.painter().rect_filled(rect, CornerRadius::same(17), fill);
                if selected {
                    ui.painter().rect_stroke(
                        rect,
                        CornerRadius::same(17),
                        Stroke::new(1.0, fade(ACCENT_GREEN)),
                        StrokeKind::Inside,
                    );
                }
                ui.painter().text(
                    rect.center(),
                    Align2::CENTER_CENTER,
                    label,
                    FontId::proportional(14.0),
                    text_color,
                );
                if resp.clicked() {
                    self.song_view_tab = tab;
                    if tab == SongViewTab::Waves {
                        self.request_waveform();
                    }
                }
                if resp.hovered() {
                    ui.ctx().set_cursor_icon(egui::CursorIcon::PointingHand);
                }
                ui.add_space(8.0);
            }
        });
        ui.add_space(12.0);
    }

    // -----------------------------------------------------------------------
    // Cover Tab — large album art + progress
    // -----------------------------------------------------------------------

    fn draw_cover_tab(&mut self, ui: &mut egui::Ui, alpha: u8) {
        let fade = move |c: Color32| {
            Color32::from_rgba_unmultiplied(c.r(), c.g(), c.b(), (c.a() as u32 * alpha as u32 / 255) as u8)
        };

        let track = self.current_track().cloned();
        let art_tex = track.as_ref().and_then(|t| {
            self.get_art_texture(ui.ctx(), &t.artist, &t.album)
        });

        let available = ui.available_rect_before_wrap();
        let art_size = (available.height() * 0.55).min(available.width() * 0.5).max(160.0);

        ui.vertical_centered(|ui| {
            ui.add_space(16.0);

            // --- Album art square ---
            let (art_rect, art_resp) =
                ui.allocate_exact_size(Vec2::splat(art_size), Sense::hover());
            if ui.is_rect_visible(art_rect) {
                let corner = CornerRadius::same(16);
                let painter = ui.painter();

                // Shadow beneath the art.
                for i in 1..=4 {
                    let expand = i as f32 * 4.0;
                    let shadow_rect = art_rect.translate(egui::vec2(0.0, 4.0 + i as f32 * 2.0));
                    painter.rect_filled(
                        shadow_rect.expand(expand),
                        CornerRadius::same(16 + i as u8 * 2),
                        Color32::from_black_alpha((60 - i * 12) as u8),
                    );
                }

                if let Some(ref tex) = art_tex {
                    let uv = egui::Rect::from_min_max(egui::pos2(0.0, 0.0), egui::pos2(1.0, 1.0));
                    painter.add(Shape::image(tex.id(), art_rect, uv, Color32::WHITE));
                    painter.rect_stroke(
                        art_rect,
                        corner,
                        Stroke::new(1.0, Color32::from_white_alpha(30)),
                        StrokeKind::Inside,
                    );
                } else if let Some(ref t) = track {
                    use crate::ui::paint_vertical_gradient;
                    let base = art_color(&t.album);
                    painter.rect_filled(art_rect, corner, base);
                    let clip = painter.with_clip_rect(art_rect);
                    paint_vertical_gradient(&clip, art_rect, lighten(base, 0.25), base);
                    if let Some(ch) = t.album.chars().find(|c| c.is_alphanumeric()) {
                        painter.text(
                            art_rect.center(),
                            Align2::CENTER_CENTER,
                            ch.to_ascii_uppercase(),
                            FontId::proportional(art_size * 0.38),
                            Color32::from_white_alpha(200),
                        );
                    }
                    painter.rect_stroke(
                        art_rect,
                        corner,
                        Stroke::new(1.0, Color32::from_white_alpha(20)),
                        StrokeKind::Inside,
                    );
                }
            }
            let _ = art_resp;

            ui.add_space(24.0);

            // Track / artist / album labels.
            if let Some(ref t) = track {
                ui.label(
                    RichText::new(&t.title)
                        .size(26.0)
                        .strong()
                        .color(fade(TEXT_BRIGHT)),
                );
                ui.add_space(4.0);
                ui.label(
                    RichText::new(format!("{} — {}", t.artist, t.album))
                        .size(15.0)
                        .color(fade(TEXT_MUTED)),
                );
            }

            ui.add_space(20.0);

            // Progress + time.
            let (position, total) = self
                .audio_player
                .as_ref()
                .map(|p| (p.position(), p.total()))
                .unwrap_or((Duration::ZERO, Duration::ZERO));
            let fraction = if total > Duration::ZERO {
                position.as_secs_f32() / total.as_secs_f32()
            } else {
                0.0
            };

            // Progress bar (wider version).
            let bar_w = art_size.min(500.0);
            let (bar_rect, _) =
                ui.allocate_exact_size(Vec2::new(bar_w, 6.0), Sense::hover());
            if ui.is_rect_visible(bar_rect) {
                let p = ui.painter();
                p.rect_filled(bar_rect, CornerRadius::same(3), fade(SURFACE_HOVER));
                let mut filled = bar_rect;
                filled.set_width(bar_rect.width() * fraction.clamp(0.0, 1.0));
                p.rect_filled(filled, CornerRadius::same(3), fade(ACCENT_GREEN));
            }

            ui.add_space(6.0);
            ui.horizontal(|ui| {
                let offset = (available.width() - bar_w) / 2.0;
                ui.add_space(offset.max(0.0));
                ui.label(
                    RichText::new(format_duration(position.as_secs() as i64))
                        .color(fade(TEXT_FAINT))
                        .small(),
                );
                ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
                    ui.add_space(offset.max(0.0));
                    let total_str = if total > Duration::ZERO {
                        format_duration(total.as_secs() as i64)
                    } else {
                        "--:--".to_string()
                    };
                    ui.label(
                        RichText::new(total_str).color(fade(TEXT_FAINT)).small(),
                    );
                });
            });
        });
    }

    // -----------------------------------------------------------------------
    // Lyrics Tab
    // -----------------------------------------------------------------------

    fn draw_lyrics_tab(&mut self, ui: &mut egui::Ui, alpha: u8) {
        let fade = move |c: Color32| {
            Color32::from_rgba_unmultiplied(c.r(), c.g(), c.b(), (c.a() as u32 * alpha as u32 / 255) as u8)
        };

        // Get current position for sync.
        let position_secs = self
            .audio_player
            .as_ref()
            .map(|p| p.position().as_secs_f64())
            .unwrap_or(0.0);

        let lyrics = self.lyrics_data.clone();

        match &lyrics {
            None => {
                ui.vertical_centered(|ui| {
                    ui.add_space(60.0);
                    ui.label(
                        RichText::new("Geen songtekst beschikbaar")
                            .size(22.0)
                            .color(fade(TEXT_MUTED)),
                    );
                    ui.add_space(12.0);
                    ui.label(
                        RichText::new(
                            "Voeg een .lrc bestand toe naast je audiobestand,\n\
                             of embed de tekst als ID3-tag (USLT).",
                        )
                        .size(14.0)
                        .color(fade(TEXT_FAINT)),
                    );
                });
            }
            Some(LyricsData::Plain(text)) => {
                let text = text.clone();
                ScrollArea::vertical()
                    .id_salt("lyrics_scroll_plain")
                    .show(ui, |ui| {
                        ui.vertical_centered(|ui| {
                            ui.add_space(16.0);
                            for line in text.lines() {
                                if line.trim().is_empty() {
                                    ui.add_space(8.0);
                                } else {
                                    ui.label(
                                        RichText::new(line)
                                            .size(17.0)
                                            .color(fade(TEXT_BRIGHT)),
                                    );
                                }
                            }
                            ui.add_space(16.0);
                        });
                    });
            }
            Some(LyricsData::Synced(lines)) => {
                let lines = lines.clone();
                // Find current line index.
                let current_idx = lines
                    .iter()
                    .rposition(|(ts, _)| *ts <= position_secs)
                    .unwrap_or(0);

                // Keep repainting while playing so the highlight moves.
                if self.audio_player.as_ref().is_some_and(|p| p.has_track() && !p.is_paused()) {
                    ui.ctx().request_repaint_after(Duration::from_millis(100));
                }

                ScrollArea::vertical()
                    .id_salt("lyrics_scroll_synced")
                    .auto_shrink([false; 2])
                    .show(ui, |ui| {
                        ui.vertical_centered(|ui| {
                            ui.add_space(ui.available_height() * 0.3);

                            for (idx, (_, line)) in lines.iter().enumerate() {
                                let dist = (idx as isize - current_idx as isize).unsigned_abs();
                                let (size, color, bold) = match dist {
                                    0 => (22.0, fade(TEXT_BRIGHT), true),
                                    1 => (17.0, fade(TEXT_MUTED), false),
                                    2 => (15.0, fade(Color32::from_rgba_unmultiplied(155, 163, 166, 140)), false),
                                    _ => (14.0, fade(Color32::from_rgba_unmultiplied(100, 108, 112, 90)), false),
                                };

                                let mut text = RichText::new(line.as_str()).size(size).color(color);
                                if bold {
                                    text = text.strong();
                                }
                                let label_resp = ui.add(
                                    egui::Label::new(text).selectable(false),
                                );
                                // Auto-scroll to current line.
                                if idx == current_idx {
                                    label_resp.scroll_to_me(Some(egui::Align::Center));
                                }
                                ui.add_space(if dist == 0 { 10.0 } else { 6.0 });
                            }

                            ui.add_space(ui.available_height() * 0.3);
                        });
                    });
            }
        }
    }

    // -----------------------------------------------------------------------
    // Waves Tab — decoded waveform visualization
    // -----------------------------------------------------------------------

    fn draw_waves_tab(&mut self, ui: &mut egui::Ui, alpha: u8) {
        let fade = move |c: Color32| {
            Color32::from_rgba_unmultiplied(c.r(), c.g(), c.b(), (c.a() as u32 * alpha as u32 / 255) as u8)
        };

        // Kick off computation if not started yet.
        if self.waveform_data.is_none() && self.waveform_receiver.is_none() {
            self.request_waveform();
        }

        // Keep repainting while computing or playing.
        if self.waveform_receiver.is_some()
            || self
                .audio_player
                .as_ref()
                .is_some_and(|p| p.has_track() && !p.is_paused())
        {
            ui.ctx().request_repaint_after(Duration::from_millis(80));
        }

        let position_frac = {
            let (pos, total) = self
                .audio_player
                .as_ref()
                .map(|p| (p.position(), p.total()))
                .unwrap_or((Duration::ZERO, Duration::ZERO));
            if total > Duration::ZERO {
                (pos.as_secs_f32() / total.as_secs_f32()).clamp(0.0, 1.0)
            } else {
                0.0
            }
        };

        ui.vertical_centered(|ui| {
            if self.waveform_receiver.is_some() {
                // Loading spinner.
                ui.add_space(60.0);
                ui.spinner();
                ui.add_space(12.0);
                ui.label(
                    RichText::new("Waveform analyseren…")
                        .color(fade(TEXT_MUTED))
                        .size(14.0),
                );
                return;
            }

            let Some(ref data) = self.waveform_data else {
                ui.add_space(60.0);
                ui.label(
                    RichText::new("Waveform niet beschikbaar.")
                        .color(fade(TEXT_MUTED))
                        .size(14.0),
                );
                return;
            };

            let wave_w = ui.available_width().min(900.0);
            let wave_h = 140.0f32;
            ui.add_space(24.0);

            let (rect, _) = ui.allocate_exact_size(Vec2::new(wave_w, wave_h), Sense::hover());
            if !ui.is_rect_visible(rect) {
                return;
            }

            let painter = ui.painter();
            let n = data.len().max(1);
            let bar_w = (wave_w / n as f32).max(1.0);
            let passed_x = rect.left() + wave_w * position_frac;
            let center_y = rect.center().y;

            for (i, &amp) in data.iter().enumerate() {
                let x = rect.left() + i as f32 * bar_w;
                let half_h = (amp * wave_h * 0.48).max(1.5);

                let played = x < passed_x;
                let base_color = if played { ACCENT_GREEN } else { SURFACE_HOVER };
                let color = fade(base_color);

                // Mirror bars (above and below center line).
                painter.rect_filled(
                    Rect::from_min_max(
                        egui::pos2(x, center_y - half_h),
                        egui::pos2(x + bar_w - 0.8, center_y),
                    ),
                    CornerRadius::same(1),
                    color,
                );
                painter.rect_filled(
                    Rect::from_min_max(
                        egui::pos2(x, center_y),
                        egui::pos2(x + bar_w - 0.8, center_y + half_h * 0.6),
                    ),
                    CornerRadius::same(1),
                    Color32::from_rgba_unmultiplied(color.r(), color.g(), color.b(), color.a() / 2),
                );
            }

            // Playhead — glowing vertical line at current position.
            let ph_x = passed_x.clamp(rect.left(), rect.right());
            painter.vline(ph_x, rect.y_range(), Stroke::new(2.0, fade(ACCENT_GREEN)));
            // Glow rings.
            for r in [8.0f32, 5.0, 3.0] {
                painter.circle_filled(
                    egui::pos2(ph_x, center_y),
                    r,
                    Color32::from_rgba_unmultiplied(
                        ACCENT_GREEN.r(),
                        ACCENT_GREEN.g(),
                        ACCENT_GREEN.b(),
                        (50.0 / r) as u8,
                    ),
                );
            }

            // Time labels below.
            ui.add_space(10.0);
            let (pos, total) = self
                .audio_player
                .as_ref()
                .map(|p| (p.position(), p.total()))
                .unwrap_or((Duration::ZERO, Duration::ZERO));

            ui.horizontal(|ui| {
                let side_pad = (ui.available_width() - wave_w) / 2.0;
                ui.add_space(side_pad.max(0.0));
                ui.label(
                    RichText::new(format_duration(pos.as_secs() as i64))
                        .color(fade(TEXT_FAINT))
                        .small(),
                );
                ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
                    ui.add_space(side_pad.max(0.0));
                    let total_str = if total > Duration::ZERO {
                        format_duration(total.as_secs() as i64)
                    } else {
                        "--:--".to_string()
                    };
                    ui.label(RichText::new(total_str).color(fade(TEXT_FAINT)).small());
                });
            });

            // Artist metadata section below waveform.
            ui.add_space(24.0);
            if let Some(track) = self.current_track() {
                ui.label(
                    RichText::new(&track.title)
                        .size(20.0)
                        .strong()
                        .color(fade(TEXT_BRIGHT)),
                );
                ui.add_space(4.0);
                ui.label(
                    RichText::new(format!("{} — {}", track.artist, track.album))
                        .size(14.0)
                        .color(fade(TEXT_MUTED)),
                );
            }
        });
    }
}

// ---------------------------------------------------------------------------
// Public helpers (used by other ui modules)
// ---------------------------------------------------------------------------

/// Check whether the user clicked the playbar art (delegated here to keep
/// play_bar.rs minimal). Returns `true` if the song view should open.
#[allow(dead_code)]
pub fn should_open_song_view(clicked: bool, has_track: bool) -> bool {
    clicked && has_track
}
