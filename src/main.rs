mod app;
mod audio;
mod db;
mod icons;
mod library;
mod models;
mod theme;
mod ui;
mod util;

use anyhow::Result;
use eframe::egui;

fn main() -> Result<()> {
    let options = eframe::NativeOptions {
        viewport: egui::ViewportBuilder::default()
            .with_inner_size([1280.0, 800.0])
            .with_min_inner_size([960.0, 600.0])
            .with_title("Playmu"),
        ..Default::default()
    };

    eframe::run_native(
        "Playmu",
        options,
        Box::new(|cc| Ok(Box::new(app::PlaymuApp::new(cc)))),
    )
    .map_err(|e| anyhow::anyhow!(e.to_string()))
}
