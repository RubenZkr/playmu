use eframe::egui::{self, CornerRadius, Margin, RichText, Stroke, TextEdit};

use crate::{
    app::PlaymuApp,
    icons::{icon_widget, Icon},
    theme::{ACCENT_GREEN, CARD_STROKE, PANEL_SOFT, TEXT_BRIGHT, TEXT_MUTED},
};

impl PlaymuApp {
    pub fn draw_top_bar(&mut self, ui: &mut egui::Ui) {
        ui.horizontal(|ui| {
            ui.vertical(|ui| {
                ui.label(
                    RichText::new(self.active_nav.label())
                        .size(30.0)
                        .strong()
                        .color(TEXT_BRIGHT),
                );
                ui.label(RichText::new(self.active_nav.subtitle()).color(TEXT_MUTED));
            });

            ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
                let focused = self.search_input_has_focus;
                let stroke = if focused {
                    Stroke::new(1.5, ACCENT_GREEN)
                } else {
                    Stroke::new(1.0, CARD_STROKE)
                };
                egui::Frame::new()
                    .fill(PANEL_SOFT)
                    .corner_radius(CornerRadius::same(20))
                    .inner_margin(Margin::symmetric(14, 8))
                    .stroke(stroke)
                    .show(ui, |ui| {
                        ui.set_width(320.0);
                        ui.horizontal(|ui| {
                            icon_widget(ui, 16.0, Icon::Search, TEXT_MUTED);
                            ui.add_space(8.0);
                            let search_response = ui.add(
                                TextEdit::singleline(&mut self.search_query)
                                    .hint_text("What do you want to play?")
                                    .frame(false)
                                    .desired_width(f32::INFINITY),
                            );
                            self.search_input_has_focus = search_response.has_focus();
                        });
                    });
            });
        });
    }
}
