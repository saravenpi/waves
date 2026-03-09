use eframe::egui;
use std::time::SystemTime;

use crate::app::WavesApp;
use crate::types::Liked;
use crate::ui::helpers::{show_context_menu, ContextMenuAction};
use crate::ui::input::MetadataEditor;
use crate::metadata::extract_metadata;

pub fn handle_context_menu(app: &mut WavesApp, ctx: &egui::Context) {
    if let Some((path, pos)) = &app.context_menu.clone() {
        let is_dir = path.is_dir();
        if let Some(action) = show_context_menu(ctx, path, *pos, is_dir) {
            match action {
                ContextMenuAction::Rename => {
                    let name = path.file_name()
                        .unwrap_or_default()
                        .to_string_lossy()
                        .to_string();
                    app.rename_prompt = Some((path.clone(), name));
                }
                ContextMenuAction::Delete => {
                    app.delete_confirm_prompt = Some(path.clone());
                }
                ContextMenuAction::Copy => {
                    app.clipboard = Some((path.clone(), crate::types::ClipboardOperation::Copy));
                }
                ContextMenuAction::Cut => {
                    app.clipboard = Some((path.clone(), crate::types::ClipboardOperation::Cut));
                }
                ContextMenuAction::ToggleLike => {
                    if is_dir {
                    } else if let Some(idx) = app.liked.iter().position(|f| f.path == *path) {
                        app.liked.remove(idx);
                        let _ = crate::liked::save(&app.liked);
                    } else {
                        let name = path.file_name()
                            .unwrap_or_default()
                            .to_string_lossy()
                            .to_string();
                        app.liked.push(Liked {
                            path: path.clone(),
                            name,
                            is_dir: false,
                            timestamp: SystemTime::now(),
                        });
                        let _ = crate::liked::save(&app.liked);
                    }
                }
                ContextMenuAction::EditMetadata => {
                    let (title, artist, _album, date, _track, _duration) = extract_metadata(path);
                    let existing_cover_data = crate::album_cover::extract_album_cover(path);
                    let has_existing_cover = existing_cover_data.is_some();
                    app.metadata_editor = Some(MetadataEditor {
                        file_path: path.clone(),
                        title,
                        artist: artist.unwrap_or_default(),
                        date: date.unwrap_or_default(),
                        cover_path: None,
                        has_existing_cover,
                        existing_cover_data,
                        cover_changed: false,
                        just_opened: true,
                        error_message: None,
                    });
                }
            }
            app.context_menu = None;
        }

        if ctx.input(|i| i.pointer.primary_released()) {
            app.context_menu = None;
        }
    }
}
