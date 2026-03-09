use eframe::egui;
use std::time::Duration;

use crate::app::WavesApp;
use crate::types::Liked;
use crate::ui::components::IconButton;
use crate::utils::format_duration;

pub fn render_playback_controls(
    app: &mut WavesApp,
    ui: &mut egui::Ui,
    ctx: &egui::Context,
    is_paused: bool,
) {
    let (has_player, title, artist, duration) = {
        let player = app.player.lock().unwrap();
        match player.as_ref() {
            Some(state) => (true, state.title.clone(), state.artist.clone(), state.duration),
            None => (false, String::new(), None, Duration::from_secs(0)),
        }
    };

    if has_player {
        let total_height = ui.available_height();
        let bottom_panel_height = 200.0_f32.min(total_height * 0.35);
        let separator_space = 20.0;
        let spectrum_height = (total_height - bottom_panel_height - separator_space).max(100.0);

        if app.config.animation {
            let (spectrum_rect, spectrum_response) = ui.allocate_exact_size(
                egui::vec2(ui.available_width(), spectrum_height),
                egui::Sense::hover()
            );

            app.render_animation(ui, spectrum_rect);

            if spectrum_response.hovered() {
                app.last_animation_hover = std::time::Instant::now();
            }

            let hover_elapsed = app.last_animation_hover.elapsed().as_secs_f32();
            let fade_duration = 2.0;
            let alpha = if hover_elapsed < fade_duration {
                (1.0 - (hover_elapsed / fade_duration)).clamp(0.0, 1.0)
            } else {
                0.0
            };

            if alpha > 0.01 {
                let button_size = egui::vec2(40.0, 40.0);
                let button_pos = egui::pos2(
                    spectrum_rect.max.x - button_size.x - 10.0,
                    spectrum_rect.max.y - button_size.y - 10.0,
                );
                let button_rect = egui::Rect::from_min_size(button_pos, button_size);

                let button_response = ui.interact(button_rect, ui.id().with("fullscreen_btn"), egui::Sense::click());

                let icon_alpha = (alpha * 255.0) as u8;

                if button_response.hovered() {
                    ui.painter().rect_stroke(
                        button_rect,
                        0.0,
                        egui::Stroke::new(1.5, egui::Color32::from_rgba_unmultiplied(255, 255, 255, icon_alpha)),
                    );
                }

                ui.painter().text(
                    button_rect.center(),
                    egui::Align2::CENTER_CENTER,
                    "⛶",
                    egui::FontId::proportional(24.0),
                    egui::Color32::from_rgba_unmultiplied(255, 255, 255, icon_alpha),
                );

                if button_response.clicked() {
                    app.animation_fullscreen = true;
                }

                ctx.request_repaint();
            }
        } else {
            ui.add_space(spectrum_height);
        }

        ui.add_space(10.0);
        ui.separator();
        ui.add_space(10.0);

        ui.horizontal(|ui| {
            let available_width = ui.available_width();
            let cover_size = 140.0_f32.min(available_width * 0.2).max(100.0);
            let volume_slider_width = 40.0;
            let total_padding = 60.0;

            let content_width = available_width - cover_size - volume_slider_width - total_padding;
            let min_content_width = 300.0;

            ui.add_space(20.0);

            {
                let player = app.player.lock().unwrap();
                if let Some(state) = player.as_ref() {
                    render_album_cover(ui, &state.album_cover, cover_size);
                }
            }

            ui.add_space(20.0);

            ui.vertical(|ui| {
                ui.set_width(content_width.max(min_content_width));

                ui.horizontal(|ui| {
                    ui.vertical_centered(|ui| {
                        ui.add_space(8.0);
                        render_like_button(app, ui);
                    });

                    ui.add_space(15.0);

                    let buttons_width = 190.0;
                    let title_width = ui.available_width() - buttons_width - 10.0;

                    ui.vertical(|ui| {
                        ui.set_width(title_width.max(200.0));
                        ui.set_height(50.0);

                        ui.label(
                            egui::RichText::new(&title)
                                .size(24.0)
                                .color(egui::Color32::WHITE)
                        );

                        if let Some(artist_name) = &artist {
                            ui.label(
                                egui::RichText::new(artist_name)
                                    .size(18.0)
                                    .color(egui::Color32::from_rgb(180, 180, 180))
                            );
                        }
                    });

                    ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
                        ui.vertical_centered(|ui| {
                            ui.add_space(10.0);
                            render_playback_buttons(app, ui, is_paused, ctx);
                        });
                    });
                });

                ui.add_space(10.0);

                {
                    let waveform = {
                        let player = app.player.lock().unwrap();
                        player.as_ref().map(|state| state.waveform.clone())
                    };
                    if let Some(waveform_data) = waveform {
                        render_waveform(app, ui, &waveform_data, duration, ctx);
                    }
                }

                ui.add_space(10.0);

                render_time_display(app, ui, duration);

                ui.add_space(5.0);
            });

            ui.add_space(10.0);

            render_volume_slider(app, ui);

            ui.add_space(20.0);
        });
    } else {
        ui.centered_and_justified(|ui| {
            ui.label(
                egui::RichText::new("No track playing")
                    .size(32.0)
                    .color(egui::Color32::from_rgb(100, 100, 100))
            );
        });
    }
}

fn render_album_cover(ui: &mut egui::Ui, album_cover: &Option<egui::TextureHandle>, size: f32) {
    let (rect, _) = ui.allocate_exact_size(
        egui::vec2(size, size),
        egui::Sense::hover()
    );

    if let Some(texture) = album_cover {
        ui.painter().image(
            texture.id(),
            rect,
            egui::Rect::from_min_max(egui::pos2(0.0, 0.0), egui::pos2(1.0, 1.0)),
            egui::Color32::WHITE
        );
    } else {
        ui.painter().rect_stroke(
            rect,
            0.0,
            egui::Stroke::new(2.0, egui::Color32::from_rgb(100, 100, 100))
        );

        ui.painter().text(
            rect.center(),
            egui::Align2::CENTER_CENTER,
            "No cover",
            egui::FontId::proportional(14.0),
            egui::Color32::from_rgb(120, 120, 120)
        );
    }
}

fn render_playback_buttons(app: &mut WavesApp, ui: &mut egui::Ui, is_paused: bool, ctx: &egui::Context) {
    ui.horizontal(|ui| {
        render_loop_button(app, ui);
        ui.add_space(6.0);
        render_next_button(app, ui, ctx);
        ui.add_space(6.0);
        render_play_pause_button(app, ui, is_paused);
        ui.add_space(6.0);
        render_previous_button(app, ui, ctx);
    });
}

fn render_next_button(app: &mut WavesApp, ui: &mut egui::Ui, ctx: &egui::Context) {
    let next_response = IconButton::new("⏭").show(ui);
    next_response.surrender_focus();

    if next_response.hovered() {
        let rect = next_response.rect;
        ui.painter().rect_stroke(
            rect,
            0.0,
            egui::Stroke::new(1.0, egui::Color32::from_rgb(64, 64, 64))
        );
    }

    if next_response.is_pointer_button_down_on() {
        let rect = next_response.rect;
        ui.painter().rect_filled(rect, 0.0, egui::Color32::WHITE);
        ui.painter().text(
            rect.center(),
            egui::Align2::CENTER_CENTER,
            "⏭",
            egui::FontId::proportional(28.0),
            egui::Color32::BLACK,
        );
    }

    if next_response.clicked() {
        app.play_next_song(ctx);
    }
}

fn render_play_pause_button(app: &mut WavesApp, ui: &mut egui::Ui, is_paused: bool) {
    let pause_play_text = if is_paused { "▶" } else { "⏸" };
    let play_pause_response = IconButton::new(pause_play_text).show(ui);
    play_pause_response.surrender_focus();

    if play_pause_response.hovered() {
        let rect = play_pause_response.rect;
        ui.painter().rect_stroke(
            rect,
            0.0,
            egui::Stroke::new(1.0, egui::Color32::from_rgb(64, 64, 64))
        );
    }

    if play_pause_response.is_pointer_button_down_on() {
        let rect = play_pause_response.rect;
        ui.painter().rect_filled(rect, 0.0, egui::Color32::WHITE);
        ui.painter().text(
            rect.center(),
            egui::Align2::CENTER_CENTER,
            pause_play_text,
            egui::FontId::proportional(28.0),
            egui::Color32::BLACK,
        );
    }

    if play_pause_response.clicked() {
        app.toggle_pause();
    }
}

fn render_loop_button(app: &mut WavesApp, ui: &mut egui::Ui) {
    let loop_color = if app.loop_enabled {
        app.primary_color()
    } else {
        egui::Color32::WHITE
    };

    let loop_response = IconButton::new("🔁")
        .size(24.0)
        .color(loop_color)
        .show(ui);
    loop_response.surrender_focus();

    if loop_response.hovered() {
        let rect = loop_response.rect;
        ui.painter().rect_stroke(rect, 0.0, egui::Stroke::new(1.0, loop_color));
    }

    if loop_response.is_pointer_button_down_on() {
        let rect = loop_response.rect;
        ui.painter().rect_filled(rect, 0.0, loop_color);
        ui.painter().text(
            rect.center(),
            egui::Align2::CENTER_CENTER,
            "🔁",
            egui::FontId::proportional(24.0),
            egui::Color32::BLACK,
        );
    }

    if loop_response.clicked() {
        app.loop_enabled = !app.loop_enabled;
    }
}

fn render_previous_button(app: &mut WavesApp, ui: &mut egui::Ui, ctx: &egui::Context) {
    let prev_response = IconButton::new("⏮").show(ui);
    prev_response.surrender_focus();

    if prev_response.hovered() {
        let rect = prev_response.rect;
        ui.painter().rect_stroke(
            rect,
            0.0,
            egui::Stroke::new(1.0, egui::Color32::from_rgb(64, 64, 64))
        );
    }

    if prev_response.is_pointer_button_down_on() {
        let rect = prev_response.rect;
        ui.painter().rect_filled(rect, 0.0, egui::Color32::WHITE);
        ui.painter().text(
            rect.center(),
            egui::Align2::CENTER_CENTER,
            "⏮",
            egui::FontId::proportional(28.0),
            egui::Color32::BLACK,
        );
    }

    if prev_response.clicked() {
        app.play_previous_song(ctx);
    }
}

fn render_like_button(app: &mut WavesApp, ui: &mut egui::Ui) {
    let current_file = app.player.lock().unwrap()
        .as_ref()
        .map(|state| state.current_file.clone());

    if let Some(ref file_path) = current_file {
        let is_current_liked = app.liked.iter().any(|f| f.path == *file_path);
        let like_color = if is_current_liked {
            app.primary_color()
        } else {
            egui::Color32::WHITE
        };
        let like_icon = if is_current_liked { "❤" } else { "♡" };

        let like_response = IconButton::new(like_icon)
            .size(24.0)
            .color(like_color)
            .show(ui);
        like_response.surrender_focus();

        if like_response.hovered() {
            let rect = like_response.rect;
            ui.painter().rect_stroke(rect, 0.0, egui::Stroke::new(1.0, like_color));
        }

        if like_response.is_pointer_button_down_on() {
            let rect = like_response.rect;
            ui.painter().rect_filled(rect, 0.0, like_color);
            ui.painter().text(
                rect.center(),
                egui::Align2::CENTER_CENTER,
                like_icon,
                egui::FontId::proportional(24.0),
                egui::Color32::BLACK,
            );
        }

        if like_response.clicked() {
            if is_current_liked {
                app.liked.retain(|f| f.path != *file_path);
            } else {
                let name = file_path
                    .file_name()
                    .unwrap_or_default()
                    .to_string_lossy()
                    .to_string();
                app.liked.insert(0, Liked {
                    path: file_path.clone(),
                    name,
                    is_dir: false,
                    timestamp: std::time::SystemTime::now(),
                });
            }
            crate::liked::save(&app.liked);
        }
    }
}

fn render_waveform(
    app: &mut WavesApp,
    ui: &mut egui::Ui,
    waveform: &[f32],
    duration: Duration,
    ctx: &egui::Context,
) {
    let waveform_width = ui.available_width().max(200.0);
    let waveform_height = 60.0;

    let (rect, response) = ui.allocate_exact_size(
        egui::vec2(waveform_width, waveform_height),
        egui::Sense::click_and_drag()
    );

    if response.dragged() || response.is_pointer_button_down_on() {
        if let Some(pos) = response.interact_pointer_pos() {
            let click_x = (pos.x - rect.min.x).max(0.0).min(rect.width());
            let progress = (click_x / rect.width()).clamp(0.0, 1.0);
            app.pending_seek = Some(progress);
            ctx.request_repaint();
        }
    } else if response.drag_stopped() || response.clicked() {
        if let Some(pending) = app.pending_seek {
            app.seek_to_position(pending);
            app.pending_seek = None;
        }
    }

    let painter = ui.painter();

    let skip_factor = 3;
    let visible_samples: Vec<_> = waveform.iter().enumerate()
        .filter(|(i, _)| i % skip_factor == 0)
        .map(|(_, &val)| val)
        .collect();

    let bar_width = rect.width() / visible_samples.len() as f32;
    let max_height = rect.height() * 0.9;

    let current_pos = app.get_current_position().unwrap_or(Duration::from_secs(0));
    let progress = if let Some(pending) = app.pending_seek {
        pending
    } else if duration.as_secs() > 0 {
        current_pos.as_secs_f32() / duration.as_secs_f32()
    } else {
        0.0
    };

    for (i, &amplitude) in visible_samples.iter().enumerate() {
        let x = rect.min.x + (i as f32 * bar_width);
        let adjusted_amplitude = (amplitude * 0.5).min(1.0);
        let height = adjusted_amplitude * max_height;
        let y_bottom = rect.max.y;
        let y_top = y_bottom - height;

        let bar_progress = (i * skip_factor) as f32 / waveform.len() as f32;
        let color = if bar_progress <= progress {
            app.primary_color()
        } else {
            egui::Color32::from_rgb(60, 60, 60)
        };

        painter.line_segment(
            [egui::pos2(x, y_top), egui::pos2(x, y_bottom)],
            egui::Stroke::new(bar_width * 0.4, color),
        );
    }

    let progress_x = rect.min.x + progress * rect.width();
    let marker_color = if app.pending_seek.is_some() {
        egui::Color32::from_rgb(255, 200, 100)
    } else {
        egui::Color32::WHITE
    };
    painter.vline(
        progress_x,
        rect.min.y..=rect.max.y,
        egui::Stroke::new(2.0, marker_color),
    );
}

fn render_time_display(app: &WavesApp, ui: &mut egui::Ui, duration: Duration) {
    ui.horizontal(|ui| {
        let display_pos = if let Some(pending) = app.pending_seek {
            Duration::from_secs_f32(duration.as_secs_f32() * pending)
        } else {
            app.get_current_position().unwrap_or(Duration::from_secs(0))
        };

        let time_color = if app.pending_seek.is_some() {
            egui::Color32::from_rgb(255, 200, 100)
        } else {
            egui::Color32::WHITE
        };

        ui.label(
            egui::RichText::new(format_duration(display_pos))
                .size(16.0)
                .color(time_color)
                .monospace()
        );

        ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
            ui.label(
                egui::RichText::new(format_duration(duration))
                    .size(16.0)
                    .color(egui::Color32::from_rgb(150, 150, 150))
                    .monospace()
            );
        });
    });
}

fn render_volume_slider(app: &mut WavesApp, ui: &mut egui::Ui) {
    ui.vertical_centered(|ui| {
        let slider_width = 6.0;
        let slider_height = 120.0;

        let (rect, response) = ui.allocate_exact_size(
            egui::vec2(slider_width, slider_height),
            egui::Sense::click_and_drag()
        );

        let painter = ui.painter();

        painter.rect_filled(
            rect,
            0.0,
            egui::Color32::from_rgb(40, 40, 40),
        );

        let fill_height = slider_height * app.volume;
        let fill_rect = egui::Rect::from_min_size(
            egui::pos2(rect.min.x, rect.max.y - fill_height),
            egui::vec2(slider_width, fill_height),
        );
        painter.rect_filled(
            fill_rect,
            0.0,
            app.primary_color(),
        );

        if response.dragged() || response.clicked() {
            if let Some(pos) = response.interact_pointer_pos() {
                let relative_y = (rect.max.y - pos.y).max(0.0).min(slider_height);
                let new_volume = (relative_y / slider_height).clamp(0.0, 1.0);
                app.volume = new_volume;
                if let Ok(player) = app.player.lock() {
                    if let Some(state) = player.as_ref() {
                        state.sink.set_volume(app.volume);
                    }
                }
            }
        }

        ui.add_space(8.0);

        ui.label(
            egui::RichText::new(format!("{:.0}", app.volume * 100.0))
                .size(12.0)
                .color(egui::Color32::WHITE)
        );
    });
}
