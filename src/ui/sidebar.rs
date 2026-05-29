use eframe::egui::{self, Margin, RichText, TextEdit, Vec2};

use crate::{
    app::PlaymuApp,
    icons::{icon_widget, paint_icon, Icon},
    models::NavSection,
    theme::{ACCENT_GREEN, BG_BASE, CARD_STROKE, PANEL_DARK, PANEL_SOFT, SURFACE, TEXT_BRIGHT, TEXT_FAINT, TEXT_MUTED},
    ui::{card_frame, widgets::nav_button},
};

impl PlaymuApp {
    pub fn draw_sidebar(&mut self, ctx: &egui::Context) {
        egui::SidePanel::left("sidebar")
            .resizable(false)
            .min_width(260.0)
            .frame(
                egui::Frame::new()
                    .fill(PANEL_DARK)
                    .inner_margin(Margin::same(16)),
            )
            .show(ctx, |ui| {
                ui.add_space(8.0);
                ui.horizontal(|ui| {
                    icon_widget(ui, 24.0, Icon::Equalizer, ACCENT_GREEN);
                    ui.add_space(8.0);
                    ui.label(
                        RichText::new("PLAYMU")
                            .size(26.0)
                            .strong()
                            .color(TEXT_BRIGHT),
                    );
                });
                ui.label(
                    RichText::new("Local music, beautifully organised")
                        .color(TEXT_FAINT)
                        .small(),
                );
                ui.add_space(18.0);

                for nav in [NavSection::Home, NavSection::Search, NavSection::Library] {
                    if nav_button(ui, self.active_nav == nav, nav.label()).clicked() {
                        self.active_nav = nav;
                    }
                    ui.add_space(4.0);
                }

                ui.add_space(18.0);

                // --- Import card ---
                card_frame(PANEL_SOFT).show(ui, |ui| {
                    ui.set_width(ui.available_width());
                    ui.label(
                        RichText::new("IMPORT LOCAL MUSIC")
                            .strong()
                            .size(12.0)
                            .color(TEXT_MUTED),
                    );
                    ui.add_space(4.0);
                    ui.add(
                        TextEdit::singleline(&mut self.source_input)
                            .hint_text("/home/you/Music")
                            .desired_width(f32::INFINITY),
                    );
                    ui.add_space(6.0);
                    if ui
                        .add_enabled(
                            !self.is_scanning,
                            egui::Button::new(
                                RichText::new("Import Folder").strong().color(BG_BASE),
                            )
                            .fill(ACCENT_GREEN)
                            .min_size(Vec2::new(ui.available_width(), 36.0)),
                        )
                        .clicked()
                    {
                        self.start_scan();
                    }
                    if self.is_scanning {
                        ui.add_space(6.0);
                        ui.horizontal(|ui| {
                            ui.spinner();
                            ui.label(RichText::new("Scanning…").color(TEXT_MUTED));
                        });
                    }
                });

                // --- Indexed folders ---
                let folders = crate::db::list_source_folders(&self.db_path).unwrap_or_default();
                if !folders.is_empty() {
                    ui.add_space(12.0);
                    ui.label(
                        RichText::new("INDEXED FOLDERS")
                            .strong()
                            .size(11.0)
                            .color(TEXT_FAINT),
                    );
                    ui.add_space(4.0);
                    let mut remove_id: Option<i64> = None;
                    let mut rescan_path: Option<String> = None;
                    for (folder_id, path) in &folders {
                        egui::Frame::new()
                            .fill(SURFACE)
                            .corner_radius(eframe::egui::CornerRadius::same(8))
                            .inner_margin(Margin::symmetric(10, 6))
                            .stroke(eframe::egui::Stroke::new(1.0, CARD_STROKE))
                            .show(ui, |ui| {
                                ui.horizontal(|ui| {
                                    // Truncate long paths
                                    let display = if path.len() > 28 {
                                        format!("…{}", &path[path.len() - 26..])
                                    } else {
                                        path.clone()
                                    };
                                    ui.label(
                                        RichText::new(display).color(TEXT_MUTED).small(),
                                    );
                                    ui.with_layout(
                                        egui::Layout::right_to_left(egui::Align::Center),
                                        |ui| {
                                            // Remove button
                                            let (r_rect, r_resp) = ui.allocate_exact_size(
                                                Vec2::splat(16.0),
                                                egui::Sense::click(),
                                            );
                                            paint_icon(
                                                ui.painter(),
                                                r_rect,
                                                Icon::Close,
                                                TEXT_FAINT,
                                            );
                                            if r_resp.clicked() {
                                                remove_id = Some(*folder_id);
                                            }
                                            if r_resp.hovered() {
                                                ui.ctx().set_cursor_icon(
                                                    egui::CursorIcon::PointingHand,
                                                );
                                            }
                                            ui.add_space(4.0);
                                            // Re-scan button
                                            if ui
                                                .add(
                                                    egui::Button::new(
                                                        RichText::new("↺").color(TEXT_MUTED),
                                                    )
                                                    .fill(egui::Color32::TRANSPARENT)
                                                    .min_size(Vec2::splat(18.0)),
                                                )
                                                .on_hover_text("Re-scan this folder")
                                                .clicked()
                                            {
                                                rescan_path = Some(path.clone());
                                            }
                                        },
                                    );
                                });
                            });
                        ui.add_space(4.0);
                    }
                    if let Some(id) = remove_id {
                        if crate::db::remove_source_folder(&self.db_path, id).is_ok() {
                            self.tracks = crate::db::list_tracks(&self.db_path).unwrap_or_default();
                            self.refresh_home_personalization();
                            self.status = "Removed folder and its tracks.".to_string();
                        }
                    }
                    if let Some(path) = rescan_path {
                        self.source_input = path;
                        self.start_scan();
                    }
                }

                // Keyboard shortcut hint
                ui.add_space(8.0);
                ui.label(
                    RichText::new("Space play/pause  ←/→ prev/next\nS shuffle  L repeat  M mute")
                        .color(TEXT_FAINT)
                        .size(11.0),
                );

                // Status + DB path pinned to bottom
                ui.with_layout(egui::Layout::bottom_up(egui::Align::Min), |ui| {
                    ui.add_space(4.0);
                    ui.label(RichText::new(&self.status).color(TEXT_MUTED).small());
                    ui.add_space(2.0);
                    ui.label(
                        RichText::new("STATUS").strong().size(11.0).color(TEXT_FAINT),
                    );
                    ui.add_space(10.0);
                    ui.label(
                        RichText::new(self.db_path.display().to_string())
                            .color(TEXT_FAINT)
                            .small(),
                    );
                    ui.label(
                        RichText::new("DATABASE").strong().size(11.0).color(TEXT_FAINT),
                    );
                });
            });
    }
}
