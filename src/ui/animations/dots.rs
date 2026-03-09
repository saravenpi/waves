use crate::app::WavesApp;
use eframe::egui;
use std::f32::consts::PI;

impl WavesApp {
    pub fn render_dots_animation(&mut self, ui: &mut egui::Ui, rect: egui::Rect) {
        let painter = ui.painter();
        let primary_color = self.primary_color();

        let is_playing = if let Ok(player) = self.player.lock() {
            if let Some(state) = player.as_ref() {
                !state.sink.is_paused() && !state.sink.empty()
            } else {
                false
            }
        } else {
            false
        };

        let dt = ui.input(|i| i.unstable_dt).min(0.1);

        if !self.dots_initialized || self.dots.is_empty() {
            self.dots.clear();
            let cols = 16;
            let rows = 10;
            let dot_count = cols * rows;

            let padding_x = rect.width() * 0.1;
            let padding_y = rect.height() * 0.1;
            let usable_width = rect.width() - 2.0 * padding_x;
            let usable_height = rect.height() - 2.0 * padding_y;

            let spacing_x = usable_width / (cols - 1) as f32;
            let spacing_y = usable_height / (rows - 1) as f32;

            use rand::Rng;
            let mut rng = rand::thread_rng();

            for i in 0..dot_count {
                let col = i % cols;
                let row = i / cols;
                let frequency_band = (i * 64) / dot_count;

                let random_offset_x = rng.gen_range(-spacing_x * 0.3..spacing_x * 0.3);
                let random_offset_y = rng.gen_range(-spacing_y * 0.3..spacing_y * 0.3);

                self.dots.push(crate::app::state::Dot {
                    x: rect.min.x + padding_x + col as f32 * spacing_x + random_offset_x,
                    y: rect.min.y + padding_y + row as f32 * spacing_y + random_offset_y,
                    vx: 0.0,
                    vy: 0.0,
                    frequency_band,
                });
            }
            self.dots_initialized = true;
        }

        let bounce_damping = 0.7;
        let connection_distance = 200.0;
        let base_attraction_strength = 800.0;
        let min_distance = 20.0;
        let center_pull_strength = 20.0;

        let avg_magnitude: f32 = self.spectrum_bars.iter().sum::<f32>() / self.spectrum_bars.len() as f32;
        let center_x = rect.min.x + rect.width() * 0.5;
        let center_y = rect.min.y + rect.height() * 0.5;

        let padding_x = rect.width() * 0.1;
        let padding_y = rect.height() * 0.1;
        let usable_width = rect.width() - 2.0 * padding_x;
        let usable_height = rect.height() - 2.0 * padding_y;
        let cols = 16;
        let _spacing_x = usable_width / (cols - 1) as f32;
        let _spacing_y = usable_height / 9.0;

        let mut forces_x = vec![0.0; self.dots.len()];
        let mut forces_y = vec![0.0; self.dots.len()];

        for i in 0..self.dots.len() {
            let dx_from_center = center_x - self.dots[i].x;
            let dy_from_center = center_y - self.dots[i].y;
            let dist_from_center = (dx_from_center * dx_from_center + dy_from_center * dy_from_center).sqrt().max(1.0);

            let center_force = center_pull_strength * (dist_from_center / rect.width());
            forces_x[i] += (dx_from_center / dist_from_center) * center_force;
            forces_y[i] += (dy_from_center / dist_from_center) * center_force;

            for j in (i + 1)..self.dots.len() {
                let dx = self.dots[j].x - self.dots[i].x;
                let dy = self.dots[j].y - self.dots[i].y;
                let distance_sq = dx * dx + dy * dy;
                let distance = distance_sq.sqrt().max(min_distance);

                if distance < min_distance * 2.0 {
                    let repel_strength = 300.0 * (1.0 - distance / (min_distance * 2.0));
                    let repel_x = -(dx / distance) * repel_strength;
                    let repel_y = -(dy / distance) * repel_strength;
                    forces_x[i] += repel_x;
                    forces_y[i] += repel_y;
                    forces_x[j] -= repel_x;
                    forces_y[j] -= repel_y;
                }

                let mag_i = self.spectrum_bars[self.dots[i].frequency_band.min(self.spectrum_bars.len() - 1)];
                let mag_j = self.spectrum_bars[self.dots[j].frequency_band.min(self.spectrum_bars.len() - 1)];
                let combined_magnitude = (mag_i + mag_j) * 0.5;

                let attraction_strength = if is_playing {
                    base_attraction_strength * (1.0 + combined_magnitude * 0.5)
                } else {
                    base_attraction_strength
                };

                let force_magnitude = attraction_strength / distance_sq;
                let force_x = (dx / distance) * force_magnitude;
                let force_y = (dy / distance) * force_magnitude;

                forces_x[i] += force_x;
                forces_y[i] += force_y;
                forces_x[j] -= force_x;
                forces_y[j] -= force_y;
            }
        }

        for i in 0..self.dots.len() {
            if is_playing {
                let magnitude = self.spectrum_bars[self.dots[i].frequency_band.min(self.spectrum_bars.len() - 1)];

                let time = ui.input(|input| input.time) as f32;
                let oscillation_x = (time * 3.0 + self.dots[i].frequency_band as f32).sin() * magnitude * 800.0;
                let oscillation_y = (time * 2.5 + self.dots[i].frequency_band as f32).cos() * magnitude * 800.0;

                forces_x[i] += oscillation_x;
                forces_y[i] += oscillation_y;
            }

            self.dots[i].vx += forces_x[i] * dt;
            self.dots[i].vy += forces_y[i] * dt;

            let damping = 0.95;
            self.dots[i].vx *= damping;
            self.dots[i].vy *= damping;

            self.dots[i].x += self.dots[i].vx * dt;
            self.dots[i].y += self.dots[i].vy * dt;

            if self.dots[i].x < rect.min.x {
                self.dots[i].x = rect.min.x;
                self.dots[i].vx = -self.dots[i].vx * bounce_damping;
            } else if self.dots[i].x > rect.max.x {
                self.dots[i].x = rect.max.x;
                self.dots[i].vx = -self.dots[i].vx * bounce_damping;
            }

            if self.dots[i].y < rect.min.y {
                self.dots[i].y = rect.min.y;
                self.dots[i].vy = -self.dots[i].vy * bounce_damping;
            } else if self.dots[i].y > rect.max.y {
                self.dots[i].y = rect.max.y;
                self.dots[i].vy = -self.dots[i].vy * bounce_damping;
            }
        }

        let time = ui.input(|input| input.time) as f32;

        for i in 0..self.dots.len() {
            for j in (i + 1)..self.dots.len() {
                let dx = self.dots[j].x - self.dots[i].x;
                let dy = self.dots[j].y - self.dots[i].y;
                let distance = (dx * dx + dy * dy).sqrt();

                if distance < connection_distance {
                    let mag_i = self.spectrum_bars[self.dots[i].frequency_band.min(self.spectrum_bars.len() - 1)];
                    let mag_j = self.spectrum_bars[self.dots[j].frequency_band.min(self.spectrum_bars.len() - 1)];
                    let combined_magnitude = (mag_i + mag_j) * 0.5;

                    let num_segments = 8;
                    for seg in 0..num_segments {
                        let t_start = seg as f32 / num_segments as f32;
                        let t_end = (seg + 1) as f32 / num_segments as f32;
                        let t_mid = (t_start + t_end) * 0.5;

                        let pulse_i = (time * 4.0 + self.dots[i].frequency_band as f32 * 0.1).sin() * mag_i;
                        let pulse_j = (time * 4.0 + self.dots[j].frequency_band as f32 * 0.1).sin() * mag_j;

                        let wave_from_i = (PI * 2.0 * (t_mid - time * 2.0)).sin() * pulse_i;
                        let wave_from_j = (PI * 2.0 * ((1.0 - t_mid) - time * 2.0)).sin() * pulse_j;
                        let pulse_intensity = ((wave_from_i + wave_from_j) * 0.5).abs();

                        let segment_alpha = ((1.0 - distance / connection_distance) * 100.0 * (1.0 + avg_magnitude * 2.0 + combined_magnitude * 1.5 + pulse_intensity * 3.0)) as u8;
                        let segment_width = 1.5 + combined_magnitude * 2.0 + pulse_intensity * 3.0;
                        let segment_color = egui::Color32::from_rgba_unmultiplied(
                            primary_color.r(),
                            primary_color.g(),
                            primary_color.b(),
                            segment_alpha.min(255),
                        );

                        let start_x = self.dots[i].x + dx * t_start;
                        let start_y = self.dots[i].y + dy * t_start;
                        let end_x = self.dots[i].x + dx * t_end;
                        let end_y = self.dots[i].y + dy * t_end;

                        painter.line_segment(
                            [egui::pos2(start_x, start_y), egui::pos2(end_x, end_y)],
                            egui::Stroke::new(segment_width, segment_color),
                        );
                    }
                }
            }
        }

        for dot in &self.dots {
            let magnitude = self.spectrum_bars[dot.frequency_band.min(self.spectrum_bars.len() - 1)];
            let radius = 2.5 + magnitude * 8.0;

            let intensity = magnitude.clamp(0.3, 1.0);
            let dot_color = egui::Color32::from_rgb(
                ((primary_color.r() as f32 * intensity) as u8).max(50),
                ((primary_color.g() as f32 * intensity) as u8).max(50),
                ((primary_color.b() as f32 * intensity) as u8).max(50),
            );

            painter.circle_filled(egui::pos2(dot.x, dot.y), radius, dot_color);

            let glow_alpha = (magnitude * 80.0) as u8;
            let glow_color = egui::Color32::from_rgba_unmultiplied(
                primary_color.r(),
                primary_color.g(),
                primary_color.b(),
                glow_alpha,
            );
            painter.circle_filled(egui::pos2(dot.x, dot.y), radius * 1.5, glow_color);
        }
    }
}
