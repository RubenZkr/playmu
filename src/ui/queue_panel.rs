use eframe::egui::{self, Color32, Margin, RichText};

use crate::{
    app::PlaymuApp,
    theme::{ACCENT_GREEN_SOFT, PANEL_DARK, SURFACE, TEXT_BRIGHT, TEXT_FAINT, TEXT_MUTED},
};

impl PlaymuApp {
    pub fn draw_right_panel(&mut self, ctx: &egui::Context) {
        egui::SidePanel::right("queue_panel")
            .default_width(300.0)
            .resizable(false)
            .frame(
                egui::Frame::new()
                    .fill(PANEL_DARK)
                    .inner_margin(Margin::same(16)),
            )
            .show(ctx, |ui| {
                ui.add_space(4.0);
                ui.label(RichText::new("Queue").size(22.0).strong().color(TEXT_BRIGHT));
                ui.label(
                    RichText::new("Up next in this session")
                        .color(TEXT_FAINT)
                        .small(),
                );
                ui.add_space(12.0);

                let queue_tracks: Vec<_> =
                    self.queue_tracks().into_iter().cloned().collect();

                egui::ScrollArea::vertical().show(ui, |ui| {
                    for (index, track) in queue_tracks.iter().enumerate() {
                        let active = self.queue_position == Some(index);
                        let button = egui::Button::new(
                            RichText::new(format!("{}\n{}", track.title, track.artist))
                                .color(Color32::WHITE),
                        )
                        .fill(if active { ACCENT_GREEN_SOFT } else { SURFACE });

                        if ui
                            .add_sized([ui.available_width(), 52.0], button)
                            .clicked()
                        {
                            self.queue_position = Some(index);
                            self.play_track(track.id);
                        }
                        ui.add_space(4.0);
                    }

                    if self.queue.is_empty() {
                        ui.add_space(16.0);
                        ui.label(RichText::new("Queue is empty.").strong());
                        ui.label(
                            RichText::new(
                                "Play a track from the visible results to build the queue.",
                            )
                            .color(TEXT_MUTED),
                        );
                    }
                });

                ui.add_space(12.0);
                ui.separator();
                ui.add_space(10.0);
                ui.label(RichText::new("Selection").strong());
                if let Some(track) = self.selected_track() {
                    ui.label(RichText::new(&track.title).size(18.0).strong());
                    ui.label(
                        RichText::new(format!("{} - {}", track.artist, track.album))
                            .color(TEXT_MUTED),
                    );
                    ui.small(&track.file_path);
                } else {
                    ui.label(RichText::new("Nothing selected.").color(TEXT_MUTED));
                }
            });
    }
}
