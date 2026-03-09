use eframe::egui;
use std::fs;

use crate::app::WavesApp;
use crate::ui::helpers::show_text_prompt;

pub fn handle_new_folder_prompt(app: &mut WavesApp, ctx: &egui::Context) {
    if let Some(folder_name) = &mut app.new_folder_prompt {
        let (confirmed, cancelled) = show_text_prompt(
            ctx,
            "folder name...",
            folder_name,
        );

        if confirmed {
            let new_path = app.current_dir.join(folder_name.clone());
            if let Err(e) = fs::create_dir(&new_path) {
                eprintln!("Failed to create folder: {}", e);
            } else {
                app.update_columns();
            }
            app.new_folder_prompt = None;
        }

        if cancelled {
            app.new_folder_prompt = None;
        }
    }
}

pub fn handle_rename_prompt(app: &mut WavesApp, ctx: &egui::Context) {
    if let Some((old_path, new_name)) = &mut app.rename_prompt {
        let old_path_clone = old_path.clone();
        let (confirmed, cancelled) = show_text_prompt(
            ctx,
            "new name...",
            new_name,
        );

        if confirmed {
            if let Some(parent) = old_path_clone.parent() {
                let new_path = parent.join(new_name.clone());
                if let Err(e) = fs::rename(&old_path_clone, &new_path) {
                    eprintln!("Failed to rename: {}", e);
                } else {
                    app.update_columns();
                }
            } else {
                eprintln!("Failed to rename: no parent directory");
            }
            app.rename_prompt = None;
        }

        if cancelled {
            app.rename_prompt = None;
        }
    }
}
