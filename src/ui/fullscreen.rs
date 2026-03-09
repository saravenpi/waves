use eframe::egui;
use crate::app::WavesApp;
use crate::types::SidebarView;

pub fn render_fullscreen_animation(app: &mut WavesApp, ctx: &egui::Context) {
    ctx.input(|i| {
        for event in &i.events {
            if let egui::Event::Key { key, pressed: true, .. } = event {
                match key {
                    egui::Key::Escape => {
                        app.animation_fullscreen = false;
                    }
                    egui::Key::Space => {
                        app.toggle_pause();
                    }
                    egui::Key::ArrowLeft => {
                        match app.playback_context {
                            SidebarView::Liked => app.play_previous_liked(ctx),
                            _ => app.play_previous_song(ctx),
                        }
                    }
                    egui::Key::ArrowRight => {
                        match app.playback_context {
                            SidebarView::Liked => app.play_next_liked(ctx),
                            _ => app.play_next_song(ctx),
                        }
                    }
                    egui::Key::ArrowUp => {
                        app.volume = (app.volume + 0.05).min(1.0);
                        if let Ok(player) = app.player.lock() {
                            if let Some(state) = player.as_ref() {
                                state.sink.set_volume(app.volume);
                            }
                        }
                    }
                    egui::Key::ArrowDown => {
                        app.volume = (app.volume - 0.05).max(0.0);
                        if let Ok(player) = app.player.lock() {
                            if let Some(state) = player.as_ref() {
                                state.sink.set_volume(app.volume);
                            }
                        }
                    }
                    _ => {}
                }
            }
        }
    });

    egui::CentralPanel::default()
        .frame(egui::Frame::none().fill(egui::Color32::from_rgb(8, 8, 8)))
        .show(ctx, |ui| {
            let fullscreen_rect = ui.max_rect();
            app.render_animation(ui, fullscreen_rect);

            let mouse_idle_duration = app.last_mouse_movement.elapsed().as_secs_f32();
            let fade_duration = 2.0;
            let alpha = if mouse_idle_duration < fade_duration {
                (1.0 - (mouse_idle_duration / fade_duration)).clamp(0.0, 1.0)
            } else {
                0.0
            };

            if alpha > 0.01 {
                let button_size = egui::vec2(50.0, 50.0);
                let button_pos = egui::pos2(
                    fullscreen_rect.max.x - button_size.x - 20.0,
                    fullscreen_rect.max.y - button_size.y - 20.0,
                );
                let button_rect = egui::Rect::from_min_size(button_pos, button_size);

                let button_id = egui::Id::new("exit_fullscreen_btn");
                let button_response = ui.interact(button_rect, button_id, egui::Sense::click());

                let icon_alpha = (alpha * 255.0) as u8;

                if button_response.hovered() {
                    ui.painter().rect_stroke(
                        button_rect,
                        0.0,
                        egui::Stroke::new(2.0, egui::Color32::from_rgba_unmultiplied(255, 255, 255, icon_alpha)),
                    );
                }

                ui.painter().text(
                    button_rect.center(),
                    egui::Align2::CENTER_CENTER,
                    "×",
                    egui::FontId::proportional(32.0),
                    egui::Color32::from_rgba_unmultiplied(255, 255, 255, icon_alpha),
                );

                if button_response.clicked() {
                    app.animation_fullscreen = false;
                }
            }
        });

    ctx.request_repaint();
}
