use eframe::egui;
use crate::app::WavesApp;

pub fn render_status_bar(app: &WavesApp, ctx: &egui::Context) {
    if app.config.show_status_bar {
        egui::TopBottomPanel::bottom("status")
            .frame(egui::Frame::default().fill(egui::Color32::from_rgb(8, 8, 8)))
            .show(ctx, |ui| {
                ui.separator();
                let volume_percent = (app.volume * 100.0) as i32;
                let status_text = format!(" h/j/k/l: navigate | ENTER: select/play | SPACE: pause | ←/→: prev/next | TAB: view | ↑/↓: vol ({}%) | ?: help", volume_percent);
                ui.label(egui::RichText::new(status_text).size(18.0).color(egui::Color32::WHITE).monospace());
            });
    }
}
