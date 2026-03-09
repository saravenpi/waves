use eframe::egui;
use std::fs;

use crate::app::WavesApp;
use crate::ui::components::ConfirmDialog;

pub fn handle_delete_confirm_prompt(app: &mut WavesApp, ctx: &egui::Context) {
    if let Some(delete_path) = &app.delete_confirm_prompt {
        let file_name = delete_path.file_name()
            .and_then(|n| n.to_str())
            .unwrap_or("unknown");
        let is_dir = delete_path.is_dir();
        let message = if is_dir {
            format!("Delete folder '{}' and all its contents?", file_name)
        } else {
            format!("Delete file '{}'?", file_name)
        };

        let delete_path_clone = delete_path.clone();

        let (confirmed, cancelled, new_selected) = ConfirmDialog::new("Confirm Delete", &message)
            .confirm_text("Delete")
            .cancel_text("Cancel")
            .selected(app.delete_confirm_selected)
            .show(ctx, app.primary_color());

        app.delete_confirm_selected = new_selected;

        if confirmed {
            let result = if is_dir {
                fs::remove_dir_all(&delete_path_clone)
            } else {
                fs::remove_file(&delete_path_clone)
            };

            if let Err(e) = result {
                eprintln!("Failed to delete: {}", e);
            } else {
                crate::sound::play_delete_sound();

                let was_playing = {
                    let player = app.player.lock().unwrap();
                    if let Some(state) = player.as_ref() {
                        state.current_file == delete_path_clone
                    } else {
                        false
                    }
                };

                if was_playing {
                    let mut player = app.player.lock().unwrap();
                    *player = None;
                }

                app.update_columns();

                if let Some((clipboard_path, _)) = &app.clipboard {
                    if clipboard_path == &delete_path_clone {
                        app.clipboard = None;
                    }
                }
            }
            app.delete_confirm_prompt = None;
            app.delete_confirm_selected = 1;
        }

        if cancelled {
            app.delete_confirm_prompt = None;
            app.delete_confirm_selected = 1;
        }
    }
}
