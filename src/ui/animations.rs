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

        let max_width = rect.width() * 0.45;
        let max_height = rect.height() * 0.45;
        let max_radius = max_width.min(max_height);

        let avg_magnitude: f32 = self.spectrum_bars.iter().take(32).sum::<f32>() / 32.0;
        let mid_magnitude: f32 = self.spectrum_bars.iter().skip(16).take(16).sum::<f32>() / 16.0;

        // Layer 1: Pulsating gradient rings with frequency-specific reactivity
        let ring_count = 16;
        for ring in 0..ring_count {
            let ring_progress = ring as f32 / ring_count as f32;

            // Assign different frequency ranges to different rings
            // Distribute 16 rings across the full spectrum
            let ring_magnitude = match ring {
                // Rings 0-3: Sub-bass/Kick (0-4 bars)
                0..=3 => {
                    let sub_bass: f32 = self.spectrum_bars.iter().take(4).sum::<f32>() / 4.0;
                    sub_bass
                }
                // Rings 4-7: Bass (4-12 bars)
                4..=7 => {
                    let bass: f32 = self.spectrum_bars.iter().skip(4).take(8).sum::<f32>() / 8.0;
                    bass
                }
                // Rings 8-11: Mids/Vocals (16-32 bars)
                8..=11 => {
                    let mids: f32 = self.spectrum_bars.iter().skip(16).take(16).sum::<f32>() / 16.0;
                    mids
                }
                // Rings 12-15: Treble/Hi-hats (48-64 bars)
                _ => {
                    let treble: f32 = self.spectrum_bars.iter().skip(48).take(16).sum::<f32>() / 16.0;
                    treble
                }
            };

            let base_radius = max_radius * (0.2 + ring_progress * 0.7);
            // Much stronger pulsating effect: rings expand outward based on frequency magnitude
            let pulse = 1.0 + ring_magnitude * 0.8; // Direct frequency-based expansion (0.0 to 1.8x)
            let radius = base_radius * pulse;

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
            let color = self.gradient_color(primary_color, hue_shift, 0.7 + ring_magnitude * 0.3, alpha);

            if points.len() > 1 {
                painter.add(egui::Shape::closed_line(
                    points,
                    egui::Stroke::new(2.5, color),
                ));
            }
        }

        // Layer 2: Radiating energy waves
        let wave_count = 20;
        for wave in 0..wave_count {
            let wave_progress = (time * 2.0 + wave as f32 * 0.3) % 1.0;
            // Start waves from 20% radius to avoid center convergence
            let wave_radius = max_radius * (0.2 + wave_progress * 0.8);

            let segments = 48;
            let points: Vec<egui::Pos2> = (0..segments)
                .map(|i| {
                    let angle = (i as f32 / segments as f32) * 2.0 * PI;
                    let bar_index = (i * self.spectrum_bars.len() / segments).min(self.spectrum_bars.len() - 1);
                    let magnitude = self.spectrum_bars[bar_index];

                    let ripple = (angle * 5.0 - time * 4.0).sin() * magnitude * 20.0;
                    let final_radius = wave_radius + ripple;

                    egui::pos2(
                        center.x + angle.cos() * final_radius,
                        center.y + angle.sin() * final_radius,
                    )
                })
                .collect();

            // Smooth alpha with high minimum to prevent blinking
            let fade = 1.0 - wave_progress;
            let smoothed_magnitude = avg_magnitude * 0.2 + 0.6; // Range: 0.6 to 0.8 (higher baseline)
            let alpha = (150.0 * fade * smoothed_magnitude) as u8;
            let final_alpha = alpha.max(30).min(100); // Ensure minimum visibility, cap maximum
            let color = self.gradient_color(primary_color, wave_progress, 0.9, final_alpha);

            if points.len() > 1 {
                painter.add(egui::Shape::closed_line(
                    points,
                    egui::Stroke::new(1.5, color),
                ));
            }
        }

    }

    /// Helper function to create vibrant gradient colors with full audio reactivity
    fn gradient_color(&self, _base_color: egui::Color32, hue_shift: f32, intensity: f32, alpha: u8) -> egui::Color32 {
        // Extract frequency-specific magnitudes for dynamic color shifting
        let bass_magnitude: f32 = self.spectrum_bars.iter().take(8).sum::<f32>() / 8.0;
        let mid_magnitude: f32 = self.spectrum_bars.iter().skip(16).take(16).sum::<f32>() / 16.0;
        let treble_magnitude: f32 = self.spectrum_bars.iter().skip(48).take(16).sum::<f32>() / 16.0;

        // Create full spectrum hue rotation (0.0 to 1.0 maps to full color wheel)
        let hue = (hue_shift + bass_magnitude * 0.1 + mid_magnitude * 0.05) % 1.0;

        // Convert HSV to RGB for vibrant colors
        let saturation = 0.8 + treble_magnitude * 0.2; // High saturation for vivid colors
        let value = intensity;

        // HSV to RGB conversion
        let h = hue * 6.0;
        let i = h.floor();
        let f = h - i;
        let p = value * (1.0 - saturation);
        let q = value * (1.0 - saturation * f);
        let t = value * (1.0 - saturation * (1.0 - f));

        let (r, g, b) = match i as i32 % 6 {
            0 => (value, t, p),
            1 => (q, value, p),
            2 => (p, value, t),
            3 => (p, q, value),
            4 => (t, p, value),
            _ => (value, p, q),
        };

        // Boost specific channels based on frequency content
        let final_r = ((r * 255.0) * (1.0 + bass_magnitude * 0.3)).min(255.0) as u8;
        let final_g = ((g * 255.0) * (1.0 + mid_magnitude * 0.3)).min(255.0) as u8;
        let final_b = ((b * 255.0) * (1.0 + treble_magnitude * 0.3)).min(255.0) as u8;

        egui::Color32::from_rgba_unmultiplied(final_r, final_g, final_b, alpha)
    }
}
