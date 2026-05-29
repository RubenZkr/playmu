mod app;
mod db;
mod library;

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
    .map_err(|error| anyhow::anyhow!(error.to_string()))
}
