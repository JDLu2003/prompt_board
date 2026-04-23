mod app;
mod db;
mod system;
mod template;

use app::PromptBoardApp;
use eframe::egui;

fn main() -> eframe::Result<()> {
    let options = eframe::NativeOptions {
        viewport: egui::ViewportBuilder::default()
            .with_title("Prompt Board")
            .with_inner_size([1290.0, 680.0])
            .with_min_inner_size([950.0, 560.0])
            .with_decorations(false)
            .with_transparent(true)
            .with_always_on_top(),
        ..Default::default()
    };

    eframe::run_native(
        "Prompt Board",
        options,
        Box::new(|cc| Ok(Box::new(PromptBoardApp::new(cc)))),
    )
}
