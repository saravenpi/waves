use eframe::egui;
use std::path::PathBuf;

use crate::app::WavesApp;
use crate::ui::input::MetadataEditor;
use crate::metadata::save_audio_metadata;

pub fn handle_metadata_editor(app: &mut WavesApp, ctx: &egui::Context) {
    let mut save_result: Option<(std::path::PathBuf, String, String, Option<String>, Option<String>)> = None;

    if let Some(editor) = &mut app.metadata_editor {
        let mut close_editor = false;
        let mut save_metadata = false;

        let window_response = egui::Window::new("")
            .anchor(egui::Align2::CENTER_CENTER, [0.0, 0.0])
            .collapsible(false)
            .resizable(false)
            .title_bar(false)
            .fixed_size([600.0, 400.0])
            .frame(egui::Frame {
                fill: egui::Color32::TRANSPARENT,
                stroke: egui::Stroke::NONE,
                ..Default::default()
            })
            .show(ctx, |ui| {
                egui::Frame {
                    fill: egui::Color32::from_rgb(8, 8, 8),
                    stroke: egui::Stroke::new(1.0, egui::Color32::from_rgb(64, 64, 64)),
                    inner_margin: egui::Margin::same(20.0),
                    ..Default::default()
                }
                .show(ui, |ui| {
                    ui.vertical(|ui| {
                        ui.label(
                            egui::RichText::new("Edit Metadata")
                                .size(18.0)
                                .color(egui::Color32::WHITE)
                        );

                        ui.add_space(10.0);

                        ui.add_sized(
                            [ui.available_width(), 20.0],
                            egui::TextEdit::singleline(&mut editor.title)
                                .font(egui::TextStyle::Monospace)
                                .hint_text("title...")
                                .frame(false)
                        );

                        ui.add_space(10.0);

                        ui.add_sized(
                            [ui.available_width(), 20.0],
                            egui::TextEdit::singleline(&mut editor.artist)
                                .font(egui::TextStyle::Monospace)
                                .hint_text("artist...")
                                .frame(false)
                        );

                        ui.add_space(10.0);

                        ui.add_sized(
                            [ui.available_width(), 20.0],
                            egui::TextEdit::singleline(&mut editor.date)
                                .font(egui::TextStyle::Monospace)
                                .hint_text("date (year)...")
                                .frame(false)
                        );

                        ui.add_space(10.0);

                        render_cover_preview(editor, ui, ctx);

                        ui.add_space(5.0);

                        ui.horizontal(|ui| {
                            if ui.button("Select Cover Image...").clicked() {
                                if let Some(path) = rfd::FileDialog::new()
                                    .add_filter("Images", &["png", "jpg", "jpeg"])
                                    .pick_file()
                                {
                                    editor.cover_path = Some(path.to_string_lossy().to_string());
                                    editor.cover_changed = true;
                                }
                            }

                            if editor.has_existing_cover || editor.cover_path.is_some() {
                                if ui.button("Remove Cover").clicked() {
                                    editor.cover_path = None;
                                    editor.cover_changed = true;
                                }
                            }
                        });

                        ui.add_space(20.0);

                        if let Some(error) = &editor.error_message {
                            ui.label(
                                egui::RichText::new(error)
                                    .size(12.0)
                                    .color(egui::Color32::from_rgb(255, 100, 100))
                            );
                            ui.add_space(10.0);
                        }

                        ui.horizontal(|ui| {
                            if ui.button("Save").clicked() {
                                save_metadata = true;
                            }

                            ui.add_space(10.0);

                            if ui.button("Cancel").clicked() {
                                close_editor = true;
                            }
                        });

                        ui.add_space(5.0);

                        ui.label(
                            egui::RichText::new("Press ESC to cancel")
                                .size(10.0)
                                .color(egui::Color32::from_rgb(100, 100, 100))
                        );
                    });

                    if ui.input(|i| i.key_pressed(egui::Key::Escape)) {
                        close_editor = true;
                    }
                });
            });

        if let Some(response) = window_response {
            if editor.just_opened {
                editor.just_opened = false;
            } else if ctx.input(|i| i.pointer.primary_released()) {
                if let Some(pos) = ctx.input(|i| i.pointer.interact_pos()) {
                    if !response.response.rect.contains(pos) {
                        close_editor = true;
                    }
                }
            }
        }

        save_result = if save_metadata {
            Some(perform_metadata_save(editor))
        } else {
            None
        };

        if close_editor {
            app.metadata_editor = None;
        }
    }

    if let Some((file_path, title, artist, cover_path, error_msg)) = save_result {
        if let Some(err) = error_msg {
            if let Some(editor) = &mut app.metadata_editor {
                editor.error_message = Some(err);
            }
        } else {
            app.album_cover_cache.pop(&file_path);
            app.last_selected_file = None;

            if let Ok(mut player) = app.player.lock() {
                if let Some(state) = player.as_mut() {
                    if state.current_file == file_path {
                        state.title = title.clone();
                        state.artist = Some(artist.clone()).filter(|a| !a.is_empty());
                        state.album_cover = None;
                    }
                }
            }

            app.metadata_editor = None;
        }

        if let Some(temp_path) = cover_path {
            if temp_path.contains("waves_temp_cover") {
                let _ = std::fs::remove_file(&temp_path);
            }
        }
    }
}

fn render_cover_preview(editor: &MetadataEditor, ui: &mut egui::Ui, ctx: &egui::Context) {
    if editor.has_existing_cover && !editor.cover_changed {
        if let Some(cover_data) = &editor.existing_cover_data {
            if let Ok(img) = image::load_from_memory(cover_data) {
                let size = [img.width() as usize, img.height() as usize];
                let rgba = img.to_rgba8();
                let pixels = rgba.as_flat_samples();
                let color_image = egui::ColorImage::from_rgba_unmultiplied(
                    size,
                    pixels.as_slice()
                );

                let texture = ctx.load_texture(
                    "existing_cover",
                    color_image,
                    Default::default()
                );

                ui.add(egui::Image::new(&texture).max_size(egui::vec2(100.0, 100.0)));

                ui.add_space(5.0);
                ui.label(
                    egui::RichText::new("✓ Existing cover (will be preserved)")
                        .size(12.0)
                        .color(egui::Color32::from_rgb(100, 200, 100))
                );
            }
        }
    } else if let Some(cover_path) = &editor.cover_path {
        let filename = std::path::Path::new(cover_path)
            .file_name()
            .and_then(|n| n.to_str())
            .unwrap_or("unknown");
        ui.label(
            egui::RichText::new(format!("📎 New cover: {}", filename))
                .size(12.0)
                .color(egui::Color32::from_rgb(150, 150, 150))
        );
    } else if editor.has_existing_cover && editor.cover_changed {
        ui.label(
            egui::RichText::new("⚠ Existing cover will be removed")
                .size(12.0)
                .color(egui::Color32::from_rgb(255, 150, 100))
        );
    } else {
        ui.label(
            egui::RichText::new("No cover")
                .size(12.0)
                .color(egui::Color32::from_rgb(100, 100, 100))
        );
    }
}

fn perform_metadata_save(editor: &MetadataEditor) -> (PathBuf, String, String, Option<String>, Option<String>) {
    let file_path = editor.file_path.clone();
    let title = editor.title.clone();
    let artist = editor.artist.clone();
    let date = editor.date.clone();

    let cover_path_to_use = if !editor.cover_changed && editor.has_existing_cover {
        if let Some(existing_cover_data) = &editor.existing_cover_data {
            let temp_dir = std::env::temp_dir();
            let temp_cover_path = temp_dir.join("waves_temp_cover.jpg");
            match std::fs::write(&temp_cover_path, existing_cover_data) {
                Ok(_) => Some(temp_cover_path.to_string_lossy().to_string()),
                Err(e) => {
                    eprintln!("Failed to write temp cover file: {}", e);
                    None
                }
            }
        } else {
            None
        }
    } else {
        editor.cover_path.clone()
    };

    let error_msg = match save_audio_metadata(&file_path, &title, &artist, &date, cover_path_to_use.as_deref()) {
        Err(e) => {
            eprintln!("Failed to save metadata: {}", e);
            Some(format!("Error: {}", e))
        }
        Ok(()) => {
            eprintln!("Metadata saved successfully, refreshing UI...");
            None
        }
    };

    (file_path, title, artist, cover_path_to_use, error_msg)
}
