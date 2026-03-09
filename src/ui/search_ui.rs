use eframe::egui;
use std::path::PathBuf;

use crate::app::WavesApp;

pub fn render_search_bar(app: &mut WavesApp, ui: &mut egui::Ui, _ctx: &egui::Context) {
    if !app.animation_fullscreen {
        ui.add_space(10.0);
        ui.horizontal(|ui| {
            ui.add_space(10.0);

            let search_bar_width = ui.available_width() - 10.0;
            let primary_color = app.primary_color();

            let search_frame = egui::Frame {
                fill: egui::Color32::from_rgb(20, 20, 20),
                stroke: egui::Stroke::new(1.0, primary_color),
                inner_margin: egui::Margin::symmetric(8.0, 6.0),
                rounding: egui::Rounding::same(0.0),
                ..Default::default()
            };

            search_frame.show(ui, |ui| {
                let search_response = ui.add_sized(
                    [search_bar_width - 16.0, 18.0],
                    egui::TextEdit::singleline(&mut app.search_query)
                        .hint_text("🔍 Search files...")
                        .frame(false)
                        .id(egui::Id::new("main_search_bar"))
                );

                if app.search_just_opened {
                    search_response.request_focus();
                    app.search_just_opened = false;
                }

                if app.search_query.starts_with('/') {
                    app.search_query = app.search_query[1..].to_string();
                }

                if !app.search_query.is_empty() {
                    app.perform_search();
                } else {
                    app.search_results.clear();
                    app.search_selected = 0;
                }
            });

            ui.add_space(10.0);
        });
    }
}

pub fn render_search_results(app: &mut WavesApp, ui: &mut egui::Ui, ctx: &egui::Context, search_has_focus: bool) {
    if !app.search_results.is_empty() {
        ui.add_space(5.0);

        if search_has_focus || !app.search_results.is_empty() {
            if ui.input(|i| i.key_pressed(egui::Key::ArrowDown)) {
                let max_display = app.search_results.len().min(5).saturating_sub(1);
                if app.search_selected < max_display {
                    app.search_selected += 1;
                }
            }
            if ui.input(|i| i.key_pressed(egui::Key::ArrowUp)) {
                if app.search_selected > 0 {
                    app.search_selected -= 1;
                }
            }
            if ui.input(|i| i.key_pressed(egui::Key::Enter)) {
                if let Some(result) = app.search_results.get(app.search_selected) {
                    let path = result.path.clone();
                    app.play_file(&path, ctx);
                    app.search_query.clear();
                    app.search_results.clear();
                    app.search_selected = 0;
                }
            }
            if ui.input(|i| i.key_pressed(egui::Key::Escape)) {
                app.search_query.clear();
                app.search_results.clear();
                app.search_selected = 0;
            }
        }

        ui.horizontal(|ui| {
            ui.add_space(10.0);

            let mut clicked_result: Option<PathBuf> = None;

            egui::Frame {
                fill: egui::Color32::from_rgb(15, 15, 15),
                stroke: egui::Stroke::new(1.0, egui::Color32::from_rgb(64, 64, 64)),
                inner_margin: egui::Margin::same(8.0),
                ..Default::default()
            }
            .show(ui, |ui| {
                ui.style_mut().spacing.item_spacing = egui::vec2(0.0, 0.0);

                let display_results = app.search_results.iter().take(5);

                for (idx, result) in display_results.enumerate() {
                    let is_selected = idx == app.search_selected;

                    let display_text = if let Some(ref artist) = result.artist {
                        format!("{} - {}", result.title, artist)
                    } else {
                        result.title.clone()
                    };

                    let album_text = result.album.as_ref()
                        .map(|a| format!(" [{}]", a))
                        .unwrap_or_default();

                    let full_text = format!("{}{}", display_text, album_text);

                    let (rect, response) = ui.allocate_exact_size(
                        egui::vec2(ui.available_width() - 8.0, 28.0),
                        egui::Sense::click()
                    );

                    if response.clicked() {
                        clicked_result = Some(result.path.clone());
                    }

                    if is_selected {
                        let primary = app.primary_color();
                        ui.painter().rect_filled(
                            rect,
                            0.0,
                            app.primary_color_with_alpha(13)
                        );
                        ui.painter().rect_stroke(
                            rect,
                            0.0,
                            egui::Stroke::new(1.0, primary),
                        );
                    }

                    let text_color = if is_selected {
                        app.primary_color()
                    } else {
                        egui::Color32::from_rgb(200, 200, 200)
                    };

                    ui.painter().text(
                        rect.left_center() + egui::vec2(8.0, 0.0),
                        egui::Align2::LEFT_CENTER,
                        &full_text,
                        egui::FontId::proportional(13.0),
                        text_color
                    );
                }
            });

            if let Some(path) = clicked_result {
                app.play_file(&path, ctx);
                app.search_query.clear();
                app.search_results.clear();
                app.search_selected = 0;
            }

            ui.add_space(10.0);
        });

        ui.add_space(5.0);
    } else if !app.animation_fullscreen {
        ui.add_space(10.0);
    }
}
