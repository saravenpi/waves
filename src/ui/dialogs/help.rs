use eframe::egui;

use crate::app::WavesApp;

pub fn handle_help_modal(app: &mut WavesApp, ctx: &egui::Context) {
    if app.help_modal_open {
        let mut close_help = false;

        let window_response = egui::Window::new("")
            .anchor(egui::Align2::CENTER_CENTER, [0.0, 0.0])
            .collapsible(false)
            .resizable(false)
            .title_bar(false)
            .fixed_size([700.0, 600.0])
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
                            egui::RichText::new("Keyboard Shortcuts")
                                .size(20.0)
                                .color(egui::Color32::WHITE)
                                .strong()
                        );

                        ui.add_space(10.0);

                        egui::ScrollArea::vertical()
                            .max_height(500.0)
                            .show(ui, |ui| {
                                render_keybindings(app, ui);
                            });

                        ui.add_space(10.0);

                        ui.horizontal(|ui| {
                            if ui.button("Close").clicked() {
                                close_help = true;
                            }
                        });

                        ui.add_space(5.0);

                        ui.label(
                            egui::RichText::new("Press ESC or ? to close")
                                .size(10.0)
                                .color(egui::Color32::from_rgb(100, 100, 100))
                        );
                    });

                    if ui.input(|i| i.key_pressed(egui::Key::Escape)) {
                        close_help = true;
                    }
                });
            });

        if let Some(response) = window_response {
            if ctx.input(|i| i.pointer.primary_released()) {
                if let Some(pos) = ctx.input(|i| i.pointer.interact_pos()) {
                    if !response.response.rect.contains(pos) {
                        close_help = true;
                    }
                }
            }
        }

        if close_help {
            app.help_modal_open = false;
        }
    }
}

fn render_keybindings(app: &WavesApp, ui: &mut egui::Ui) {
    let keybind = |ui: &mut egui::Ui, key: &str, desc: &str| {
        ui.horizontal(|ui| {
            ui.label(
                egui::RichText::new(key)
                    .size(14.0)
                    .color(app.primary_color())
                    .monospace()
            );
            ui.label(
                egui::RichText::new(desc)
                    .size(14.0)
                    .color(egui::Color32::from_rgb(200, 200, 200))
            );
        });
        ui.add_space(5.0);
    };

    ui.label(egui::RichText::new("Navigation").size(16.0).color(egui::Color32::WHITE).strong());
    ui.add_space(5.0);
    keybind(ui, "h/j/k/l", "Navigate left/down/up/right");
    keybind(ui, "ENTER", "Select directory or play file");
    keybind(ui, "TAB", "Cycle views (Files → Liked → Settings)");
    keybind(ui, "ESC", "Cancel clipboard operation");

    ui.add_space(10.0);
    ui.label(egui::RichText::new("Playback").size(16.0).color(egui::Color32::WHITE).strong());
    ui.add_space(5.0);
    keybind(ui, "SPACE", "Pause/resume playback");
    keybind(ui, "←/→", "Previous/next track");
    keybind(ui, "↑/↓", "Increase/decrease volume");

    ui.add_space(10.0);
    ui.label(egui::RichText::new("File Operations").size(16.0).color(egui::Color32::WHITE).strong());
    ui.add_space(5.0);
    keybind(ui, "n", "Create new folder");
    keybind(ui, "r", "Rename selected file/folder");
    keybind(ui, "y", "Copy (yank) selected item");
    keybind(ui, "x", "Cut selected item");
    keybind(ui, "p", "Paste into current directory");
    keybind(ui, "d", "Delete selected item");

    ui.add_space(10.0);
    ui.label(egui::RichText::new("Organization").size(16.0).color(egui::Color32::WHITE).strong());
    ui.add_space(5.0);
    keybind(ui, "f", "Like/unlike selected item");
    keybind(ui, "m", "Edit metadata (audio files only)");
    keybind(ui, "/", "Search files");

    ui.add_space(10.0);
    ui.label(egui::RichText::new("View").size(16.0).color(egui::Color32::WHITE).strong());
    ui.add_space(5.0);
    keybind(ui, "b", "Toggle browse mode");
    keybind(ui, "?", "Show/hide this help");
}
