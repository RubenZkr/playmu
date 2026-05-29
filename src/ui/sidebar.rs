use eframe::egui::{self, Margin, RichText, TextEdit, Vec2};

use crate::{
    app::PlaymuApp,
    icons::{icon_widget, Icon},
    models::NavSection,
    theme::{ACCENT_GREEN, BG_BASE, PANEL_DARK, PANEL_SOFT, TEXT_BRIGHT, TEXT_FAINT, TEXT_MUTED},
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
                card_frame(PANEL_SOFT).show(ui, |ui| {
                    ui.set_width(ui.available_width());
                    ui.label(
                        RichText::new("IMPORT LOCAL MUSIC")
                            .strong()
                            .size(12.0)
                            .color(TEXT_MUTED),
                    );
                    ui.add_space(2.0);
                    ui.label(
                        RichText::new("Point Playmu at a folder on this machine.")
                            .color(TEXT_FAINT)
                            .small(),
                    );
                    ui.add_space(8.0);
                    ui.add(
                        TextEdit::singleline(&mut self.source_input)
                            .hint_text("/home/you/Music")
                            .desired_width(f32::INFINITY),
                    );
                    ui.add_space(8.0);
                    if ui
                        .add_enabled(
                            !self.is_scanning,
                            egui::Button::new(
                                RichText::new("Import Folder").strong().color(BG_BASE),
                            )
                            .fill(ACCENT_GREEN)
                            .min_size(Vec2::new(ui.available_width(), 38.0)),
                        )
                        .clicked()
                    {
                        self.start_scan();
                    }
                    if self.is_scanning {
                        ui.add_space(8.0);
                        ui.horizontal(|ui| {
                            ui.spinner();
                            ui.label(RichText::new("Scanning library...").color(TEXT_MUTED));
                        });
                    }
                });

                ui.with_layout(egui::Layout::bottom_up(egui::Align::Min), |ui| {
                    ui.add_space(4.0);
                    ui.label(RichText::new(&self.status).color(TEXT_MUTED).small());
                    ui.add_space(2.0);
                    ui.label(
                        RichText::new("STATUS")
                            .strong()
                            .size(11.0)
                            .color(TEXT_FAINT),
                    );
                    ui.add_space(10.0);
                    ui.label(
                        RichText::new(self.db_path.display().to_string())
                            .color(TEXT_FAINT)
                            .small(),
                    );
                    ui.label(
                        RichText::new("DATABASE")
                            .strong()
                            .size(11.0)
                            .color(TEXT_FAINT),
                    );
                });
            });
    }
}
