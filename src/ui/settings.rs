use eframe::egui;
use crate::app::state::WavesApp;

pub fn render_settings(app: &mut WavesApp, ui: &mut egui::Ui, list_height: f32) {
    ui.add_space(10.0);
    ui.heading(egui::RichText::new("Settings").size(20.0).color(egui::Color32::WHITE));
    ui.add_space(20.0);

    egui::ScrollArea::vertical()
        .auto_shrink([false, false])
        .min_scrolled_height(list_height)
        .max_height(list_height)
        .id_salt("settings_scroll")
        .show(ui, |ui| {
            ui.add_space(10.0);

            render_primary_color_setting(app, ui);
            ui.add_space(10.0);
            ui.separator();
            ui.add_space(10.0);

            render_show_status_bar_toggle(app, ui);
            ui.add_space(10.0);

            render_show_title_bar_toggle(app, ui);
            ui.add_space(10.0);

            render_visual_animation_toggle(app, ui);
            ui.add_space(10.0);

            if app.config.animation {
                render_animation_style_selector(app, ui);
                ui.add_space(10.0);
            }

            ui.separator();
            ui.add_space(10.0);

            render_ui_sounds_toggle(app, ui);
            ui.add_space(10.0);

            if app.config.ui_sounds_enabled {
                render_sound_volume_slider(app, ui);
                ui.add_space(10.0);
            }

            render_startup_sound_toggle(app, ui);
            ui.add_space(10.0);

            ui.separator();
            ui.add_space(10.0);

            render_default_folder_setting(app, ui);
            ui.add_space(10.0);

            render_sidebar_position_setting(app, ui);
            ui.add_space(20.0);
        });
}

fn render_show_status_bar_toggle(app: &mut WavesApp, ui: &mut egui::Ui) {
    render_toggle_setting(app, ui, 1, "Show Status Bar", app.config.show_status_bar);
}

fn render_show_title_bar_toggle(app: &mut WavesApp, ui: &mut egui::Ui) {
    render_toggle_setting(app, ui, 2, "Show Title Bar", app.config.decorations);
    if ui.input(|i| i.pointer.any_released()) && app.settings_focused_item == 2 {
        ui.ctx().send_viewport_cmd(egui::ViewportCommand::Decorations(app.config.decorations));
    }
}

fn render_visual_animation_toggle(app: &mut WavesApp, ui: &mut egui::Ui) {
    render_toggle_setting(app, ui, 3, "Visual Animation", app.config.animation);
}

fn render_ui_sounds_toggle(app: &mut WavesApp, ui: &mut egui::Ui) {
    render_toggle_setting(app, ui, 6, "UI Sounds", app.config.ui_sounds_enabled);
}

fn render_startup_sound_toggle(app: &mut WavesApp, ui: &mut egui::Ui) {
    render_toggle_setting(app, ui, 8, "Startup Sound", app.config.startup_sound_enabled);
}

fn render_toggle_setting(
    app: &mut WavesApp,
    ui: &mut egui::Ui,
    index: usize,
    label: &str,
    value: bool,
) {
    let is_focused = app.settings_focused_item == index;
    let frame = if is_focused {
        egui::Frame::default()
            .stroke(egui::Stroke::new(2.0, egui::Color32::from_rgb(64, 64, 64)))
            .inner_margin(egui::Margin::same(4.0))
            .rounding(0.0)
    } else {
        egui::Frame::default()
            .inner_margin(egui::Margin::same(4.0))
    };

    frame.show(ui, |ui| {
        let available_width = ui.available_width();

        ui.allocate_ui_with_layout(
            egui::vec2(available_width, 30.0),
            egui::Layout::left_to_right(egui::Align::Center),
            |ui| {
                ui.label(egui::RichText::new(label).size(16.0).color(egui::Color32::WHITE));

                ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
                    let mut new_value = value;
                    render_toggle_widget(ui, value, app.primary_color(), |clicked| {
                        if clicked {
                            new_value = !value;
                        }
                    });

                    if new_value != value {
                        match index {
                            1 => {
                                app.config.show_status_bar = new_value;
                                let _ = app.config.save();
                            }
                            2 => {
                                app.config.decorations = new_value;
                                let _ = app.config.save();
                            }
                            3 => {
                                app.config.animation = new_value;
                                let _ = app.config.save();
                            }
                            6 => {
                                app.config.ui_sounds_enabled = new_value;
                                let _ = app.config.save();
                            }
                            8 => {
                                app.config.startup_sound_enabled = new_value;
                                let _ = app.config.save();
                            }
                            _ => {}
                        }
                    }
                });
            },
        );
    });
}

fn render_toggle_widget<F>(ui: &mut egui::Ui, value: bool, primary_color: egui::Color32, mut callback: F)
where
    F: FnMut(bool),
{
    let toggle_width = 50.0;
    let toggle_height = 25.0;
    let (rect, response) = ui.allocate_exact_size(
        egui::vec2(toggle_width, toggle_height),
        egui::Sense::click(),
    );

    callback(response.clicked());

    if value {
        ui.painter().rect_filled(
            rect,
            0.0,
            egui::Color32::from_rgba_premultiplied(
                primary_color.r(),
                primary_color.g(),
                primary_color.b(),
                13,
            ),
        );
        ui.painter().rect_stroke(rect, 0.0, egui::Stroke::new(1.0, primary_color));
    } else {
        ui.painter().rect_filled(rect, 0.0, egui::Color32::from_rgb(60, 60, 60));
    }

    let square_size = 20.0;
    let square_x = if value {
        rect.max.x - square_size - 2.5
    } else {
        rect.min.x + 2.5
    };
    let square_rect = egui::Rect::from_min_size(
        egui::pos2(square_x, rect.center().y - square_size / 2.0),
        egui::vec2(square_size, square_size),
    );
    ui.painter().rect_filled(square_rect, 0.0, egui::Color32::WHITE);
}

fn render_primary_color_setting(app: &mut WavesApp, ui: &mut egui::Ui) {
    let is_focused = app.settings_focused_item == 0;
    let frame = if is_focused {
        egui::Frame::default()
            .stroke(egui::Stroke::new(2.0, egui::Color32::from_rgb(64, 64, 64)))
            .inner_margin(egui::Margin::same(4.0))
            .rounding(0.0)
    } else {
        egui::Frame::default().inner_margin(egui::Margin::same(4.0))
    };

    frame.show(ui, |ui| {
        ui.label(egui::RichText::new("Primary Color").size(16.0).color(egui::Color32::WHITE));
        ui.add_space(5.0);

        let preset_colors = vec![
            ("#FD5D9C", egui::Color32::from_rgb(253, 93, 156)),
            ("#653DA2", egui::Color32::from_rgb(101, 61, 162)),
            ("#426EA2", egui::Color32::from_rgb(66, 110, 162)),
            ("#AE6024", egui::Color32::from_rgb(174, 96, 36)),
            ("#AE961F", egui::Color32::from_rgb(174, 150, 31)),
            ("#3F9D79", egui::Color32::from_rgb(63, 157, 121)),
        ];

        let available_width = ui.available_width();
        let num_colors = preset_colors.len() as f32;
        let spacing = ui.spacing().item_spacing.x;
        let total_spacing = spacing * (num_colors - 1.0);
        let color_width = (available_width - total_spacing) / num_colors;
        let color_height = 30.0;

        ui.horizontal(|ui| {
            for (hex, color) in preset_colors {
                let size = egui::vec2(color_width, color_height);
                let (rect, response) = ui.allocate_exact_size(size, egui::Sense::click());

                let is_selected = app.config.primary_color.to_lowercase() == hex.to_lowercase();

                let fill_color = egui::Color32::from_rgba_unmultiplied(
                    color.r(),
                    color.g(),
                    color.b(),
                    51,
                );
                ui.painter().rect_filled(rect, 0.0, fill_color);

                let border_width = if is_selected { 2.5 } else { 1.5 };
                ui.painter().rect_stroke(
                    rect,
                    0.0,
                    egui::Stroke::new(border_width, color),
                );

                if response.clicked() {
                    app.config.primary_color = hex.to_string();
                    let _ = app.config.save();
                }
            }
        });
    });
}


fn render_sound_volume_slider(app: &mut WavesApp, ui: &mut egui::Ui) {
    let is_focused = app.settings_focused_item == 7;
    let frame = if is_focused {
        egui::Frame::default()
            .stroke(egui::Stroke::new(2.0, egui::Color32::from_rgb(64, 64, 64)))
            .inner_margin(egui::Margin::same(4.0))
            .rounding(0.0)
    } else {
        egui::Frame::default().inner_margin(egui::Margin::same(4.0))
    };

    frame.show(ui, |ui| {
        ui.label(egui::RichText::new("Sound Volume").size(14.0).color(egui::Color32::from_rgb(200, 200, 200)));
        ui.add_space(5.0);

        let mut ui_volume = app.config.ui_sounds_volume;
        let slider_height = 6.0;

        ui.horizontal(|ui| {
            let slider_width = ui.available_width() - 45.0;
            let (rect, response) = ui.allocate_exact_size(
                egui::vec2(slider_width, slider_height),
                egui::Sense::click_and_drag(),
            );

            let painter = ui.painter();

            painter.rect_filled(
                rect,
                0.0,
                egui::Color32::from_rgb(40, 40, 40),
            );

            let fill_width = slider_width * ui_volume;
            let fill_rect = egui::Rect::from_min_size(
                rect.min,
                egui::vec2(fill_width, slider_height),
            );
            painter.rect_filled(
                fill_rect,
                0.0,
                app.primary_color(),
            );

            if response.dragged() || response.clicked() {
                if let Some(pos) = response.interact_pointer_pos() {
                    let relative_x = (pos.x - rect.min.x).max(0.0).min(slider_width);
                    ui_volume = (relative_x / slider_width).clamp(0.0, 1.0);
                }
            }

            ui.label(egui::RichText::new(format!("{:.0}%", ui_volume * 100.0)).size(14.0).color(egui::Color32::WHITE));
        });

        if ui_volume != app.config.ui_sounds_volume {
            app.config.ui_sounds_volume = ui_volume;
            let _ = app.config.save();
        }
    });
}

fn render_sidebar_position_setting(app: &mut WavesApp, ui: &mut egui::Ui) {
    use crate::config::SidebarPosition;
    
    let is_focused = app.settings_focused_item == 9;
    let frame = if is_focused {
        egui::Frame::default()
            .stroke(egui::Stroke::new(2.0, egui::Color32::from_rgb(64, 64, 64)))
            .inner_margin(egui::Margin::same(4.0))
            .rounding(0.0)
    } else {
        egui::Frame::default().inner_margin(egui::Margin::same(4.0))
    };

    frame.show(ui, |ui| {
        ui.label(egui::RichText::new("Sidebar Position").size(16.0).color(egui::Color32::WHITE));
        ui.add_space(5.0);

        let primary = app.primary_color();

        ui.horizontal(|ui| {
            let is_left = matches!(app.config.sidebar_position, SidebarPosition::Left);
            let left_bg = if is_left { primary } else { egui::Color32::from_rgb(60, 60, 60) };
            let left_text = if is_left { egui::Color32::BLACK } else { egui::Color32::WHITE };

            if ui.add(egui::Button::new(egui::RichText::new("Left").color(left_text)).fill(left_bg)).clicked() {
                app.config.sidebar_position = SidebarPosition::Left;
                let _ = app.config.save();
            }

            let is_right = matches!(app.config.sidebar_position, SidebarPosition::Right);
            let right_bg = if is_right { primary } else { egui::Color32::from_rgb(60, 60, 60) };
            let right_text = if is_right { egui::Color32::BLACK } else { egui::Color32::WHITE };

            if ui.add(egui::Button::new(egui::RichText::new("Right").color(right_text)).fill(right_bg)).clicked() {
                app.config.sidebar_position = SidebarPosition::Right;
                let _ = app.config.save();
            }
        });
    });
}

fn render_animation_style_selector(app: &mut WavesApp, ui: &mut egui::Ui) {
    use crate::config::AnimationType;

    let is_focused = app.settings_focused_item == 4;
    let frame = if is_focused {
        egui::Frame::default()
            .stroke(egui::Stroke::new(2.0, egui::Color32::from_rgb(64, 64, 64)))
            .inner_margin(egui::Margin::same(4.0))
            .rounding(0.0)
    } else {
        egui::Frame::default().inner_margin(egui::Margin::same(4.0))
    };

    frame.show(ui, |ui| {
        ui.label(egui::RichText::new("Animation Style").size(14.0).color(egui::Color32::from_rgb(200, 200, 200)));
        ui.add_space(5.0);

        let primary = app.primary_color();

        let animation_types = vec![
            (AnimationType::Spectrum, "Spectrum"),
            (AnimationType::CircleSpectrum, "Circle"),
            (AnimationType::Agbe, "Agbe"),
            (AnimationType::Dots, "Dots"),
        ];

        ui.horizontal_wrapped(|ui| {
            for (anim_type, label) in animation_types {
                let is_selected = app.config.animation_type == anim_type;
                let bg_color = if is_selected { primary } else { egui::Color32::from_rgb(60, 60, 60) };
                let text_color = if is_selected { egui::Color32::BLACK } else { egui::Color32::WHITE };

                if ui.add(egui::Button::new(egui::RichText::new(label).color(text_color)).fill(bg_color)).clicked() {
                    app.config.animation_type = anim_type;
                    let _ = app.config.save();
                }
            }
        });
    });
}

fn render_default_folder_setting(app: &mut WavesApp, ui: &mut egui::Ui) {
    let is_focused = app.settings_focused_item == 10;
    let frame = if is_focused {
        egui::Frame::default()
            .stroke(egui::Stroke::new(2.0, egui::Color32::from_rgb(64, 64, 64)))
            .inner_margin(egui::Margin::same(4.0))
            .rounding(0.0)
    } else {
        egui::Frame::default().inner_margin(egui::Margin::same(4.0))
    };

    frame.show(ui, |ui| {
        ui.label(egui::RichText::new("Default Music Folder").size(16.0).color(egui::Color32::WHITE));
        ui.add_space(5.0);

        let current_config_value = app.config.default_folder.clone().unwrap_or_else(|| {
            if let Some(music_dir) = dirs::audio_dir() {
                music_dir.to_string_lossy().to_string()
            } else {
                "~/Music".to_string()
            }
        });

        if !ui.memory(|mem| mem.has_focus(ui.next_auto_id())) && app.default_folder_edit != current_config_value {
            app.default_folder_edit = current_config_value;
        }

        ui.horizontal(|ui| {
            let available_width = ui.available_width();
            let button_width = 80.0;
            let text_width = available_width - button_width - ui.spacing().item_spacing.x;

            let text_edit = egui::TextEdit::singleline(&mut app.default_folder_edit)
                .font(egui::TextStyle::Body)
                .desired_width(text_width)
                .text_color(egui::Color32::WHITE);

            let response = ui.add(text_edit);

            if response.changed() || response.lost_focus() {
                let new_value = if app.default_folder_edit.trim().is_empty() {
                    None
                } else {
                    Some(app.default_folder_edit.trim().to_string())
                };

                if new_value != app.config.default_folder {
                    app.config.default_folder = new_value;
                    let _ = app.config.save();
                }
            }

            let button = egui::Button::new(egui::RichText::new("Browse...").color(egui::Color32::WHITE))
                .fill(egui::Color32::from_rgb(60, 60, 60))
                .min_size(egui::vec2(button_width, 0.0));

            if ui.add(button).clicked() {
                if let Some(folder) = rfd::FileDialog::new()
                    .set_directory(&app.default_folder_edit)
                    .pick_folder()
                {
                    let folder_path = folder.to_string_lossy().to_string();
                    app.default_folder_edit = folder_path.clone();
                    app.config.default_folder = Some(folder_path);
                    let _ = app.config.save();
                }
            }
        });
    });
}

