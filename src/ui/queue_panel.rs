use eframe::egui::{self, Color32, DragAndDrop, Margin, RichText, Sense, Vec2};

use crate::{
    app::PlaymuApp,
    theme::{ACCENT_GREEN, ACCENT_GREEN_SOFT, PANEL_DARK, PANEL_SOFT, SURFACE, TEXT_BRIGHT, TEXT_FAINT, TEXT_MUTED},
    ui::widgets::hover_highlight_row,
};

/// Payload type for queue drag-and-drop.
#[derive(Clone, Copy, Debug)]
struct QueueDrag {
    from_index: usize,
}

impl PlaymuApp {
    pub fn draw_right_panel(&mut self, ctx: &egui::Context) {
        egui::SidePanel::right("queue_panel")
            .default_width(290.0)
            .resizable(true)
            .frame(
                egui::Frame::new()
                    .fill(PANEL_DARK)
                    .inner_margin(Margin::same(14)),
            )
            .show(ctx, |ui| {
                ui.add_space(4.0);
                ui.horizontal(|ui| {
                    ui.label(RichText::new("Queue").size(20.0).strong().color(TEXT_BRIGHT));
                    if !self.queue.is_empty() {
                        ui.label(
                            RichText::new(format!("({} tracks)", self.queue.len()))
                                .color(TEXT_FAINT)
                                .small(),
                        );
                    }
                });
                ui.label(
                    RichText::new("Drag rows to reorder")
                        .color(TEXT_FAINT)
                        .size(11.0),
                );
                ui.add_space(10.0);

                let queue_tracks: Vec<crate::db::Track> =
                    self.queue_tracks().into_iter().cloned().collect();

                // Track pending swap from drag-and-drop.
                let mut swap: Option<(usize, usize)> = None;

                egui::ScrollArea::vertical()
                    .id_salt("queue_scroll")
                    .show(ui, |ui| {
                        for (index, track) in queue_tracks.iter().enumerate() {
                            let active = self.queue_position == Some(index);

                            // Drop-target highlight: show a line above this item if something is
                            // being dragged above it.
                            let is_drop_target = DragAndDrop::has_payload_of_type::<QueueDrag>(ctx);
                            if is_drop_target {
                                let (line_rect, _) = ui.allocate_exact_size(
                                    Vec2::new(ui.available_width(), 2.0),
                                    Sense::hover(),
                                );
                                // If pointer is near this item, highlight the gap.
                                if ui.rect_contains_pointer(line_rect.expand(4.0)) {
                                    if let Some(payload) =
                                        DragAndDrop::take_payload::<QueueDrag>(ctx)
                                    {
                                        let from = payload.from_index;
                                        if from != index && from + 1 != index {
                                            swap = Some((from, index));
                                        }
                                    }
                                    ui.painter().hline(
                                        line_rect.x_range(),
                                        line_rect.center().y,
                                        (2.0, ACCENT_GREEN),
                                    );
                                }
                            }

                            // Row
                            let fill = if active { ACCENT_GREEN_SOFT } else { SURFACE };
                            let response = egui::Frame::new()
                                .fill(fill)
                                .corner_radius(eframe::egui::CornerRadius::same(8))
                                .inner_margin(Margin::symmetric(10, 6))
                                .show(ui, |ui| {
                                    ui.set_min_height(44.0);
                                    ui.horizontal(|ui| {
                                        // Drag handle
                                        let (handle_rect, handle_resp) = ui.allocate_exact_size(
                                            Vec2::new(12.0, 30.0),
                                            Sense::drag(),
                                        );
                                        // Draw ⋮⋮ dots as drag handle.
                                        let painter = ui.painter();
                                        for row in 0..3 {
                                            for col in 0..2 {
                                                let dot = eframe::egui::pos2(
                                                    handle_rect.left() + 3.0 + col as f32 * 5.0,
                                                    handle_rect.center().y - 4.0 + row as f32 * 4.0,
                                                );
                                                painter.circle_filled(dot, 1.2, TEXT_FAINT);
                                            }
                                        }
                                        if handle_resp.drag_started() {
                                            DragAndDrop::set_payload(
                                                ctx,
                                                QueueDrag { from_index: index },
                                            );
                                        }
                                        if handle_resp.hovered() {
                                            ui.ctx()
                                                .set_cursor_icon(egui::CursorIcon::Grab);
                                        }

                                        ui.add_space(6.0);
                                        ui.vertical(|ui| {
                                            ui.add(
                                                egui::Label::new(
                                                    RichText::new(&track.title)
                                                        .color(if active {
                                                            ACCENT_GREEN
                                                        } else {
                                                            Color32::WHITE
                                                        })
                                                        .strong(),
                                                )
                                                .truncate(),
                                            );
                                            ui.add(
                                                egui::Label::new(
                                                    RichText::new(&track.artist)
                                                        .color(TEXT_MUTED)
                                                        .small(),
                                                )
                                                .truncate(),
                                            );
                                        });
                                    });
                                })
                                .response
                                .interact(Sense::click());

                            if response.hovered() && !is_drop_target {
                                hover_highlight_row(ui, response.rect, 8);
                            }
                            if response.clicked() {
                                self.queue_position = Some(index);
                                self.play_track(track.id);
                            }
                            ui.add_space(3.0);
                        }

                        if self.queue.is_empty() {
                            ui.add_space(16.0);
                            egui::Frame::new()
                                .fill(PANEL_SOFT)
                                .corner_radius(eframe::egui::CornerRadius::same(10))
                                .inner_margin(Margin::same(14))
                                .show(ui, |ui| {
                                    ui.label(RichText::new("Queue is empty").strong());
                                    ui.add_space(4.0);
                                    ui.label(
                                        RichText::new("Double-click a track or press Enter in search to start.")
                                            .color(TEXT_MUTED)
                                            .small(),
                                    );
                                });
                        }
                    });

                // Apply drag-and-drop reorder
                if let Some((from, to)) = swap {
                    if from < self.queue.len() && to <= self.queue.len() {
                        let id = self.queue.remove(from);
                        let insert_at = if to > from { to - 1 } else { to };
                        self.queue.insert(insert_at.min(self.queue.len()), id);
                        // Keep queue_position pointing to the same track.
                        if let Some(pos) = self.queue_position {
                            if pos == from {
                                self.queue_position = Some(insert_at);
                            }
                        }
                    }
                }

                ui.add_space(10.0);
                ui.separator();
                ui.add_space(8.0);
                ui.label(RichText::new("Selection").strong().color(TEXT_BRIGHT));
                if let Some(track) = self.selected_track() {
                    ui.add(egui::Label::new(RichText::new(&track.title).size(16.0).strong()).truncate());
                    ui.label(
                        RichText::new(format!("{} — {}", track.artist, track.album))
                            .color(TEXT_MUTED),
                    );
                    ui.label(RichText::new(&track.file_path).color(TEXT_FAINT).small());
                } else {
                    ui.label(RichText::new("Nothing selected.").color(TEXT_MUTED));
                }
            });
    }
}
