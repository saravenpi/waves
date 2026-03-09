use crate::app::WavesApp;
use eframe::egui;
use std::f32::consts::PI;

impl WavesApp {
    pub fn render_agbe_animation(&self, ui: &mut egui::Ui, rect: egui::Rect) {
        let painter = ui.painter();
        let primary_color = self.primary_color();
        let center = rect.center();

        let is_playing = if let Ok(player) = self.player.lock() {
            if let Some(state) = player.as_ref() {
                !state.sink.is_paused() && !state.sink.empty()
            } else {
                false
            }
        } else {
            false
        };

        let time = if is_playing {
            ui.input(|i| i.time) as f32
        } else {
            0.0
        };

        let max_width = rect.width() * 0.35;
        let max_height = rect.height() * 0.35;
        let max_radius = max_width.min(max_height);

        let mid_magnitude: f32 = self.spectrum_bars.iter().skip(16).take(16).sum::<f32>() / 16.0;


        let ring_count = 100;
        for ring in 0..ring_count {
            let ring_progress = ring as f32 / ring_count as f32;



            let ring_magnitude = match ring {

                0..=24 => {
                    let sub_bass: f32 = self.spectrum_bars.iter().take(4).sum::<f32>() / 4.0;
                    sub_bass
                }

                25..=49 => {
                    let bass: f32 = self.spectrum_bars.iter().skip(4).take(8).sum::<f32>() / 8.0;
                    bass
                }

                50..=74 => {
                    let mids: f32 = self.spectrum_bars.iter().skip(16).take(16).sum::<f32>() / 16.0;
                    mids
                }

                _ => {
                    let treble: f32 = self.spectrum_bars.iter().skip(48).take(16).sum::<f32>() / 16.0;
                    treble
                }
            };

            let base_radius = max_radius * (0.2 + ring_progress * 0.7);

            let radius = base_radius * ring_magnitude * 2.0;


            let segments = 64;
            let points: Vec<egui::Pos2> = (0..segments)
                .map(|i| {
                    let angle = (i as f32 / segments as f32) * 2.0 * PI;
                    let bar_index = (i * self.spectrum_bars.len() / segments).min(self.spectrum_bars.len() - 1);
                    let deformation = self.spectrum_bars[bar_index] * 30.0;

                    let wobble = (angle * 3.0 + time * 2.0).sin() * 15.0 * mid_magnitude;
                    let final_radius = radius + deformation + wobble;

                    egui::pos2(
                        center.x + angle.cos() * final_radius,
                        center.y + angle.sin() * final_radius,
                    )
                })
                .collect();

            let hue_shift = (time * 0.2 + ring_progress) % 1.0;
            let alpha = (200.0 * (1.0 - ring_progress * 0.5)) as u8;

            let color = self.gradient_color(primary_color, hue_shift, ring_magnitude, alpha);

            if points.len() > 1 {
                painter.add(egui::Shape::closed_line(
                    points,
                    egui::Stroke::new(2.5, color),
                ));
            }
        }

    }

    pub fn gradient_color(&self, base_color: egui::Color32, hue_shift: f32, magnitude: f32, alpha: u8) -> egui::Color32 {

        let base_r = base_color.r() as f32 / 255.0;
        let base_g = base_color.g() as f32 / 255.0;
        let base_b = base_color.b() as f32 / 255.0;




        let base_brightness = 0.2 + magnitude * 0.8;


        let brightness_variation = 0.8 + hue_shift * 0.4;


        let brightness_boost = if magnitude > 0.5 { 1.0 + (magnitude - 0.5) * 1.0 } else { 1.0 };

        let final_intensity = (base_brightness * brightness_variation * brightness_boost).min(1.5);


        let final_r = (base_r * final_intensity * 255.0).min(255.0) as u8;
        let final_g = (base_g * final_intensity * 255.0).min(255.0) as u8;
        let final_b = (base_b * final_intensity * 255.0).min(255.0) as u8;

        egui::Color32::from_rgba_unmultiplied(final_r, final_g, final_b, alpha)
    }
}
