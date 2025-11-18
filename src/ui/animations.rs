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
            AnimationType::Agbe => self.render_agbe_animation(ui, rect),
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

    fn render_agbe_animation(&self, ui: &mut egui::Ui, rect: egui::Rect) {
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

        // Layer 1: Pulsating gradient rings with frequency-specific reactivity
        let ring_count = 100;
        for ring in 0..ring_count {
            let ring_progress = ring as f32 / ring_count as f32;

            // Assign different frequency ranges to different rings
            // Distribute 100 rings evenly across the full spectrum (25 per frequency range)
            let ring_magnitude = match ring {
                // Rings 0-24: Sub-bass/Kick (0-4 bars) - 25 rings
                0..=24 => {
                    let sub_bass: f32 = self.spectrum_bars.iter().take(4).sum::<f32>() / 4.0;
                    sub_bass
                }
                // Rings 25-49: Bass (4-12 bars) - 25 rings
                25..=49 => {
                    let bass: f32 = self.spectrum_bars.iter().skip(4).take(8).sum::<f32>() / 8.0;
                    bass
                }
                // Rings 50-74: Mids/Vocals (16-32 bars) - 25 rings
                50..=74 => {
                    let mids: f32 = self.spectrum_bars.iter().skip(16).take(16).sum::<f32>() / 16.0;
                    mids
                }
                // Rings 75-99: Treble/Hi-hats (48-64 bars) - 25 rings
                _ => {
                    let treble: f32 = self.spectrum_bars.iter().skip(48).take(16).sum::<f32>() / 16.0;
                    treble
                }
            };

            let base_radius = max_radius * (0.2 + ring_progress * 0.7);
            // Rings expand from 0 (center) when magnitude is 0, to full size at magnitude 1.0
            let radius = base_radius * ring_magnitude * 2.0; // 0.0 to 2.0x expansion

            // Create warped circle using audio-reactive deformation
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
            // Pass magnitude for dynamic color intensity
            let color = self.gradient_color(primary_color, hue_shift, ring_magnitude, alpha);

            if points.len() > 1 {
                painter.add(egui::Shape::closed_line(
                    points,
                    egui::Stroke::new(2.5, color),
                ));
            }
        }

    }

    /// Helper function to create gradient colors based on primary color with magnitude-driven dynamics
    fn gradient_color(&self, base_color: egui::Color32, hue_shift: f32, magnitude: f32, alpha: u8) -> egui::Color32 {
        // Extract base color components
        let base_r = base_color.r() as f32 / 255.0;
        let base_g = base_color.g() as f32 / 255.0;
        let base_b = base_color.b() as f32 / 255.0;

        // Create dramatic color dynamics based on magnitude
        // When magnitude is 0: very dark (20% brightness)
        // When magnitude is 1.0: very bright (up to 150% brightness with boost)
        let base_brightness = 0.2 + magnitude * 0.8; // Range: 0.2 to 1.0

        // Add hue_shift for subtle variation across rings
        let brightness_variation = 0.8 + hue_shift * 0.4; // Range: 0.8 to 1.2

        // Boost bright colors even more for dramatic effect
        let brightness_boost = if magnitude > 0.5 { 1.0 + (magnitude - 0.5) * 1.0 } else { 1.0 };

        let final_intensity = (base_brightness * brightness_variation * brightness_boost).min(1.5);

        // Apply intensity to base color to create gradient
        let final_r = (base_r * final_intensity * 255.0).min(255.0) as u8;
        let final_g = (base_g * final_intensity * 255.0).min(255.0) as u8;
        let final_b = (base_b * final_intensity * 255.0).min(255.0) as u8;

        egui::Color32::from_rgba_unmultiplied(final_r, final_g, final_b, alpha)
    }
}
