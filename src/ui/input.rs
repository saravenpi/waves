use eframe::egui;
use crate::metadata::extract_metadata;
use crate::audio::PlayerState;
use crate::album_cover::extract_album_cover;

pub struct MetadataEditor {
    pub file_path: std::path::PathBuf,
    pub title: String,
    pub artist: String,
    pub date: String,
    pub cover_path: Option<String>,
    pub has_existing_cover: bool,
    pub existing_cover_data: Option<Vec<u8>>,
    pub cover_changed: bool,
    pub just_opened: bool,
    pub error_message: Option<String>,
}

#[allow(dead_code)]
pub trait NavigationHandler {
    fn handle_navigation(&mut self, key: egui::Key, ctx: &egui::Context);
    fn is_prompt_open(&self) -> bool;
    fn is_search_open(&self) -> bool;
    fn get_columns_mut(&mut self) -> &mut Vec<crate::types::Column>;
    fn get_sidebar_view(&self) -> &crate::types::SidebarView;
    fn get_sidebar_view_mut(&mut self) -> &mut crate::types::SidebarView;
    fn get_liked(&self) -> &Vec<crate::types::Liked>;
    fn get_liked_selected_mut(&mut self) -> &mut usize;
    fn get_current_dir_mut(&mut self) -> &mut std::path::PathBuf;
    fn get_root_dir(&self) -> &std::path::PathBuf;
    fn update_columns_with_selection(&mut self, selection: Option<usize>);
    fn update_columns(&mut self);
    fn play_file(&mut self, path: &std::path::Path, ctx: &egui::Context);
    fn toggle_pause(&mut self);
    fn get_volume_mut(&mut self) -> &mut f32;
    fn get_player(&self) -> &std::sync::Arc<std::sync::Mutex<Option<PlayerState>>>;
    fn set_new_folder_prompt(&mut self, prompt: Option<String>);
    fn set_rename_prompt(&mut self, prompt: Option<(std::path::PathBuf, String)>);
    fn get_clipboard_mut(&mut self) -> &mut Option<(std::path::PathBuf, crate::types::ClipboardOperation)>;
    fn set_delete_confirm_prompt(&mut self, prompt: Option<std::path::PathBuf>);
    fn toggle_like(&mut self);
    fn paste_clipboard(&mut self);
    fn play_next_song(&mut self, ctx: &egui::Context);
    fn play_previous_song(&mut self, ctx: &egui::Context);
    fn set_metadata_editor(&mut self, editor: Option<MetadataEditor>);
}

#[allow(dead_code)]
pub fn handle_navigation<T: NavigationHandler>(app: &mut T, key: egui::Key, ctx: &egui::Context) {
    if app.get_columns_mut().is_empty() {
        return;
    }

    if app.is_prompt_open() || app.is_search_open() {
        return;
    }

    use crate::types::{ClipboardOperation, SidebarView};

    match key {
        egui::Key::J => {
            match app.get_sidebar_view() {
                SidebarView::FileBrowser => {
                    let columns = app.get_columns_mut();
                    if columns[0].selected < columns[0].entries.len().saturating_sub(1) {
                        columns[0].selected += 1;
                    }
                }
                SidebarView::Liked => {
                    let favorites_len = app.get_liked().len();
                    let selected = app.get_liked_selected_mut();
                    if *selected < favorites_len.saturating_sub(1) {
                        *selected += 1;
                    }
                }
                SidebarView::Settings => {}
            }
        }
        egui::Key::K => {
            match app.get_sidebar_view() {
                SidebarView::FileBrowser => {
                    let columns = app.get_columns_mut();
                    if columns[0].selected > 0 {
                        columns[0].selected -= 1;
                    }
                }
                SidebarView::Liked => {
                    let selected = app.get_liked_selected_mut();
                    if *selected > 0 {
                        *selected -= 1;
                    }
                }
                SidebarView::Settings => {}
            }
        }
        egui::Key::L | egui::Key::Enter => {
            match app.get_sidebar_view() {
                SidebarView::FileBrowser => {
                    let selected = app.get_columns_mut()[0].selected;
                    let entry = app.get_columns_mut()[0].entries.get(selected).cloned();
                    if let Some(entry) = entry {
                        if entry.is_dir {
                            *app.get_current_dir_mut() = entry.path.clone();
                            app.update_columns_with_selection(Some(0));
                        } else {
                            app.play_file(&entry.path, ctx);
                        }
                    }
                }
                SidebarView::Liked => {
                    let selected = *app.get_liked_selected_mut();
                    let fav = app.get_liked().get(selected).cloned();
                    if let Some(fav) = fav {
                        if fav.is_dir {
                            *app.get_current_dir_mut() = fav.path.clone();
                            app.update_columns_with_selection(Some(0));
                            *app.get_sidebar_view_mut() = SidebarView::FileBrowser;
                        } else {
                            app.play_file(&fav.path, ctx);
                        }
                    }
                }
                SidebarView::Settings => {}
            }
        }
        egui::Key::H => {
            let parent = app.get_current_dir_mut().parent().map(|p| p.to_path_buf());
            let root_dir = app.get_root_dir().clone();
            if let Some(parent) = parent {
                if parent >= root_dir {
                    *app.get_current_dir_mut() = parent;
                    app.update_columns();
                }
            }
        }
        egui::Key::Space => {
            app.toggle_pause();
        }
        egui::Key::ArrowUp => {
            let volume_val = {
                let volume = app.get_volume_mut();
                *volume = (*volume + 0.05).min(1.0);
                *volume
            };
            if let Ok(player) = app.get_player().lock() {
                if let Some(state) = player.as_ref() {
                    state.sink.set_volume(volume_val);
                }
            }
        }
        egui::Key::ArrowDown => {
            let volume_val = {
                let volume = app.get_volume_mut();
                *volume = (*volume - 0.05).max(0.0);
                *volume
            };
            if let Ok(player) = app.get_player().lock() {
                if let Some(state) = player.as_ref() {
                    state.sink.set_volume(volume_val);
                }
            }
        }
        egui::Key::N => {
            app.set_new_folder_prompt(Some(String::new()));
        }
        egui::Key::R => {
            let selected = app.get_columns_mut()[0].selected;
            let entry = app.get_columns_mut()[0].entries.get(selected).cloned();
            if let Some(entry) = entry {
                app.set_rename_prompt(Some((entry.path.clone(), entry.name.clone())));
            }
        }
        egui::Key::Y => {
            let selected = app.get_columns_mut()[0].selected;
            let entry = app.get_columns_mut()[0].entries.get(selected).cloned();
            if let Some(entry) = entry {
                let clipboard = app.get_clipboard_mut();
                if let Some((path, ClipboardOperation::Copy)) = clipboard {
                    if path == &entry.path {
                        *clipboard = None;
                    } else {
                        *clipboard = Some((entry.path.clone(), ClipboardOperation::Copy));
                    }
                } else {
                    *clipboard = Some((entry.path.clone(), ClipboardOperation::Copy));
                }
            }
        }
        egui::Key::X => {
            let selected = app.get_columns_mut()[0].selected;
            let entry = app.get_columns_mut()[0].entries.get(selected).cloned();
            if let Some(entry) = entry {
                let clipboard = app.get_clipboard_mut();
                if let Some((path, ClipboardOperation::Cut)) = clipboard {
                    if path == &entry.path {
                        *clipboard = None;
                    } else {
                        *clipboard = Some((entry.path.clone(), ClipboardOperation::Cut));
                    }
                } else {
                    *clipboard = Some((entry.path.clone(), ClipboardOperation::Cut));
                }
            }
        }
        egui::Key::P => {
            app.paste_clipboard();
        }
        egui::Key::D => {
            let selected = app.get_columns_mut()[0].selected;
            let entry = app.get_columns_mut()[0].entries.get(selected).cloned();
            if let Some(entry) = entry {
                app.set_delete_confirm_prompt(Some(entry.path.clone()));
            }
        }
        egui::Key::F => {
            app.toggle_like();
        }
        egui::Key::M => {
            match app.get_sidebar_view() {
                SidebarView::FileBrowser => {
                    let selected = app.get_columns_mut()[0].selected;
                    let entry = app.get_columns_mut()[0].entries.get(selected).cloned();
                    if let Some(entry) = entry {
                        if !entry.is_dir {
                            let (title, artist, _album, date, _track, _duration) = extract_metadata(&entry.path);
                            let existing_cover_data = extract_album_cover(&entry.path);
                            let has_existing_cover = existing_cover_data.is_some();
                            app.set_metadata_editor(Some(MetadataEditor {
                                file_path: entry.path.clone(),
                                title,
                                artist: artist.unwrap_or_default(),
                                date: date.unwrap_or_default(),
                                cover_path: None,
                                has_existing_cover,
                                existing_cover_data,
                                cover_changed: false,
                                just_opened: true,
                                error_message: None,
                            }));
                        }
                    }
                }
                SidebarView::Liked => {
                    let selected = *app.get_liked_selected_mut();
                    let fav = app.get_liked().get(selected).cloned();
                    if let Some(fav) = fav {
                        if !fav.is_dir {
                            let (title, artist, _album, date, _track, _duration) = extract_metadata(&fav.path);
                            let existing_cover_data = extract_album_cover(&fav.path);
                            let has_existing_cover = existing_cover_data.is_some();
                            app.set_metadata_editor(Some(MetadataEditor {
                                file_path: fav.path.clone(),
                                title,
                                artist: artist.unwrap_or_default(),
                                date: date.unwrap_or_default(),
                                cover_path: None,
                                has_existing_cover,
                                existing_cover_data,
                                cover_changed: false,
                                just_opened: true,
                                error_message: None,
                            }));
                        }
                    }
                }
                SidebarView::Settings => {}
            }
        }
        egui::Key::Tab => {
            let sidebar_view = app.get_sidebar_view_mut();
            *sidebar_view = match sidebar_view {
                SidebarView::FileBrowser => SidebarView::Liked,
                SidebarView::Liked => SidebarView::Settings,
                SidebarView::Settings => SidebarView::FileBrowser,
            };
        }
        egui::Key::ArrowRight => {
            app.play_next_song(ctx);
        }
        egui::Key::ArrowLeft => {
            app.play_previous_song(ctx);
        }
        egui::Key::Escape => {
            *app.get_clipboard_mut() = None;
        }
        _ => {}
    }
}
