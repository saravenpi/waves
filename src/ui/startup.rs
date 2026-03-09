use eframe::egui;
use crate::app::WavesApp;

pub fn render_startup_screen(app: &WavesApp, ctx: &egui::Context) {
    egui::CentralPanel::default()
        .frame(egui::Frame::none().fill(egui::Color32::from_rgb(8, 8, 8)))
        .show(ctx, |ui| {
            ui.vertical_centered(|ui| {
                let available_height = ui.available_height();
                ui.add_space(available_height * 0.35);

                ui.label(
                    egui::RichText::new("Waves")
                        .size(72.0)
                        .color(app.primary_color())
                        .strong()
                );

                ui.add_space(40.0);

                crate::ui::spinner::square_spinner(ui, 60.0, app.primary_color());

                ui.add_space(20.0);

                ui.label(
                    egui::RichText::new("Music Player")
                        .size(18.0)
                        .color(egui::Color32::from_gray(180))
                );
            });
        });

    ctx.request_repaint();
}
