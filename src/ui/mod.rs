pub mod home;
pub mod library;
pub mod play_bar;
pub mod queue_panel;
pub mod search;
pub mod sidebar;
pub mod top_bar;
pub mod track_row;
pub mod widgets;

use eframe::egui::{
    self, Align2, Color32, CornerRadius, FontId, Margin, Rect, Sense, Stroke, Vec2,
};

use crate::theme::{ART_PALETTE, CARD_STROKE};

/// Padded, rounded card frame with hairline border and gentle shadow.
pub fn card_frame(fill: Color32) -> egui::Frame {
    egui::Frame::new()
        .fill(fill)
        .corner_radius(CornerRadius::same(12))
        .inner_margin(Margin::same(14))
        .stroke(Stroke::new(1.0, CARD_STROKE))
        .shadow(egui::epaint::Shadow {
            offset: [0, 2],
            blur: 10,
            spread: 0,
            color: Color32::from_black_alpha(40),
        })
}

/// Deterministically pick a vivid colour for an album/artist from its name.
pub fn art_color(seed: &str) -> Color32 {
    let mut hash: u32 = 2166136261;
    for byte in seed.bytes() {
        hash ^= u32::from(byte);
        hash = hash.wrapping_mul(16777619);
    }
    ART_PALETTE[(hash as usize) % ART_PALETTE.len()]
}

pub fn lighten(color: Color32, amount: f32) -> Color32 {
    let mix = |c: u8| -> u8 {
        let v = f32::from(c) + (255.0 - f32::from(c)) * amount;
        v.clamp(0.0, 255.0) as u8
    };
    Color32::from_rgb(mix(color.r()), mix(color.g()), mix(color.b()))
}

pub fn paint_vertical_gradient(painter: &egui::Painter, rect: Rect, top: Color32, bottom: Color32) {
    let mut mesh = egui::epaint::Mesh::default();
    mesh.colored_vertex(rect.left_top(), top);
    mesh.colored_vertex(rect.right_top(), top);
    mesh.colored_vertex(rect.left_bottom(), bottom);
    mesh.colored_vertex(rect.right_bottom(), bottom);
    mesh.add_triangle(0, 1, 2);
    mesh.add_triangle(1, 2, 3);
    painter.add(egui::Shape::mesh(mesh));
}

// ---------------------------------------------------------------------------
// Art painting — gradient placeholder or real texture
// ---------------------------------------------------------------------------

fn draw_art_background(
    painter: &egui::Painter,
    rect: Rect,
    corner: CornerRadius,
    seed: &str,
    texture: Option<&egui::TextureHandle>,
    label_alpha: u8,
) {
    use crate::icons::{paint_icon, Icon};

    if let Some(tex) = texture {
        // Real album art image.
        let uv = egui::Rect::from_min_max(egui::pos2(0.0, 0.0), egui::pos2(1.0, 1.0));
        painter.add(egui::Shape::image(tex.id(), rect, uv, Color32::WHITE));
    } else {
        // Gradient placeholder.
        let base = art_color(seed);
        painter.rect_filled(rect, corner, base);
        let clip = painter.with_clip_rect(rect);
        paint_vertical_gradient(&clip, rect, lighten(base, 0.22), base);

        match seed.chars().find(|c| c.is_alphanumeric()) {
            Some(ch) => {
                let size = rect.width().min(rect.height());
                painter.text(
                    rect.center(),
                    Align2::CENTER_CENTER,
                    ch.to_ascii_uppercase(),
                    FontId::proportional(size * 0.46),
                    Color32::from_white_alpha(label_alpha),
                );
            }
            None => {
                let gr = Rect::from_center_size(
                    rect.center(),
                    Vec2::splat(rect.width().min(rect.height()) * 0.5),
                );
                paint_icon(painter, gr, Icon::Equalizer, Color32::from_white_alpha(label_alpha));
            }
        }
    }

    // Hairline border on top of everything.
    painter.rect_stroke(
        rect,
        corner,
        Stroke::new(1.0, Color32::from_white_alpha(20)),
        egui::StrokeKind::Inside,
    );
}

/// Non-interactive album-art square (hover-only sense).
pub fn paint_art(
    ui: &mut egui::Ui,
    seed: &str,
    texture: Option<&egui::TextureHandle>,
    size: f32,
    radius: u8,
) -> egui::Response {
    let (rect, response) = ui.allocate_exact_size(Vec2::splat(size), Sense::hover());
    if ui.is_rect_visible(rect) {
        let corner = CornerRadius::same(radius);
        draw_art_background(ui.painter(), rect, corner, seed, texture, 235);
    }
    response
}

/// Interactive art square — shows a play-icon overlay on hover.
/// Returns `(response, play_clicked)`.
pub fn paint_art_play(
    ui: &mut egui::Ui,
    seed: &str,
    texture: Option<&egui::TextureHandle>,
    size: f32,
    radius: u8,
) -> (egui::Response, bool) {
    use crate::icons::{paint_icon, Icon};

    let (rect, response) = ui.allocate_exact_size(Vec2::splat(size), Sense::click());
    if !ui.is_rect_visible(rect) {
        return (response, false);
    }

    let corner = CornerRadius::same(radius);
    let hovered = response.hovered();
    let label_alpha = if hovered { 80 } else { 235 };
    draw_art_background(ui.painter(), rect, corner, seed, texture, label_alpha);

    if hovered {
        let painter = ui.painter();
        painter.rect_filled(rect, corner, Color32::from_black_alpha(140));
        let play_rect = Rect::from_center_size(rect.center(), Vec2::splat(size * 0.52));
        paint_icon(painter, play_rect, Icon::Play, Color32::WHITE);
        ui.ctx().set_cursor_icon(egui::CursorIcon::PointingHand);
    }

    (response, response.clicked())
}
