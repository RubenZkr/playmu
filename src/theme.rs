use eframe::egui::{self, Color32, CornerRadius, FontId, Margin, Stroke};

pub const ACCENT_GREEN: Color32 = Color32::from_rgb(30, 215, 96);
pub const ACCENT_GREEN_HOVER: Color32 = Color32::from_rgb(54, 226, 117);
pub const ACCENT_GREEN_SOFT: Color32 = Color32::from_rgb(30, 78, 52);
pub const BG_BASE: Color32 = Color32::from_rgb(8, 9, 11);
pub const PANEL_DARK: Color32 = Color32::from_rgb(13, 15, 17);
pub const PANEL_SOFT: Color32 = Color32::from_rgb(20, 23, 26);
pub const SURFACE: Color32 = Color32::from_rgb(26, 30, 34);
pub const SURFACE_HOVER: Color32 = Color32::from_rgb(36, 41, 47);
pub const CARD_STROKE: Color32 = Color32::from_rgb(38, 43, 49);
pub const TEXT_BRIGHT: Color32 = Color32::from_rgb(244, 246, 247);
pub const TEXT_MUTED: Color32 = Color32::from_rgb(155, 163, 166);
pub const TEXT_FAINT: Color32 = Color32::from_rgb(108, 114, 120);

pub const ART_PALETTE: [Color32; 8] = [
    Color32::from_rgb(45, 92, 219),
    Color32::from_rgb(155, 64, 217),
    Color32::from_rgb(214, 73, 122),
    Color32::from_rgb(221, 120, 51),
    Color32::from_rgb(38, 166, 154),
    Color32::from_rgb(67, 160, 71),
    Color32::from_rgb(197, 57, 57),
    Color32::from_rgb(58, 123, 213),
];

pub fn configure_theme(ctx: &egui::Context) {
    let mut visuals = egui::Visuals::dark();
    visuals.override_text_color = Some(TEXT_BRIGHT);
    visuals.panel_fill = BG_BASE;
    visuals.faint_bg_color = PANEL_SOFT;
    visuals.extreme_bg_color = PANEL_DARK;
    visuals.window_fill = PANEL_SOFT;
    visuals.window_stroke = Stroke::new(1.0, CARD_STROKE);

    let card_radius = CornerRadius::same(12);
    let widget_radius = CornerRadius::same(10);
    visuals.window_corner_radius = card_radius;
    visuals.menu_corner_radius = widget_radius;

    visuals.window_shadow = egui::epaint::Shadow {
        offset: [0, 8],
        blur: 24,
        spread: 0,
        color: Color32::from_black_alpha(120),
    };
    visuals.popup_shadow = egui::epaint::Shadow {
        offset: [0, 6],
        blur: 18,
        spread: 0,
        color: Color32::from_black_alpha(110),
    };

    visuals.widgets.noninteractive.bg_fill = PANEL_SOFT;
    visuals.widgets.noninteractive.corner_radius = card_radius;
    visuals.widgets.noninteractive.bg_stroke = Stroke::new(1.0, CARD_STROKE);

    for widget in [
        &mut visuals.widgets.inactive,
        &mut visuals.widgets.hovered,
        &mut visuals.widgets.active,
        &mut visuals.widgets.open,
    ] {
        widget.corner_radius = widget_radius;
    }

    visuals.widgets.inactive.bg_fill = SURFACE;
    visuals.widgets.inactive.weak_bg_fill = SURFACE;
    visuals.widgets.inactive.bg_stroke = Stroke::NONE;
    visuals.widgets.inactive.fg_stroke = Stroke::new(1.0, TEXT_MUTED);

    visuals.widgets.hovered.bg_fill = SURFACE_HOVER;
    visuals.widgets.hovered.weak_bg_fill = SURFACE_HOVER;
    visuals.widgets.hovered.bg_stroke = Stroke::new(1.0, CARD_STROKE);
    visuals.widgets.hovered.fg_stroke = Stroke::new(1.5, TEXT_BRIGHT);
    visuals.widgets.hovered.expansion = 1.0;

    visuals.widgets.active.bg_fill = ACCENT_GREEN_SOFT;
    visuals.widgets.active.weak_bg_fill = ACCENT_GREEN_SOFT;
    visuals.widgets.active.bg_stroke = Stroke::NONE;
    visuals.widgets.active.fg_stroke = Stroke::new(1.5, TEXT_BRIGHT);

    visuals.selection.bg_fill = ACCENT_GREEN_SOFT;
    visuals.selection.stroke = Stroke::new(1.0, ACCENT_GREEN);

    ctx.set_visuals(visuals);

    let mut style = (*ctx.style()).clone();
    style.spacing.item_spacing = egui::vec2(10.0, 10.0);
    style.spacing.button_padding = egui::vec2(16.0, 10.0);
    style.spacing.window_margin = Margin::same(16);
    style.spacing.menu_margin = Margin::same(8);
    style.spacing.scroll.floating = true;
    style.spacing.scroll.bar_width = 8.0;
    style.spacing.scroll.floating_width = 8.0;

    use egui::{FontFamily, TextStyle};
    style.text_styles.insert(
        TextStyle::Body,
        FontId::new(15.0, FontFamily::Proportional),
    );
    style.text_styles.insert(
        TextStyle::Button,
        FontId::new(15.0, FontFamily::Proportional),
    );
    style.text_styles.insert(
        TextStyle::Small,
        FontId::new(12.0, FontFamily::Proportional),
    );
    ctx.set_style(style);
}
