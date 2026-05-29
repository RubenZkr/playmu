use eframe::egui::{self, Color32, CornerRadius, Rect, Sense, Stroke, Vec2};

#[derive(Clone, Copy)]
pub enum Icon {
    Play,
    Pause,
    Prev,
    Next,
    Search,
    Volume,
    VolumeLow,
    Mute,
    Equalizer,
    Shuffle,
    Repeat,
    RepeatOne,
    ChevronLeft,
    ChevronRight,
    Close,
}

pub fn paint_icon(painter: &egui::Painter, rect: Rect, icon: Icon, color: Color32) {
    use egui::{pos2, vec2, Shape};
    let c = rect.center();
    let s = rect.width().min(rect.height());
    let sw = (s * 0.1).max(1.5);
    let stroke = Stroke::new(sw, color);
    let solid = Stroke::NONE;

    match icon {
        Icon::Play => {
            let r = s * 0.32;
            painter.add(Shape::convex_polygon(
                vec![
                    pos2(c.x - r * 0.5, c.y - r),
                    pos2(c.x - r * 0.5, c.y + r),
                    pos2(c.x + r, c.y),
                ],
                color,
                solid,
            ));
        }
        Icon::Pause => {
            let bw = s * 0.13;
            let bh = s * 0.62;
            let gap = s * 0.11;
            painter.rect_filled(
                Rect::from_center_size(pos2(c.x - gap - bw * 0.5, c.y), vec2(bw, bh)),
                CornerRadius::same(2),
                color,
            );
            painter.rect_filled(
                Rect::from_center_size(pos2(c.x + gap + bw * 0.5, c.y), vec2(bw, bh)),
                CornerRadius::same(2),
                color,
            );
        }
        Icon::Next | Icon::Prev => {
            let dir = if matches!(icon, Icon::Next) { 1.0 } else { -1.0 };
            let r = s * 0.26;
            painter.add(Shape::convex_polygon(
                vec![
                    pos2(c.x - dir * r * 0.9, c.y - r),
                    pos2(c.x - dir * r * 0.9, c.y + r),
                    pos2(c.x + dir * r * 0.3, c.y),
                ],
                color,
                solid,
            ));
            painter.rect_filled(
                Rect::from_center_size(pos2(c.x + dir * r * 0.6, c.y), vec2(s * 0.1, r * 2.0)),
                CornerRadius::same(1),
                color,
            );
        }
        Icon::Search => {
            let rad = s * 0.26;
            let cc = pos2(c.x - s * 0.08, c.y - s * 0.08);
            painter.circle_stroke(cc, rad, stroke);
            let dir = vec2(1.0, 1.0).normalized();
            painter.line_segment([cc + dir * rad, cc + dir * (rad + s * 0.3)], stroke);
        }
        Icon::Volume | Icon::VolumeLow | Icon::Mute => {
            let back =
                Rect::from_center_size(pos2(c.x - s * 0.24, c.y), vec2(s * 0.14, s * 0.26));
            painter.rect_filled(back, CornerRadius::same(1), color);
            painter.add(Shape::convex_polygon(
                vec![
                    pos2(back.right(), c.y - s * 0.13),
                    pos2(c.x - s * 0.02, c.y - s * 0.3),
                    pos2(c.x - s * 0.02, c.y + s * 0.3),
                    pos2(back.right(), c.y + s * 0.13),
                ],
                color,
                solid,
            ));
            let wave = |painter: &egui::Painter, x: f32, h: f32| {
                painter.line_segment(
                    [pos2(x, c.y - h), pos2(x + s * 0.07, c.y)],
                    Stroke::new(sw * 0.9, color),
                );
                painter.line_segment(
                    [pos2(x + s * 0.07, c.y), pos2(x, c.y + h)],
                    Stroke::new(sw * 0.9, color),
                );
            };
            match icon {
                Icon::Volume => {
                    wave(painter, c.x + s * 0.08, s * 0.12);
                    wave(painter, c.x + s * 0.2, s * 0.2);
                }
                Icon::VolumeLow => wave(painter, c.x + s * 0.08, s * 0.12),
                _ => {
                    let mx = c.x + s * 0.18;
                    let d = s * 0.11;
                    painter.line_segment([pos2(mx - d, c.y - d), pos2(mx + d, c.y + d)], stroke);
                    painter.line_segment([pos2(mx - d, c.y + d), pos2(mx + d, c.y - d)], stroke);
                }
            }
        }
        Icon::Equalizer => {
            let bw = s * 0.16;
            let gap = s * 0.12;
            let heights = [0.5_f32, 0.86, 0.62];
            for (i, h) in heights.iter().enumerate() {
                let x = c.x + (i as f32 - 1.0) * (bw + gap);
                painter.rect_filled(
                    Rect::from_center_size(pos2(x, c.y), vec2(bw, s * h)),
                    CornerRadius::same(1),
                    color,
                );
            }
        }
        // Animated equalizer — caller passes heights [0..1] per bar.
        // For static drawing use Equalizer instead.

        Icon::Shuffle => {
            // Two diagonal arrows forming an X-cross shuffle icon.
            let lw = sw * 1.1;
            let st = Stroke::new(lw, color);
            // Top-left → bottom-right arrow
            let p1 = pos2(c.x - s * 0.33, c.y - s * 0.2);
            let p2 = pos2(c.x + s * 0.33, c.y + s * 0.2);
            painter.line_segment([p1, p2], st);
            // arrowhead right
            let tip = p2;
            painter.line_segment([tip, pos2(tip.x - s * 0.12, tip.y - s * 0.06)], st);
            painter.line_segment([tip, pos2(tip.x - s * 0.06, tip.y + s * 0.12)], st);
            // Bottom-left → top-right arrow
            let q1 = pos2(c.x - s * 0.33, c.y + s * 0.2);
            let q2 = pos2(c.x + s * 0.33, c.y - s * 0.2);
            painter.line_segment([q1, q2], st);
            // arrowhead right-up
            let tip2 = q2;
            painter.line_segment([tip2, pos2(tip2.x - s * 0.12, tip2.y + s * 0.06)], st);
            painter.line_segment([tip2, pos2(tip2.x - s * 0.06, tip2.y - s * 0.12)], st);
        }
        Icon::Repeat | Icon::RepeatOne => {
            // Circular arrow (↺) with an arrowhead.
            let rad = s * 0.3;
            // Draw ~270° of a circle arc approximated as a polyline.
            let steps = 18_usize;
            let start_angle: f32 = 0.35; // radians, slightly off top
            let end_angle: f32 = std::f32::consts::TAU - 0.35;
            let pts: Vec<egui::Pos2> = (0..=steps)
                .map(|i| {
                    let t = start_angle + (end_angle - start_angle) * (i as f32 / steps as f32);
                    pos2(c.x + rad * t.cos(), c.y - rad * t.sin())
                })
                .collect();
            for pair in pts.windows(2) {
                painter.line_segment([pair[0], pair[1]], stroke);
            }
            // Arrowhead at end of arc.
            let tip = *pts.last().unwrap();
            let before = pts[pts.len() - 2];
            let dir = (tip - before).normalized();
            let perp = vec2(-dir.y, dir.x);
            let ahs = s * 0.14;
            painter.add(Shape::convex_polygon(
                vec![tip, tip - dir * ahs + perp * ahs * 0.5, tip - dir * ahs - perp * ahs * 0.5],
                color,
                solid,
            ));
            // For RepeatOne, draw a tiny "1" in the center.
            if matches!(icon, Icon::RepeatOne) {
                use eframe::egui::{Align2, FontId};
                painter.text(
                    c,
                    Align2::CENTER_CENTER,
                    "1",
                    FontId::proportional(s * 0.35),
                    color,
                );
            }
        }
        Icon::ChevronLeft | Icon::ChevronRight => {
            let dir = if matches!(icon, Icon::ChevronRight) { 1.0 } else { -1.0 };
            let arm = s * 0.28;
            let mid = c.x + dir * s * 0.05;
            let tip = pos2(mid + dir * arm * 0.5, c.y);
            let top = pos2(mid - dir * arm * 0.5, c.y - arm);
            let bot = pos2(mid - dir * arm * 0.5, c.y + arm);
            painter.line_segment([top, tip], stroke);
            painter.line_segment([bot, tip], stroke);
        }
        Icon::Close => {
            let r = s * 0.28;
            painter.line_segment([pos2(c.x - r, c.y - r), pos2(c.x + r, c.y + r)], stroke);
            painter.line_segment([pos2(c.x + r, c.y - r), pos2(c.x - r, c.y + r)], stroke);
        }
    }
}

/// Paint animated equalizer bars (call each frame while playing).
pub fn paint_equalizer_animated(
    painter: &egui::Painter,
    rect: Rect,
    color: Color32,
    time: f64,
) {
    use egui::{pos2, vec2};
    let c = rect.center();
    let s = rect.width().min(rect.height());
    let bw = s * 0.16;
    let gap = s * 0.12;
    // Each bar oscillates at a different rate and phase.
    let configs: [(f64, f64); 3] = [(3.7, 0.0), (5.1, 1.2), (4.3, 2.5)];
    for (i, (freq, phase)) in configs.iter().enumerate() {
        let raw = ((time * freq + phase).sin() * 0.5 + 0.5) as f32;
        let h = 0.28 + raw * 0.62;
        let x = c.x + (i as f32 - 1.0) * (bw + gap);
        painter.rect_filled(
            Rect::from_center_size(pos2(x, c.y), vec2(bw, s * h)),
            CornerRadius::same(1),
            color,
        );
    }
}

/// Allocate a square slot and paint a vector icon into it.
pub fn icon_widget(ui: &mut egui::Ui, size: f32, icon: Icon, color: Color32) -> egui::Response {
    let (rect, response) = ui.allocate_exact_size(Vec2::splat(size), Sense::hover());
    if ui.is_rect_visible(rect) {
        paint_icon(ui.painter(), rect, icon, color);
    }
    response
}
