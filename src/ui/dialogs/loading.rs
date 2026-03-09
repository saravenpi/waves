use eframe::egui;

use crate::app::WavesApp;

pub fn render_loading_overlay(app: &WavesApp, ctx: &egui::Context) {
    if app.song_loading {
        let screen_rect = ctx.screen_rect();
        egui::Area::new(egui::Id::new("loading_overlay"))
            .fixed_pos(screen_rect.min)
            .show(ctx, |ui| {
                ui.allocate_ui(screen_rect.size(), |ui| {
                    ui.painter().rect_filled(
                        screen_rect,
                        0.0,
                        egui::Color32::from_black_alpha(180)
                    );

                    ui.with_layout(
                        egui::Layout::centered_and_justified(egui::Direction::TopDown),
                        |ui| {
                            ui.vertical_centered(|ui| {
                                let center_y = screen_rect.height() * 0.45;
                                ui.add_space(center_y);

                                crate::ui::spinner::square_spinner(ui, 50.0, app.primary_color());

                                ui.add_space(10.0);

                                ui.label(
                                    egui::RichText::new("Loading...")
                                        .size(16.0)
                                        .color(egui::Color32::from_gray(200))
                                );
                            });
                        }
                    );
                });
            });

        ctx.request_repaint();
    }
}
