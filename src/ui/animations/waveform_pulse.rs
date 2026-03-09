use crate::app::WavesApp;
use eframe::egui;
use std::f32::consts::PI;

impl WavesApp {
    pub fn render_waveform_pulse_animation(&self, ui: &mut egui::Ui, rect: egui::Rect) {
        let painter = ui.painter();
        let primary_color = self.primary_color();

        let center = rect.center();

        let max_width = rect.width() * 0.45;
        let max_height = rect.height() * 0.45;
        let max_safe_radius = max_width.min(max_height);

        let sample_count = 64;
        let ring_count = 100;

        for ring in 0..ring_count {
            let ring_progress = ring as f32 / ring_count as f32;
            let base_radius = max_safe_radius * 0.3 + (max_safe_radius * 0.7 * ring_progress);
            let max_extension = max_safe_radius * 0.15;

            let points: Vec<egui::Pos2> = (0..sample_count)
                .map(|i| {
                    let angle = (i as f32 / sample_count as f32) * 2.0 * PI;
                    let bar_index = (i * self.spectrum_bars.len() / sample_count).min(self.spectrum_bars.len() - 1);
                    let magnitude = self.spectrum_bars[bar_index];
                    let radius = base_radius + magnitude * max_extension;

                    egui::pos2(
                        center.x + angle.cos() * radius,
                        center.y + angle.sin() * radius,
                    )
                })
                .collect();

            let alpha = (180.0 * (1.0 - ring_progress * 0.5)) as u8;
            let color = egui::Color32::from_rgba_unmultiplied(
                primary_color.r(),
                primary_color.g(),
                primary_color.b(),
                alpha,
            );

            if points.len() > 1 {
                painter.add(egui::Shape::closed_line(
                    points,
                    egui::Stroke::new(2.0, color),
                ));
            }
        }
    }
}
