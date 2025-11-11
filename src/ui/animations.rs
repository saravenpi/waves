use crate::app::WavesApp;
use crate::config::AnimationType;
use eframe::egui;
use std::f32::consts::PI;

impl WavesApp {
    pub fn render_animation(&self, ui: &mut egui::Ui, rect: egui::Rect) {
        if !self.config.animation {
            return;
        }

        match self.config.animation_type {
            AnimationType::Spectrum => self.render_spectrum_animation(ui, rect),
            AnimationType::WaveformPulse => self.render_waveform_pulse_animation(ui, rect),
            AnimationType::CircleSpectrum => self.render_circle_spectrum_animation(ui, rect),
        }
    }

    fn render_spectrum_animation(&self, ui: &mut egui::Ui, rect: egui::Rect) {
        let painter = ui.painter();
        let bar_count = self.spectrum_bars.len();
        let bar_width = rect.width() / bar_count as f32;
        let primary_color = self.primary_color();

        for (i, &magnitude) in self.spectrum_bars.iter().enumerate() {
            let bar_height = magnitude * rect.height() * 0.8;
            let x = rect.min.x + i as f32 * bar_width;
            let y = rect.max.y - bar_height;

            let intensity = magnitude.clamp(0.0, 1.0);
            let color = egui::Color32::from_rgb(
                ((primary_color.r() as f32 * intensity) as u8).max(20),
                ((primary_color.g() as f32 * intensity) as u8).max(20),
                ((primary_color.b() as f32 * intensity) as u8).max(20),
            );

            painter.rect_filled(
                egui::Rect::from_min_size(
                    egui::pos2(x, y),
                    egui::vec2(bar_width * 0.8, bar_height),
                ),
                0.0,
                color,
            );
        }
    }

    fn render_waveform_pulse_animation(&self, ui: &mut egui::Ui, rect: egui::Rect) {
        let painter = ui.painter();
        let primary_color = self.primary_color();

        let center = rect.center();
        let avg_magnitude: f32 = self.spectrum_bars.iter().take(16).sum::<f32>() / 16.0;

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

        let max_width = rect.width() * 0.45;
        let max_height = rect.height() * 0.45;
        let max_safe_radius = max_width.min(max_height);

        let pulse_scale = 1.0 + (avg_magnitude * 0.4);

        let ring_count = 6;
        for ring in 0..ring_count {
            let ring_offset = ring as f32 * 0.15;
            let phase = time * 2.0 + ring_offset * PI;
            let ring_progress = ring as f32 / ring_count as f32;
            let base_radius = max_safe_radius * 0.3 * (1.0 + ring_progress * 0.8);
            let animated_radius = base_radius * pulse_scale * (1.0 + (phase.sin() * 0.1));

            let alpha = (255.0 * (1.0 - ring_progress * 0.3) * avg_magnitude.max(0.3)) as u8;
            let color = egui::Color32::from_rgba_unmultiplied(
                primary_color.r(),
                primary_color.g(),
                primary_color.b(),
                alpha.min(220),
            );

            painter.circle_stroke(
                center,
                animated_radius,
                egui::Stroke::new(3.0, color),
            );
        }

        let sample_count = 64;
        let outer_base_radius = max_safe_radius * 0.65;
        let outer_max_extension = max_safe_radius * 0.35;

        let points: Vec<egui::Pos2> = (0..sample_count)
            .map(|i| {
                let angle = (i as f32 / sample_count as f32) * 2.0 * PI;
                let bar_index = (i * self.spectrum_bars.len() / sample_count).min(self.spectrum_bars.len() - 1);
                let magnitude = self.spectrum_bars[bar_index];
                let radius = outer_base_radius + magnitude * outer_max_extension;

                egui::pos2(
                    center.x + angle.cos() * radius,
                    center.y + angle.sin() * radius,
                )
            })
            .collect();

        if points.len() > 1 {
            painter.add(egui::Shape::closed_line(
                points,
                egui::Stroke::new(3.0, primary_color),
            ));
        }
    }

    fn render_circle_spectrum_animation(&self, ui: &mut egui::Ui, rect: egui::Rect) {
        let painter = ui.painter();
        let primary_color = self.primary_color();

        let center = rect.center();
        let bar_count = self.spectrum_bars.len();
        let angle_step = 2.0 * PI / bar_count as f32;

        let max_width = rect.width() * 0.45;
        let max_height = rect.height() * 0.45;
        let max_safe_radius = max_width.min(max_height);

        let inner_radius = max_safe_radius * 0.35;
        let max_bar_length = max_safe_radius * 0.65;

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
        let rotation = time * 0.5;

        for (i, &magnitude) in self.spectrum_bars.iter().enumerate() {
            let angle = i as f32 * angle_step + rotation;
            let bar_length = magnitude * max_bar_length;

            let start_x = center.x + angle.cos() * inner_radius;
            let start_y = center.y + angle.sin() * inner_radius;
            let end_x = center.x + angle.cos() * (inner_radius + bar_length);
            let end_y = center.y + angle.sin() * (inner_radius + bar_length);

            let intensity = magnitude.clamp(0.0, 1.0);
            let color = egui::Color32::from_rgb(
                ((primary_color.r() as f32 * (0.3 + intensity * 0.7)) as u8).max(30),
                ((primary_color.g() as f32 * (0.3 + intensity * 0.7)) as u8).max(30),
                ((primary_color.b() as f32 * (0.3 + intensity * 0.7)) as u8).max(30),
            );

            painter.line_segment(
                [egui::pos2(start_x, start_y), egui::pos2(end_x, end_y)],
                egui::Stroke::new(12.0, color),
            );
        }

        painter.circle_stroke(
            center,
            inner_radius,
            egui::Stroke::new(2.0, egui::Color32::from_gray(80)),
        );
    }
}
