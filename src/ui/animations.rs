use crate::app::WavesApp;
use crate::config::AnimationType;
use eframe::egui;
use std::f32::consts::PI;

impl WavesApp {
    pub fn render_animation(&mut self, ui: &mut egui::Ui, rect: egui::Rect) {
        if !self.config.animation {
            return;
        }

        match self.config.animation_type {
            AnimationType::Spectrum => self.render_spectrum_animation(ui, rect),
            AnimationType::WaveformPulse => self.render_waveform_pulse_animation(ui, rect),
            AnimationType::CircleSpectrum => self.render_circle_spectrum_animation(ui, rect),
            AnimationType::Agbe => self.render_agbe_animation(ui, rect),
            AnimationType::Dots => self.render_dots_animation(ui, rect),
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

    fn render_dots_animation(&mut self, ui: &mut egui::Ui, rect: egui::Rect) {
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

    fn gradient_color(&self, base_color: egui::Color32, hue_shift: f32, magnitude: f32, alpha: u8) -> egui::Color32 {

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
