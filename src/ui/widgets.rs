use eframe::egui::{self, Color32, CornerRadius, Rect, RichText, Sense, Stroke, StrokeKind, Vec2};

use crate::{
    db::Playlist,
    icons::{paint_icon, Icon},
    models::LibrarySortKey,
    theme::{
        ACCENT_GREEN, ACCENT_GREEN_HOVER, ACCENT_GREEN_SOFT, BG_BASE, PANEL_SOFT,
        SURFACE, SURFACE_HOVER, TEXT_BRIGHT, TEXT_FAINT, TEXT_MUTED,
    },
    ui::{card_frame, paint_art},
};

/// Pill-shaped navigation / tab button (full-width).
pub fn nav_button(ui: &mut egui::Ui, selected: bool, label: &str) -> egui::Response {
    let (fill, text) = if selected {
        (ACCENT_GREEN_SOFT, TEXT_BRIGHT)
    } else {
        (Color32::TRANSPARENT, TEXT_MUTED)
    };
    ui.add(
        egui::Button::new(RichText::new(label).color(text).strong())
            .fill(fill)
            .corner_radius(CornerRadius::same(10))
            .min_size(Vec2::new(ui.available_width(), 42.0)),
    )
}

/// Circular transport control with a painted vector icon.
pub fn transport_button(
    ui: &mut egui::Ui,
    icon: Icon,
    accent: bool,
    enabled: bool,
) -> egui::Response {
    let size = if accent { 46.0 } else { 38.0 };
    let fill = if accent { ACCENT_GREEN } else { Color32::TRANSPARENT };
    let response = ui.add_enabled(
        enabled,
        egui::Button::new("")
            .fill(fill)
            .corner_radius(CornerRadius::same((size / 2.0) as u8))
            .min_size(Vec2::splat(size)),
    );

    if accent && enabled && response.hovered() {
        ui.painter().circle_filled(
            response.rect.center(),
            response.rect.width() / 2.0,
            ACCENT_GREEN_HOVER,
        );
    }

    let fg = if !enabled {
        TEXT_FAINT
    } else if accent {
        BG_BASE
    } else if response.hovered() {
        TEXT_BRIGHT
    } else {
        TEXT_MUTED
    };
    let icon_rect = Rect::from_center_size(response.rect.center(), Vec2::splat(size * 0.5));
    paint_icon(ui.painter(), icon_rect, icon, fg);
    response
}

/// Slim, non-interactive playback progress track.
pub fn draw_progress_bar(ui: &mut egui::Ui, fraction: f32) {
    let width = ui.available_width().max(40.0);
    let (rect, _) = ui.allocate_exact_size(Vec2::new(width, 6.0), Sense::hover());
    if !ui.is_rect_visible(rect) {
        return;
    }
    let painter = ui.painter();
    painter.rect_filled(rect, CornerRadius::same(3), SURFACE_HOVER);
    let mut filled = rect;
    filled.set_width(rect.width() * fraction.clamp(0.0, 1.0));
    painter.rect_filled(filled, CornerRadius::same(3), ACCENT_GREEN);
}

/// Three-value stat card (e.g. "TRACKS / 1234 / subtitle").
pub fn draw_stat_card(ui: &mut egui::Ui, title: &str, value: String, subtitle: &str) {
    card_frame(SURFACE).show(ui, |ui| {
        ui.set_min_size(egui::vec2(210.0, 118.0));
        ui.label(
            RichText::new(title.to_uppercase())
                .color(TEXT_MUTED)
                .strong()
                .size(12.0),
        );
        ui.add_space(8.0);
        ui.label(RichText::new(value).size(36.0).strong().color(ACCENT_GREEN));
        ui.add_space(8.0);
        ui.label(RichText::new(subtitle).color(TEXT_FAINT).small());
    });
}

/// Album / artist / mix card with art thumbnail. Returns a clickable Response.
pub fn draw_collection_card(
    ui: &mut egui::Ui,
    title: &str,
    subtitle: &str,
    meta: &str,
    fill: Color32,
) -> egui::Response {
    let response = card_frame(fill)
        .show(ui, |ui| {
            ui.set_min_size(egui::vec2(240.0, 92.0));
            ui.horizontal(|ui| {
                paint_art(ui, title, None, 64.0, 10);
                ui.add_space(12.0);
                ui.vertical(|ui| {
                    ui.label(RichText::new(title).size(17.0).strong().color(TEXT_BRIGHT));
                    ui.add_space(2.0);
                    ui.label(RichText::new(subtitle).color(TEXT_MUTED));
                    ui.add_space(6.0);
                    ui.label(RichText::new(meta).color(TEXT_FAINT).small());
                });
            });
        })
        .response
        .interact(Sense::click());

    if response.hovered() {
        ui.painter().rect_stroke(
            response.rect,
            CornerRadius::same(12),
            Stroke::new(1.5, ACCENT_GREEN),
            StrokeKind::Inside,
        );
        ui.ctx().set_cursor_icon(egui::CursorIcon::PointingHand);
    }
    response
}

/// Clickable sort column header for the songs table.
pub fn draw_library_sort_cell(
    ui: &mut egui::Ui,
    active_sort: LibrarySortKey,
    ascending: bool,
    cell_sort: LibrarySortKey,
) -> egui::Response {
    let arrow = if active_sort == cell_sort {
        if ascending { " ▲" } else { " ▼" }
    } else {
        ""
    };
    let label = format!("{}{}", cell_sort.label(), arrow);
    ui.add_sized(
        [ui.available_width(), 30.0],
        egui::Button::new(RichText::new(label).strong().color(Color32::WHITE)).fill(PANEL_SOFT),
    )
}

/// Small ghost "+" button. Returns the chosen playlist id, or None.
pub fn add_to_playlist_button(
    ui: &mut egui::Ui,
    track_id: i64,
    playlists: &[Playlist],
    _popup_open: bool,
) -> Option<i64> {
    let button = ui.add(
        egui::Button::new(RichText::new("+").color(TEXT_MUTED))
            .fill(Color32::TRANSPARENT)
            .min_size(Vec2::splat(22.0)),
    );

    let mut chosen = None;
    // Use the Popup builder with a stable id derived from the track.
    let popup_id = egui::Id::new("add_pl").with(track_id);
    egui::Popup::from_toggle_button_response(&button)
        .id(popup_id)
        .close_behavior(egui::PopupCloseBehavior::CloseOnClickOutside)
        .show(|ui| {
            ui.set_min_width(180.0);
            ui.label(RichText::new("Add to playlist").strong().color(TEXT_MUTED).small());
            ui.add_space(4.0);
            if playlists.is_empty() {
                ui.label(RichText::new("No playlists yet.").color(TEXT_MUTED).small());
            }
            for pl in playlists {
                if ui
                    .button(RichText::new(&pl.name).color(TEXT_BRIGHT))
                    .clicked()
                {
                    chosen = Some(pl.id);
                }
            }
        });
    chosen
}

/// Stack a translucent overlay on top of a hovered row rect.
pub fn hover_highlight_row(ui: &mut egui::Ui, rect: egui::Rect, corner_radius: u8) {
    ui.painter().rect_filled(
        rect,
        CornerRadius::same(corner_radius),
        Color32::from_white_alpha(8),
    );
    ui.ctx().set_cursor_icon(egui::CursorIcon::PointingHand);
}
