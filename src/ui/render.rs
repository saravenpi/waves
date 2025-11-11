use eframe::egui;
use std::fs;
use std::path::PathBuf;
use std::time::{Duration, SystemTime};

use crate::app::WavesApp;
use crate::config::SidebarPosition;
use crate::types::{FileEntry, Favorite, ClipboardOperation, SidebarView};
use crate::ui::helpers::{ContextMenuAction, show_confirm_prompt, show_text_prompt, show_context_menu};
use crate::ui::input::MetadataEditor;
use crate::utils::{format_duration, truncate_text};
use crate::metadata::{extract_metadata, save_audio_metadata};
use crate::favorites;
use crate::file_operations::SearchResult;

impl eframe::App for WavesApp {
    fn update(&mut self, ctx: &egui::Context, _frame: &mut eframe::Frame) {
        if let Some(file_path) = self.file_to_play_on_start.take() {
            if !self.columns.is_empty() {
                if let Some(idx) = self.columns[0].entries.iter().position(|e| e.path == file_path) {
                    self.columns[0].selected = idx;
                }
            }
            self.play_file(&file_path, ctx);
        }

        while let Ok(file_path) = self.file_open_receiver.try_recv() {
            eprintln!("WAVES: Handling file open request from receiver: {:?}", file_path);
            if file_path.exists() {
                if file_path.is_file() {
                    if let Some(parent) = file_path.parent() {
                        self.current_dir = parent.to_path_buf();
                        self.root_dir = parent.to_path_buf();
                        self.update_columns();

                        if !self.columns.is_empty() {
                            if let Some(idx) = self.columns[0].entries.iter().position(|e| e.path == file_path) {
                                self.columns[0].selected = idx;
                            }
                        }
                    }
                    self.play_file(&file_path, ctx);
                } else if file_path.is_dir() {
                    self.current_dir = file_path.clone();
                    self.root_dir = file_path;
                    self.update_columns();
                }
            }
        }

        #[cfg(target_os = "macos")]
        while let Ok(action) = self.menu_action_receiver.try_recv() {
            use crate::macos::MenuAction;
            match action {
                MenuAction::OpenFile => {
                    if let Some(path) = rfd::FileDialog::new()
                        .add_filter("Audio Files", &["mp3", "wav", "flac", "ogg", "m4a"])
                        .pick_file()
                    {
                        if let Some(parent) = path.parent() {
                            self.current_dir = parent.to_path_buf();
                            self.root_dir = parent.to_path_buf();
                            self.update_columns();

                            if !self.columns.is_empty() {
                                if let Some(idx) = self.columns[0].entries.iter().position(|e| e.path == path) {
                                    self.columns[0].selected = idx;
                                }
                            }
                        }
                        self.play_file(&path, ctx);
                    }
                }
                MenuAction::OpenFolder => {
                    if let Some(path) = rfd::FileDialog::new().pick_folder() {
                        self.current_dir = path.clone();
                        self.root_dir = path;
                        self.update_columns();
                    }
                }
            }
        }

        while let Ok((path, waveform)) = self.waveform_receiver.try_recv() {
            self.waveform_cache.insert(path.clone(), waveform.clone());

            if let Ok(mut player) = self.player.lock() {
                if let Some(state) = player.as_mut() {
                    if state.current_file == path {
                        state.waveform = waveform;
                        ctx.request_repaint();
                    }
                }
            }
        }

        while let Ok((path, color_image)) = self.album_cover_receiver.try_recv() {
            let texture = ctx.load_texture(
                format!("album_cover_{}", path.display()),
                color_image,
                egui::TextureOptions::LINEAR
            );

            self.album_cover_cache.insert(path.clone(), texture.clone());

            if let Ok(mut player) = self.player.lock() {
                if let Some(state) = player.as_mut() {
                    if state.current_file == path {
                        state.album_cover = Some(texture);
                        ctx.request_repaint();
                    }
                }
            }
            ctx.request_repaint();
        }

        if ctx.input(|i| i.pointer.is_moving()) {
            self.last_mouse_movement = std::time::Instant::now();
        }

        if self.animation_fullscreen {
            ctx.input(|i| {
                for event in &i.events {
                    if let egui::Event::Key { key, pressed: true, .. } = event {
                        match key {
                            egui::Key::Escape => {
                                self.animation_fullscreen = false;
                            }
                            egui::Key::Space => {
                                self.toggle_pause();
                            }
                            egui::Key::ArrowLeft => {
                                match self.playback_context {
                                    SidebarView::Favorites => self.play_previous_favorite(ctx),
                                    _ => self.play_previous_song(ctx),
                                }
                            }
                            egui::Key::ArrowRight => {
                                match self.playback_context {
                                    SidebarView::Favorites => self.play_next_favorite(ctx),
                                    _ => self.play_next_song(ctx),
                                }
                            }
                            egui::Key::ArrowUp => {
                                self.volume = (self.volume + 0.05).min(1.0);
                                if let Ok(player) = self.player.lock() {
                                    if let Some(state) = player.as_ref() {
                                        state.sink.set_volume(self.volume);
                                    }
                                }
                            }
                            egui::Key::ArrowDown => {
                                self.volume = (self.volume - 0.05).max(0.0);
                                if let Ok(player) = self.player.lock() {
                                    if let Some(state) = player.as_ref() {
                                        state.sink.set_volume(self.volume);
                                    }
                                }
                            }
                            _ => {}
                        }
                    }
                }
            });

            let opacity = (self.config.window_opacity / 100.0 * 255.0) as u8;
            let bg_color = egui::Color32::from_rgba_unmultiplied(0, 0, 0, opacity);

            egui::CentralPanel::default()
                .frame(egui::Frame::none().fill(bg_color))
                .show(ctx, |ui| {
                    let fullscreen_rect = ui.max_rect();
                    self.render_animation(ui, fullscreen_rect);

                    let mouse_idle_duration = self.last_mouse_movement.elapsed().as_secs_f32();
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
                            self.animation_fullscreen = false;
                        }
                    }
                });

            ctx.request_repaint();
            return;
        }

        ctx.input(|i| {
            for event in &i.events {
                if let egui::Event::Key { key, pressed: true, modifiers, .. } = event {
                    if self.metadata_editor.is_some() || self.new_folder_prompt.is_some() ||
                       self.rename_prompt.is_some() || self.search_open {
                        continue;
                    }

                    if *key == egui::Key::F && (modifiers.command || modifiers.ctrl) {
                        self.search_open = !self.search_open;
                        if self.search_open {
                            self.search_just_opened = true;
                        } else {
                            self.search_query.clear();
                            self.search_results.clear();
                        }
                    } else {
                        self.handle_navigation(*key, ctx);
                    }
                }
            }

            if !i.raw.dropped_files.is_empty() {
                for file in &i.raw.dropped_files {
                    if let Some(path) = &file.path {
                        eprintln!("WAVES: File dropped: {:?}", path);
                        if path.exists() {
                            if path.is_file() {
                                if let Some(parent) = path.parent() {
                                    self.current_dir = parent.to_path_buf();
                                    self.root_dir = parent.to_path_buf();
                                    self.update_columns();

                                    if !self.columns.is_empty() {
                                        if let Some(idx) = self.columns[0].entries.iter().position(|e| &e.path == path) {
                                            self.columns[0].selected = idx;
                                        }
                                    }
                                }
                                self.play_file(path, ctx);
                            } else if path.is_dir() {
                                self.current_dir = path.clone();
                                self.root_dir = path.clone();
                                self.update_columns();
                            }
                        }
                    }
                }
            }
        });

        let now = std::time::Instant::now();
        if now.duration_since(self.last_folder_check).as_secs() >= 2 {
            self.last_folder_check = now;

            if let Ok(entries) = std::fs::read_dir(&self.current_dir) {
                let current_count = entries.count();
                if current_count != self.last_folder_file_count {
                    self.last_folder_file_count = current_count;
                    self.update_columns();
                }
            }
        }

        let (is_playing, is_empty, is_paused) = {
            let player = self.player.lock().unwrap();
            if let Some(state) = player.as_ref() {
                (true, state.sink.empty(), state.sink.is_paused())
            } else {
                (false, false, false)
            }
        };

        if is_playing {
            if is_empty {
                if self.loop_enabled {
                    let current_file = self.player.lock().unwrap()
                        .as_ref()
                        .map(|state| state.current_file.clone());
                    if let Some(file) = current_file {
                        self.play_file(&file, ctx);
                    }
                } else {
                    match self.playback_context {
                        SidebarView::Favorites => self.play_next_favorite(ctx),
                        _ => self.play_next_song(ctx),
                    }
                }
            } else if !is_paused {
                let (audio_buffer, sample_rate, channels) = {
                    let player = self.player.lock().unwrap();
                    if let Some(state) = player.as_ref() {
                        (state.audio_buffer.clone(), state.sample_rate, state.channels)
                    } else {
                        return;
                    }
                };

                self.update_spectrum(&audio_buffer, sample_rate, channels);
                ctx.request_repaint();
            } else {
                let audio_buffer = {
                    let player = self.player.lock().unwrap();
                    if let Some(state) = player.as_ref() {
                        Some(state.audio_buffer.clone())
                    } else {
                        None
                    }
                };

                if let Some(buffer) = audio_buffer {
                    buffer.lock().unwrap().clear();
                }

                for i in 0..self.spectrum_bars.len() {
                    self.spectrum_bars[i] = (self.spectrum_bars[i] - 0.02).max(0.0);
                }
                if self.spectrum_bars.iter().any(|&x| x > 0.0) {
                    ctx.request_repaint();
                }
            }
        }

        if self.config.show_status_bar {
            let status_opacity = ((self.config.window_opacity / 100.0) * 255.0).min(255.0) as u8;

            egui::TopBottomPanel::bottom("status")
                .frame(egui::Frame::default().fill(egui::Color32::from_black_alpha(status_opacity)))
                .show(ctx, |ui| {
                    ui.separator();
                    let volume_percent = (self.volume * 100.0) as i32;
                    let status_text = format!(" h/j/k/l: navigate | ENTER: select/play | SPACE: pause | ←/→: prev/next | n: new | r: rename | y: copy | x: cut | p: paste | d: delete | f: fav | TAB: view | ↑/↓: vol ({}%)", volume_percent);
                    ui.label(egui::RichText::new(status_text).size(18.0).color(egui::Color32::WHITE).monospace());
                });
        }

        let current_playing_file = if is_playing {
            let player = self.player.lock().unwrap();
            player.as_ref().map(|state| state.current_file.clone())
        } else {
            None
        };

        let sidebar_panel = match self.config.sidebar_position {
            SidebarPosition::Left => egui::SidePanel::left("file_browser"),
            SidebarPosition::Right => egui::SidePanel::right("file_browser"),
        };

        let opacity_alpha = (self.config.window_opacity * 2.40).min(240.0) as u8;

        let sidebar_panel_configured = if self.sidebar_collapsed {
            sidebar_panel
                .resizable(false)
                .exact_width(40.0)
                .frame(egui::Frame::default()
                    .fill(egui::Color32::from_black_alpha(opacity_alpha))
                    .stroke(egui::Stroke::new(1.0, egui::Color32::WHITE))
                    .inner_margin(egui::Margin::same(5.0)))
        } else {
            sidebar_panel
                .resizable(true)
                .default_width(self.config.sidebar_width)
                .width_range(250.0..=800.0)
                .frame(egui::Frame::default()
                    .fill(egui::Color32::from_black_alpha(opacity_alpha))
                    .stroke(egui::Stroke::new(1.0, egui::Color32::WHITE))
                    .inner_margin(egui::Margin::same(10.0)))
        };

        let sidebar_response = sidebar_panel_configured
            .show(ctx, |ui| {
                let is_left_sidebar = matches!(self.config.sidebar_position, SidebarPosition::Left);
                let expand_arrow = if is_left_sidebar { "▶" } else { "◀" };
                let collapse_arrow = if is_left_sidebar { "◀" } else { "▶" };

                if self.sidebar_collapsed {
                    ui.vertical_centered(|ui| {
                        ui.add_space(10.0);
                        let expand_button = egui::Button::new(
                            egui::RichText::new(expand_arrow).size(20.0).color(egui::Color32::WHITE)
                        )
                        .fill(egui::Color32::TRANSPARENT)
                        .stroke(egui::Stroke::NONE)
                        .min_size(egui::vec2(30.0, 30.0));

                        let expand_response = ui.add(expand_button);
                        expand_response.surrender_focus();

                        if expand_response.hovered() {
                            ui.painter().rect_stroke(
                                expand_response.rect,
                                0.0,
                                egui::Stroke::new(1.0, egui::Color32::from_rgb(150, 150, 150))
                            );
                        }

                        if expand_response.is_pointer_button_down_on() {
                            ui.painter().rect_filled(
                                expand_response.rect,
                                0.0,
                                egui::Color32::from_rgb(60, 60, 60)
                            );
                            ui.painter().text(
                                expand_response.rect.center(),
                                egui::Align2::CENTER_CENTER,
                                expand_arrow,
                                egui::FontId::proportional(20.0),
                                egui::Color32::WHITE,
                            );
                        }

                        if expand_response.clicked() {
                            self.sidebar_collapsed = false;
                        }
                    });
                    return;
                }

                ui.horizontal(|ui| {
                    ui.with_layout(egui::Layout::right_to_left(egui::Align::Min), |ui| {
                        let collapse_button = egui::Button::new(
                            egui::RichText::new(collapse_arrow).size(20.0).color(egui::Color32::WHITE)
                        )
                        .fill(egui::Color32::TRANSPARENT)
                        .stroke(egui::Stroke::NONE)
                        .min_size(egui::vec2(30.0, 30.0));

                        let collapse_response = ui.add(collapse_button);
                        collapse_response.surrender_focus();

                        if collapse_response.hovered() {
                            ui.painter().rect_stroke(
                                collapse_response.rect,
                                0.0,
                                egui::Stroke::new(1.0, egui::Color32::from_rgb(150, 150, 150))
                            );
                        }

                        if collapse_response.clicked() {
                            self.sidebar_collapsed = true;
                        }
                    });
                });

                ui.add_space(5.0);
                let full_height = ui.available_height();
                let metadata_height = 200.0;
                let browser_height = full_height - metadata_height - 20.0;

                let mut clicked_entry: Option<(usize, FileEntry)> = None;
                let mut back_button_clicked = false;
                let mut context_menu_event: Option<(PathBuf, egui::Pos2)> = None;

                ui.allocate_ui_with_layout(
                    egui::vec2(ui.available_width(), browser_height),
                    egui::Layout::top_down(egui::Align::LEFT),
                    |ui| {
                        let folder_name = self.current_dir.file_name()
                            .and_then(|n| n.to_str())
                            .unwrap_or("/");

                        ui.allocate_ui_with_layout(
                            egui::vec2(ui.available_width(), 30.0),
                            egui::Layout::left_to_right(egui::Align::Center),
                            |ui| {
                                ui.add_space(2.0);

                                let at_root = self.current_dir == self.root_dir;
                                if !at_root {
                                    let back_button = egui::Button::new(
                                        egui::RichText::new("<")
                                            .size(14.0)
                                            .color(egui::Color32::from_rgb(150, 150, 150))
                                    )
                                    .fill(egui::Color32::TRANSPARENT)
                                    .stroke(egui::Stroke::NONE)
                                    .frame(false);

                                    if ui.add(back_button).clicked() {
                                        back_button_clicked = true;
                                    }

                                    ui.add_space(5.0);
                                }
                                ui.label(
                                    egui::RichText::new(folder_name)
                                        .size(14.0)
                                        .color(egui::Color32::from_rgb(150, 150, 150))
                                );

                                let mut search_clicked = false;

                                ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
                                ui.add_space(2.0);

                                ui.add_space(5.0);

                                let search_button = egui::Button::new(
                                    egui::RichText::new("🔍")
                                        .size(18.0)
                                )
                                .fill(egui::Color32::TRANSPARENT)
                                .stroke(egui::Stroke::NONE)
                                .min_size(egui::vec2(45.0, 30.0));

                                let search_response = ui.add(search_button);

                                if search_response.hovered() {
                                    let rect = search_response.rect;
                                    ui.painter().rect_stroke(rect, 0.0, egui::Stroke::new(1.0, egui::Color32::WHITE));
                                }

                                if search_response.clicked() {
                                    search_clicked = true;
                                }

                                ui.add_space(5.0);

                                let section_text = match self.sidebar_view {
                                    SidebarView::FileBrowser => "File Browser",
                                    SidebarView::Favorites => "Favorites",
                                    SidebarView::Settings => "Settings",
                                };
                                ui.label(egui::RichText::new(section_text).size(14.0).color(egui::Color32::from_rgb(150, 150, 150)));
                            });

                            if search_clicked {
                                self.search_open = !self.search_open;
                                if self.search_open {
                                    self.search_just_opened = true;
                                } else {
                                    self.search_query.clear();
                                    self.search_results.clear();
                                }
                            }
                        });
                        ui.add_space(5.0);
                        ui.separator();
                        ui.add_space(5.0);

                        let header_height = 35.0;
                        let list_height = browser_height - header_height;

                        match self.sidebar_view {
                            SidebarView::FileBrowser => {
                        if !self.columns.is_empty() {
                            let column = &self.columns[0];

                            egui::ScrollArea::vertical()
                                .auto_shrink([false, false])
                                .max_height(list_height)
                                .scroll_bar_visibility(egui::scroll_area::ScrollBarVisibility::VisibleWhenNeeded)
                                .show(ui, |ui| {
                                    let available_width = ui.available_width();
                                    let max_chars = ((available_width - 10.0) / 10.5) as usize;

                                    for (idx, entry) in column.entries.iter().enumerate() {
                                        let is_selected = idx == column.selected;

                                        let is_playing_or_parent = if let Some(playing_path) = &current_playing_file {
                                            if playing_path == &entry.path {
                                                true
                                            } else if entry.is_dir {
                                                playing_path.starts_with(&entry.path)
                                            } else {
                                                false
                                            }
                                        } else {
                                            false
                                        };

                                        let (rect, response) = ui.allocate_exact_size(
                                            egui::vec2(available_width, 25.0),
                                            egui::Sense::click()
                                        );

                                        if response.clicked() {
                                            clicked_entry = Some((idx, entry.clone()));
                                        }

                                        if response.secondary_clicked() {
                                            if let Some(pos) = response.interact_pointer_pos() {
                                                context_menu_event = Some((entry.path.clone(), pos));
                                            }
                                        }

                                        let is_hovered = response.hovered();

                                        let icon = if entry.is_dir {
                                            "📁"
                                        } else {
                                            "🎵"
                                        };

                                        let display_name = truncate_text(&entry.name, max_chars.saturating_sub(4));
                                        let is_favorited = self.favorites.iter().any(|f| f.path == entry.path);
                                        let star = if is_favorited { "* " } else { "" };
                                        let display_text = format!(" {}{} {}", star, icon, display_name);

                                        let is_in_clipboard = self.clipboard.as_ref()
                                            .map(|(path, _)| path == &entry.path)
                                            .unwrap_or(false);

                                        let color = if is_selected {
                                            let bg_color = if is_playing_or_parent {
                                                self.primary_color()
                                            } else {
                                                egui::Color32::WHITE
                                            };
                                            ui.painter().rect_filled(
                                                rect,
                                                0.0,
                                                bg_color,
                                            );
                                            if is_playing_or_parent {
                                                egui::Color32::WHITE
                                            } else {
                                                egui::Color32::BLACK
                                            }
                                        } else {
                                            if is_playing_or_parent {
                                                ui.painter().rect_stroke(
                                                    rect,
                                                    0.0,
                                                    egui::Stroke::new(2.0, self.primary_color()),
                                                );
                                            } else if is_hovered {
                                                ui.painter().rect_stroke(
                                                    rect,
                                                    0.0,
                                                    egui::Stroke::new(1.0, egui::Color32::WHITE),
                                                );
                                            }

                                            if is_in_clipboard {
                                                egui::Color32::from_rgb(100, 100, 100)
                                            } else {
                                                egui::Color32::WHITE
                                            }
                                        };

                                        ui.painter().text(
                                            rect.left_top() + egui::vec2(2.0, 1.0),
                                            egui::Align2::LEFT_TOP,
                                            &display_text,
                                            egui::FontId::monospace(18.0),
                                            color,
                                        );
                                    }
                                });
                        }
                            }
                            SidebarView::Favorites => {
                                let favorites_clone = self.favorites.clone();
                                let selected = self.favorites_selected;
                                let mut clicked_favorite: Option<usize> = None;
                                let mut context_menu_event: Option<(PathBuf, egui::Pos2)> = None;

                                egui::ScrollArea::vertical()
                                    .auto_shrink([false, false])
                                    .max_height(list_height)
                                    .scroll_bar_visibility(egui::scroll_area::ScrollBarVisibility::VisibleWhenNeeded)
                                    .show(ui, |ui| {
                                        let available_width = ui.available_width();
                                        let max_chars = ((available_width - 10.0) / 10.5) as usize;

                                        for (idx, fav) in favorites_clone.iter().enumerate() {
                                            let is_selected = idx == selected;

                                            let is_playing_or_parent = if let Some(playing_path) = &current_playing_file {
                                                if playing_path == &fav.path {
                                                    true
                                                } else if fav.is_dir {
                                                    playing_path.starts_with(&fav.path)
                                                } else {
                                                    false
                                                }
                                            } else {
                                                false
                                            };

                                            let (rect, response) = ui.allocate_exact_size(
                                                egui::vec2(available_width, 25.0),
                                                egui::Sense::click()
                                            );

                                            if response.clicked() {
                                                clicked_favorite = Some(idx);
                                            }

                                            if response.secondary_clicked() {
                                                if let Some(pos) = response.interact_pointer_pos() {
                                                    context_menu_event = Some((fav.path.clone(), pos));
                                                }
                                            }

                                            let is_hovered = response.hovered();

                                            let icon = if fav.is_dir {
                                                "📁"
                                            } else {
                                                "🎵"
                                            };

                                            let display_name = truncate_text(&fav.name, max_chars.saturating_sub(5));
                                            let display_text = format!(" * {} {}", icon, display_name);

                                            let color = if is_selected {
                                                let bg_color = if is_playing_or_parent {
                                                    self.primary_color()
                                                } else {
                                                    egui::Color32::WHITE
                                                };
                                                ui.painter().rect_filled(
                                                    rect,
                                                    0.0,
                                                    bg_color,
                                                );
                                                if is_playing_or_parent {
                                                    egui::Color32::WHITE
                                                } else {
                                                    egui::Color32::BLACK
                                                }
                                            } else {
                                                if is_playing_or_parent {
                                                    ui.painter().rect_stroke(
                                                        rect,
                                                        0.0,
                                                        egui::Stroke::new(2.0, self.primary_color()),
                                                    );
                                                } else if is_hovered {
                                                    ui.painter().rect_stroke(
                                                        rect,
                                                        0.0,
                                                        egui::Stroke::new(1.0, egui::Color32::WHITE),
                                                    );
                                                }
                                                egui::Color32::WHITE
                                            };

                                            ui.painter().text(
                                                rect.left_top() + egui::vec2(2.0, 1.0),
                                                egui::Align2::LEFT_TOP,
                                                &display_text,
                                                egui::FontId::monospace(18.0),
                                                color,
                                            );
                                        }
                                    });

                                if let Some((path, pos)) = context_menu_event {
                                    self.context_menu = Some((path, pos));
                                }

                                if let Some(idx) = clicked_favorite {
                                    self.favorites_selected = idx;
                                    if let Some(fav) = favorites_clone.get(idx) {
                                        if fav.is_dir {
                                            self.current_dir = fav.path.clone();
                                            self.update_columns_with_selection(Some(0));
                                            self.sidebar_view = SidebarView::FileBrowser;
                                        } else {
                                            self.playback_context = SidebarView::Favorites;
                                            self.play_file(&fav.path, ctx);
                                        }
                                    }
                                }
                            }
                            SidebarView::Settings => {
                                ui.add_space(10.0);
                                ui.heading(egui::RichText::new("Settings").size(20.0).color(egui::Color32::WHITE));
                                ui.add_space(20.0);

                                egui::ScrollArea::vertical()
                                    .auto_shrink([false, false])
                                    .max_height(list_height)
                                    .show(ui, |ui| {
                                        ui.add_space(10.0);

                                        let is_focused = self.settings_focused_item == 0;
                                        let frame = if is_focused {
                                            egui::Frame::default()
                                                .stroke(egui::Stroke::new(2.0, egui::Color32::WHITE))
                                                .inner_margin(egui::Margin::same(8.0))
                                                .rounding(0.0)
                                        } else {
                                            egui::Frame::default().inner_margin(egui::Margin::same(8.0))
                                        };
                                        frame.show(ui, |ui| {
                                            ui.label(egui::RichText::new("Primary Color").size(16.0).color(egui::Color32::WHITE));
                                            ui.add_space(5.0);

                                            let preset_colors = vec![
                                                ("#9664FF", egui::Color32::from_rgb(150, 100, 255)),
                                                ("#4A90E2", egui::Color32::from_rgb(74, 144, 226)),
                                                ("#50E3C2", egui::Color32::from_rgb(80, 227, 194)),
                                                ("#FF6B9D", egui::Color32::from_rgb(255, 107, 157)),
                                                ("#FF8A00", egui::Color32::from_rgb(255, 138, 0)),
                                                ("#FF4444", egui::Color32::from_rgb(255, 68, 68)),
                                            ];

                                            ui.horizontal(|ui| {
                                                for (hex, color) in preset_colors {
                                                    let size = egui::vec2(30.0, 30.0);
                                                    let (rect, response) = ui.allocate_exact_size(size, egui::Sense::click());

                                                    let is_selected = self.config.primary_color.to_lowercase() == hex.to_lowercase();

                                                    if is_selected {
                                                        ui.painter().rect_stroke(
                                                            rect.expand(3.0),
                                                            0.0,
                                                            egui::Stroke::new(2.0, egui::Color32::WHITE),
                                                        );
                                                    }

                                                    ui.painter().rect_filled(rect, 0.0, color);

                                                    if response.clicked() {
                                                        self.config.primary_color = hex.to_string();
                                                        let _ = self.config.save();
                                                    }
                                                }
                                            });
                                        });
                                        ui.add_space(15.0);

                                        let is_focused = self.settings_focused_item == 1;
                                        let frame = if is_focused {
                                            egui::Frame::default()
                                                .stroke(egui::Stroke::new(2.0, egui::Color32::WHITE))
                                                .inner_margin(egui::Margin::same(8.0))
                                                .rounding(0.0)
                                        } else {
                                            egui::Frame::default().inner_margin(egui::Margin::same(8.0))
                                        };
                                        frame.show(ui, |ui| {
                                            ui.label(egui::RichText::new("Window Opacity").size(16.0).color(egui::Color32::WHITE));
                                            ui.add_space(5.0);

                                            let mut opacity = self.config.window_opacity;
                                            let slider_height = 6.0;
                                            let slider_width = ui.available_width() - 60.0;

                                            ui.horizontal(|ui| {
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

                                                let fill_width = slider_width * (opacity / 100.0);
                                                let fill_rect = egui::Rect::from_min_size(
                                                    rect.min,
                                                    egui::vec2(fill_width, slider_height),
                                                );
                                                painter.rect_filled(
                                                    fill_rect,
                                                    0.0,
                                                    self.primary_color(),
                                                );

                                                if response.dragged() || response.clicked() {
                                                    if let Some(pos) = response.interact_pointer_pos() {
                                                        let relative_x = (pos.x - rect.min.x).max(0.0).min(slider_width);
                                                        opacity = ((relative_x / slider_width) * 100.0).clamp(0.0, 100.0);
                                                    }
                                                }

                                                ui.label(egui::RichText::new(format!("{:.0}%", opacity)).size(14.0).color(egui::Color32::WHITE));
                                            });

                                            if opacity != self.config.window_opacity {
                                                self.config.window_opacity = opacity;
                                                let _ = self.config.save();
                                            }
                                        });
                                        ui.add_space(15.0);

                                        ui.separator();
                                        ui.add_space(15.0);

                                        let is_focused = self.settings_focused_item == 2;
                                        let frame = if is_focused {
                                            egui::Frame::default()
                                                .stroke(egui::Stroke::new(2.0, egui::Color32::WHITE))
                                                .inner_margin(egui::Margin::same(8.0))
                                                .rounding(0.0)
                                        } else {
                                            egui::Frame::default().inner_margin(egui::Margin::same(8.0))
                                        };
                                        frame.show(ui, |ui| {
                                            ui.horizontal(|ui| {
                                                ui.label(egui::RichText::new("Show Status Bar").size(16.0).color(egui::Color32::WHITE));

                                                ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
                                                    let toggle_width = 50.0;
                                                    let toggle_height = 25.0;
                                                    let (rect, response) = ui.allocate_exact_size(
                                                        egui::vec2(toggle_width, toggle_height),
                                                        egui::Sense::click()
                                                    );

                                                    if response.clicked() {
                                                        self.config.show_status_bar = !self.config.show_status_bar;
                                                        let _ = self.config.save();
                                                    }

                                                    let bg_color = if self.config.show_status_bar {
                                                        self.primary_color()
                                                    } else {
                                                        egui::Color32::from_rgb(60, 60, 60)
                                                    };

                                                    ui.painter().rect_filled(rect, 0.0, bg_color);

                                                    let square_size = 20.0;
                                                    let square_x = if self.config.show_status_bar {
                                                        rect.max.x - square_size - 2.5
                                                    } else {
                                                        rect.min.x + 2.5
                                                    };
                                                    let square_rect = egui::Rect::from_min_size(
                                                        egui::pos2(square_x, rect.center().y - square_size / 2.0),
                                                        egui::vec2(square_size, square_size)
                                                    );
                                                    ui.painter().rect_filled(square_rect, 0.0, egui::Color32::WHITE);
                                                });
                                            });
                                        });
                                        ui.add_space(15.0);

                                        let is_focused = self.settings_focused_item == 3;
                                        let frame = if is_focused {
                                            egui::Frame::default()
                                                .stroke(egui::Stroke::new(2.0, egui::Color32::WHITE))
                                                .inner_margin(egui::Margin::same(8.0))
                                                .rounding(0.0)
                                        } else {
                                            egui::Frame::default().inner_margin(egui::Margin::same(8.0))
                                        };
                                        frame.show(ui, |ui| {
                                            ui.horizontal(|ui| {
                                                ui.label(egui::RichText::new("Visual Animation").size(16.0).color(egui::Color32::WHITE));

                                                ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
                                                    let toggle_width = 50.0;
                                                    let toggle_height = 25.0;
                                                    let (rect, response) = ui.allocate_exact_size(
                                                        egui::vec2(toggle_width, toggle_height),
                                                        egui::Sense::click()
                                                    );

                                                    if response.clicked() {
                                                        self.config.animation = !self.config.animation;
                                                        let _ = self.config.save();
                                                    }

                                                    let bg_color = if self.config.animation {
                                                        self.primary_color()
                                                    } else {
                                                        egui::Color32::from_rgb(60, 60, 60)
                                                    };

                                                    ui.painter().rect_filled(rect, 0.0, bg_color);

                                                    let square_size = 20.0;
                                                    let square_x = if self.config.animation {
                                                        rect.max.x - square_size - 2.5
                                                    } else {
                                                        rect.min.x + 2.5
                                                    };
                                                    let square_rect = egui::Rect::from_min_size(
                                                        egui::pos2(square_x, rect.center().y - square_size / 2.0),
                                                        egui::vec2(square_size, square_size)
                                                    );
                                                    ui.painter().rect_filled(square_rect, 0.0, egui::Color32::WHITE);
                                                });
                                            });
                                        });
                                        ui.add_space(10.0);

                                        if self.config.animation {
                                            let is_focused = self.settings_focused_item == 4;
                                            let frame = if is_focused {
                                                egui::Frame::default()
                                                    .stroke(egui::Stroke::new(2.0, egui::Color32::WHITE))
                                                    .inner_margin(egui::Margin::same(8.0))
                                                    .rounding(0.0)
                                            } else {
                                                egui::Frame::default().inner_margin(egui::Margin::same(8.0))
                                            };
                                            frame.show(ui, |ui| {
                                                ui.label(egui::RichText::new("Animation Style").size(14.0).color(egui::Color32::from_rgb(200, 200, 200)));
                                                ui.add_space(5.0);
                                                ui.horizontal(|ui| {
                                                    use crate::config::AnimationType;
                                                    for anim_type in AnimationType::all() {
                                                        let is_selected = self.config.animation_type == anim_type;

                                                        let button_text = anim_type.display_name();
                                                        let text_color = if is_selected {
                                                            egui::Color32::BLACK
                                                        } else {
                                                            egui::Color32::WHITE
                                                        };
                                                        let bg_color = if is_selected {
                                                            egui::Color32::WHITE
                                                        } else {
                                                            egui::Color32::TRANSPARENT
                                                        };

                                                        let button = egui::Button::new(egui::RichText::new(button_text).color(text_color))
                                                            .fill(bg_color)
                                                            .stroke(egui::Stroke::NONE);

                                                        if ui.add(button).clicked() {
                                                            self.config.animation_type = anim_type;
                                                            let _ = self.config.save();
                                                        }
                                                    }
                                                });
                                            });
                                        }
                                        ui.add_space(15.0);

                                        ui.separator();
                                        ui.add_space(15.0);

                                        let is_focused = self.settings_focused_item == 5;
                                        let frame = if is_focused {
                                            egui::Frame::default()
                                                .stroke(egui::Stroke::new(2.0, egui::Color32::WHITE))
                                                .inner_margin(egui::Margin::same(8.0))
                                                .rounding(0.0)
                                        } else {
                                            egui::Frame::default().inner_margin(egui::Margin::same(8.0))
                                        };
                                        frame.show(ui, |ui| {
                                            ui.label(egui::RichText::new("Sidebar Position").size(16.0).color(egui::Color32::WHITE));
                                            ui.add_space(5.0);
                                            ui.horizontal(|ui| {
                                                let is_left = matches!(self.config.sidebar_position, SidebarPosition::Left);

                                                let left_text_color = if is_left {
                                                    egui::Color32::BLACK
                                                } else {
                                                    egui::Color32::WHITE
                                                };
                                                let left_bg_color = if is_left {
                                                    egui::Color32::WHITE
                                                } else {
                                                    egui::Color32::TRANSPARENT
                                                };

                                                let left_button = egui::Button::new(egui::RichText::new("Left").color(left_text_color))
                                                    .fill(left_bg_color)
                                                    .stroke(egui::Stroke::NONE);

                                                if ui.add(left_button).clicked() {
                                                    self.config.sidebar_position = SidebarPosition::Left;
                                                    let _ = self.config.save();
                                                }

                                                let right_text_color = if !is_left {
                                                    egui::Color32::BLACK
                                                } else {
                                                    egui::Color32::WHITE
                                                };
                                                let right_bg_color = if !is_left {
                                                    egui::Color32::WHITE
                                                } else {
                                                    egui::Color32::TRANSPARENT
                                                };

                                                let right_button = egui::Button::new(egui::RichText::new("Right").color(right_text_color))
                                                    .fill(right_bg_color)
                                                    .stroke(egui::Stroke::NONE);

                                                if ui.add(right_button).clicked() {
                                                    self.config.sidebar_position = SidebarPosition::Right;
                                                    let _ = self.config.save();
                                                }
                                            });
                                        });
                                        ui.add_space(15.0);

                                        ui.separator();
                                        ui.add_space(15.0);

                                        let is_focused = self.settings_focused_item == 6;
                                        let frame = if is_focused {
                                            egui::Frame::default()
                                                .stroke(egui::Stroke::new(2.0, egui::Color32::WHITE))
                                                .inner_margin(egui::Margin::same(8.0))
                                                .rounding(0.0)
                                        } else {
                                            egui::Frame::default().inner_margin(egui::Margin::same(8.0))
                                        };
                                        frame.show(ui, |ui| {
                                            ui.horizontal(|ui| {
                                                ui.label(egui::RichText::new("UI Sounds").size(16.0).color(egui::Color32::WHITE));

                                                ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
                                                    let toggle_width = 50.0;
                                                    let toggle_height = 25.0;
                                                    let (rect, response) = ui.allocate_exact_size(
                                                        egui::vec2(toggle_width, toggle_height),
                                                        egui::Sense::click()
                                                    );

                                                    if response.clicked() {
                                                        self.config.ui_sounds_enabled = !self.config.ui_sounds_enabled;
                                                        let _ = self.config.save();
                                                    }

                                                    let bg_color = if self.config.ui_sounds_enabled {
                                                        self.primary_color()
                                                    } else {
                                                        egui::Color32::from_rgb(60, 60, 60)
                                                    };

                                                    ui.painter().rect_filled(rect, 0.0, bg_color);

                                                    let square_size = 20.0;
                                                    let square_x = if self.config.ui_sounds_enabled {
                                                        rect.max.x - square_size - 2.5
                                                    } else {
                                                        rect.min.x + 2.5
                                                    };
                                                    let square_rect = egui::Rect::from_min_size(
                                                        egui::pos2(square_x, rect.center().y - square_size / 2.0),
                                                        egui::vec2(square_size, square_size)
                                                    );
                                                    ui.painter().rect_filled(square_rect, 0.0, egui::Color32::WHITE);
                                                });
                                            });
                                        });
                                        ui.add_space(10.0);

                                        if self.config.ui_sounds_enabled {
                                            let is_focused = self.settings_focused_item == 7;
                                            let frame = if is_focused {
                                                egui::Frame::default()
                                                    .stroke(egui::Stroke::new(2.0, egui::Color32::WHITE))
                                                    .inner_margin(egui::Margin::same(8.0))
                                                    .rounding(0.0)
                                            } else {
                                                egui::Frame::default().inner_margin(egui::Margin::same(8.0))
                                            };
                                            frame.show(ui, |ui| {
                                                ui.label(egui::RichText::new("Sound Volume").size(14.0).color(egui::Color32::from_rgb(200, 200, 200)));
                                                ui.add_space(5.0);

                                                let mut ui_volume = self.config.ui_sounds_volume;
                                                let slider_height = 6.0;
                                                let slider_width = ui.available_width() - 80.0;

                                                ui.horizontal(|ui| {
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

                                                    let fill_width = slider_width * ui_volume;
                                                    let fill_rect = egui::Rect::from_min_size(
                                                        rect.min,
                                                        egui::vec2(fill_width, slider_height),
                                                    );
                                                    painter.rect_filled(
                                                        fill_rect,
                                                        0.0,
                                                        self.primary_color(),
                                                    );

                                                    if response.dragged() || response.clicked() {
                                                        if let Some(pos) = response.interact_pointer_pos() {
                                                            let relative_x = (pos.x - rect.min.x).max(0.0).min(slider_width);
                                                            ui_volume = (relative_x / slider_width).clamp(0.0, 1.0);
                                                        }
                                                    }

                                                    ui.label(egui::RichText::new(format!("{:.0}%", ui_volume * 100.0)).size(14.0).color(egui::Color32::WHITE));
                                                });

                                                if ui_volume != self.config.ui_sounds_volume {
                                                    self.config.ui_sounds_volume = ui_volume;
                                                    let _ = self.config.save();
                                                }
                                            });
                                        }
                                        ui.add_space(15.0);

                                        ui.separator();
                                        ui.add_space(15.0);

                                        let is_focused = self.settings_focused_item == 8;
                                        let frame = if is_focused {
                                            egui::Frame::default()
                                                .stroke(egui::Stroke::new(2.0, egui::Color32::WHITE))
                                                .inner_margin(egui::Margin::same(8.0))
                                                .rounding(0.0)
                                        } else {
                                            egui::Frame::default().inner_margin(egui::Margin::same(8.0))
                                        };
                                        frame.show(ui, |ui| {
                                            ui.label(egui::RichText::new("Default Folder").size(16.0).color(egui::Color32::WHITE));
                                            ui.add_space(5.0);
                                            ui.horizontal(|ui| {
                                                let text_edit = egui::TextEdit::singleline(&mut self.default_folder_input)
                                                    .desired_width(300.0)
                                                    .hint_text("~/Music");
                                                ui.add(text_edit);
                                                if ui.button("Save").clicked() {
                                                    let trimmed = self.default_folder_input.trim();
                                                    if !trimmed.is_empty() {
                                                        self.config.default_folder = Some(trimmed.to_string());
                                                    } else {
                                                        self.config.default_folder = None;
                                                    }
                                                    let _ = self.config.save();
                                                }
                                            });
                                            ui.label(egui::RichText::new("Use ~ for home directory (e.g., ~/Music)").size(12.0).color(egui::Color32::from_rgb(140, 140, 140)));
                                        });
                                        ui.add_space(30.0);
                                    });
                            }
                        }
                    }
                );

                if let Some((idx, entry)) = clicked_entry {
                    self.columns[0].selected = idx;

                    if entry.is_dir {
                        self.current_dir = entry.path.clone();
                        self.update_columns_with_selection(Some(0));
                    } else {
                        self.playback_context = SidebarView::FileBrowser;
                        self.play_file(&entry.path, ctx);
                    }
                }

                if let Some((path, pos)) = context_menu_event {
                    self.context_menu = Some((path, pos));
                }

                if back_button_clicked {
                    if let Some(parent) = self.current_dir.parent() {
                        if parent >= self.root_dir.as_path() {
                            self.current_dir = parent.to_path_buf();
                            self.update_columns_with_selection(Some(0));
                        }
                    }
                }

            });

        if !self.sidebar_collapsed {
            let new_width = sidebar_response.response.rect.width().max(250.0).min(800.0);
            if (new_width - self.config.sidebar_width).abs() > 1.0 {
                self.config.sidebar_width = new_width;
                let _ = self.config.save();
            }
        }

        let central_opacity = ((self.config.window_opacity / 100.0) * 255.0).min(255.0) as u8;

        egui::CentralPanel::default()
            .frame(egui::Frame::default().fill(egui::Color32::from_black_alpha(central_opacity)))
            .show(ctx, |ui| {
                let player_info = if is_playing {
                    let player = self.player.lock().unwrap();
                    player.as_ref().map(|state| {
                        (state.title.clone(), state.artist.clone(), state.duration, state.waveform.clone(), state.album_cover.clone())
                    })
                } else {
                    None
                };

                if let Some((title, artist, duration, waveform, album_cover)) = player_info {
                    let total_height = ui.available_height();
                    let bottom_panel_height = 200.0;
                    let spectrum_height = total_height - bottom_panel_height;

                    if self.config.animation {
                        let (spectrum_rect, spectrum_response) = ui.allocate_exact_size(
                            egui::vec2(ui.available_width(), spectrum_height),
                            egui::Sense::hover()
                        );

                        self.render_animation(ui, spectrum_rect);

                        if spectrum_response.hovered() {
                            self.last_animation_hover = std::time::Instant::now();
                        }

                        let hover_elapsed = self.last_animation_hover.elapsed().as_secs_f32();
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
                                self.animation_fullscreen = true;
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
                        ui.add_space(20.0);

                        let cover_size = 140.0;
                        let (rect, _) = ui.allocate_exact_size(
                            egui::vec2(cover_size, cover_size),
                            egui::Sense::hover()
                        );

                        if let Some(texture) = &album_cover {
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

                        ui.add_space(20.0);

                        ui.vertical(|ui| {
                            ui.set_width(ui.available_width() - 50.0);
                            ui.horizontal(|ui| {
                                ui.vertical(|ui| {
                                    ui.label(
                                        egui::RichText::new(&title)
                                            .size(24.0)
                                            .color(egui::Color32::WHITE)
                                    );

                                    if let Some(artist_name) = &artist {
                                        ui.label(
                                            egui::RichText::new(artist_name)
                                                .size(20.0)
                                                .color(egui::Color32::from_rgb(180, 180, 180))
                                        );
                                    }
                                });

                                ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
                                    ui.add_space(20.0);

                                    let next_button = egui::Button::new(egui::RichText::new("⏭").size(28.0).color(egui::Color32::WHITE))
                                        .fill(egui::Color32::TRANSPARENT)
                                        .stroke(egui::Stroke::NONE);
                                    let next_response = ui.add(next_button);
                                    next_response.surrender_focus();
                                    if next_response.hovered() {
                                        let rect = next_response.rect;
                                        ui.painter().rect_stroke(rect, 0.0, egui::Stroke::new(1.0, egui::Color32::WHITE));
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
                                        self.play_next_song(ctx);
                                    }

                                    ui.add_space(8.0);

                                    let pause_play_text = if is_paused { "▶" } else { "⏸" };
                                    let play_pause_button = egui::Button::new(egui::RichText::new(pause_play_text).size(28.0).color(egui::Color32::WHITE))
                                        .fill(egui::Color32::TRANSPARENT)
                                        .stroke(egui::Stroke::NONE);
                                    let play_pause_response = ui.add(play_pause_button);
                                    play_pause_response.surrender_focus();
                                    if play_pause_response.hovered() {
                                        let rect = play_pause_response.rect;
                                        ui.painter().rect_stroke(rect, 0.0, egui::Stroke::new(1.0, egui::Color32::WHITE));
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
                                        self.toggle_pause();
                                    }

                                    ui.add_space(8.0);

                                    let loop_color = if self.loop_enabled {
                                        self.primary_color()
                                    } else {
                                        egui::Color32::WHITE
                                    };
                                    let loop_button = egui::Button::new(egui::RichText::new("🔁").size(24.0).color(loop_color))
                                        .fill(egui::Color32::TRANSPARENT)
                                        .stroke(egui::Stroke::NONE);
                                    let loop_response = ui.add(loop_button);
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
                                        self.loop_enabled = !self.loop_enabled;
                                    }

                                    ui.add_space(8.0);

                                    let prev_button = egui::Button::new(egui::RichText::new("⏮").size(28.0).color(egui::Color32::WHITE))
                                        .fill(egui::Color32::TRANSPARENT)
                                        .stroke(egui::Stroke::NONE);
                                    let prev_response = ui.add(prev_button);
                                    prev_response.surrender_focus();
                                    if prev_response.hovered() {
                                        let rect = prev_response.rect;
                                        ui.painter().rect_stroke(rect, 0.0, egui::Stroke::new(1.0, egui::Color32::WHITE));
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
                                        self.play_previous_song(ctx);
                                    }
                                });
                            });

                            ui.add_space(10.0);

                            let waveform_width = (ui.available_width() - 40.0).max(100.0);
                            let waveform_height = 60.0;

                            let (rect, response) = ui.allocate_exact_size(
                                egui::vec2(waveform_width, waveform_height),
                                egui::Sense::click_and_drag()
                            );

                            if response.dragged() || response.is_pointer_button_down_on() {
                                if let Some(pos) = response.interact_pointer_pos() {
                                    let click_x = (pos.x - rect.min.x).max(0.0).min(rect.width());
                                    let progress = (click_x / rect.width()).clamp(0.0, 1.0);
                                    self.pending_seek = Some(progress);
                                    ctx.request_repaint();
                                }
                            } else if response.drag_stopped() || response.clicked() {
                                if let Some(pending) = self.pending_seek {
                                    self.seek_to_position(pending);
                                    self.pending_seek = None;
                                }
                            }

                            let painter = ui.painter();

                            let bar_width = rect.width() / waveform.len() as f32;
                            let max_height = rect.height() * 0.8;

                            let current_pos = self.get_current_position().unwrap_or(Duration::from_secs(0));
                            let progress = if let Some(pending) = self.pending_seek {
                                pending
                            } else if duration.as_secs() > 0 {
                                current_pos.as_secs_f32() / duration.as_secs_f32()
                            } else {
                                0.0
                            };

                            for (i, &amplitude) in waveform.iter().enumerate() {
                                let x = rect.min.x + (i as f32 * bar_width);
                                let adjusted_amplitude = (amplitude * 0.4).min(1.0);
                                let height = adjusted_amplitude * max_height;
                                let y_top = rect.center().y - height / 2.0;
                                let y_bottom = rect.center().y + height / 2.0;

                                let bar_progress = i as f32 / waveform.len() as f32;
                                let color = if bar_progress <= progress {
                                    self.primary_color()
                                } else {
                                    egui::Color32::from_rgb(60, 60, 60)
                                };

                                painter.line_segment(
                                    [egui::pos2(x, y_top), egui::pos2(x, y_bottom)],
                                    egui::Stroke::new(bar_width * 0.8, color),
                                );
                            }

                            let progress_x = rect.min.x + progress * rect.width();
                            let marker_color = if self.pending_seek.is_some() {
                                egui::Color32::from_rgb(255, 200, 100)
                            } else {
                                egui::Color32::WHITE
                            };
                            painter.vline(
                                progress_x,
                                rect.min.y..=rect.max.y,
                                egui::Stroke::new(2.0, marker_color),
                            );

                            ui.add_space(10.0);

                            ui.horizontal(|ui| {
                                let display_pos = if let Some(pending) = self.pending_seek {
                                    Duration::from_secs_f32(duration.as_secs_f32() * pending)
                                } else {
                                    self.get_current_position().unwrap_or(Duration::from_secs(0))
                                };

                                let time_color = if self.pending_seek.is_some() {
                                    egui::Color32::from_rgb(255, 200, 100)
                                } else {
                                    egui::Color32::WHITE
                                };

                                ui.label(
                                    egui::RichText::new(format_duration(display_pos))
                                        .size(18.0)
                                        .color(time_color)
                                        .monospace()
                                );

                                ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
                                    ui.add_space(20.0);
                                    ui.label(
                                        egui::RichText::new(format_duration(duration))
                                            .size(18.0)
                                            .color(egui::Color32::WHITE)
                                            .monospace()
                                    );
                                });
                            });
                            ui.add_space(10.0);
                        });

                        ui.add_space(15.0);

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

                            let fill_height = slider_height * self.volume;
                            let fill_rect = egui::Rect::from_min_size(
                                egui::pos2(rect.min.x, rect.max.y - fill_height),
                                egui::vec2(slider_width, fill_height),
                            );
                            painter.rect_filled(
                                fill_rect,
                                0.0,
                                self.primary_color(),
                            );

                            if response.dragged() || response.clicked() {
                                if let Some(pos) = response.interact_pointer_pos() {
                                    let relative_y = (rect.max.y - pos.y).max(0.0).min(slider_height);
                                    let new_volume = (relative_y / slider_height).clamp(0.0, 1.0);
                                    self.volume = new_volume;
                                    if let Ok(player) = self.player.lock() {
                                        if let Some(state) = player.as_ref() {
                                            state.sink.set_volume(self.volume);
                                        }
                                    }
                                }
                            }

                            ui.add_space(8.0);

                            ui.label(
                                egui::RichText::new(format!("{:.0}", self.volume * 100.0))
                                    .size(12.0)
                                    .color(egui::Color32::WHITE)
                            );
                        });

                        ui.add_space(40.0);
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
            });

        if let Some(folder_name) = &mut self.new_folder_prompt {
            let (confirmed, cancelled) = show_text_prompt(
                ctx,
                "folder name...",
                folder_name,
            );

            if confirmed {
                let new_path = self.current_dir.join(folder_name.clone());
                if let Err(e) = fs::create_dir(&new_path) {
                    eprintln!("Failed to create folder: {}", e);
                } else {
                    self.update_columns();
                }
                self.new_folder_prompt = None;
            }

            if cancelled {
                self.new_folder_prompt = None;
            }
        }

        if let Some((old_path, new_name)) = &mut self.rename_prompt {
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
                        self.update_columns();
                    }
                } else {
                    eprintln!("Failed to rename: no parent directory");
                }
                self.rename_prompt = None;
            }

            if cancelled {
                self.rename_prompt = None;
            }
        }

        if let Some(delete_path) = &self.delete_confirm_prompt {
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
            let (confirmed, cancelled) = show_confirm_prompt(
                ctx,
                "Confirm Delete",
                &message
            );

            if confirmed {
                let result = if is_dir {
                    fs::remove_dir_all(&delete_path_clone)
                } else {
                    fs::remove_file(&delete_path_clone)
                };

                if let Err(e) = result {
                    eprintln!("Failed to delete: {}", e);
                } else {
                    let was_playing = {
                        let player = self.player.lock().unwrap();
                        if let Some(state) = player.as_ref() {
                            state.current_file == delete_path_clone
                        } else {
                            false
                        }
                    };

                    if was_playing {
                        let mut player = self.player.lock().unwrap();
                        *player = None;
                    }

                    self.update_columns();

                    if let Some((clipboard_path, _)) = &self.clipboard {
                        if clipboard_path == &delete_path_clone {
                            self.clipboard = None;
                        }
                    }
                }
                self.delete_confirm_prompt = None;
            }

            if cancelled {
                self.delete_confirm_prompt = None;
            }
        }

        if let Some(editor) = &mut self.metadata_editor {
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
                        fill: egui::Color32::BLACK,
                        stroke: egui::Stroke::new(1.0, egui::Color32::from_rgb(80, 80, 80)),
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

                            ui.add_space(15.0);

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

                            ui.add_space(15.0);

                            ui.horizontal(|ui| {
                                if let Some(cover_path) = &editor.cover_path {
                                    let filename = std::path::Path::new(cover_path)
                                        .file_name()
                                        .and_then(|n| n.to_str())
                                        .unwrap_or("unknown");
                                    ui.label(
                                        egui::RichText::new(format!("📎 {}", filename))
                                            .size(12.0)
                                            .color(egui::Color32::from_rgb(150, 150, 150))
                                    );
                                    if ui.button("✕").clicked() {
                                        editor.cover_path = None;
                                    }
                                } else {
                                    ui.label(
                                        egui::RichText::new("No cover selected")
                                            .size(12.0)
                                            .color(egui::Color32::from_rgb(100, 100, 100))
                                    );
                                }
                            });

                            ui.add_space(5.0);

                            if ui.button("Select Cover Image...").clicked() {
                                if let Some(path) = rfd::FileDialog::new()
                                    .add_filter("Images", &["png", "jpg", "jpeg"])
                                    .pick_file()
                                {
                                    editor.cover_path = Some(path.to_string_lossy().to_string());
                                }
                            }

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

            if save_metadata {
                let file_path = editor.file_path.clone();
                let title = editor.title.clone();
                let artist = editor.artist.clone();
                let date = editor.date.clone();
                let cover_path = editor.cover_path.clone();

                match save_audio_metadata(&file_path, &title, &artist, &date, cover_path.as_deref()) {
                    Err(e) => {
                        eprintln!("Failed to save metadata: {}", e);
                        editor.error_message = Some(format!("Error: {}", e));
                    }
                    Ok(()) => {
                        eprintln!("Metadata saved successfully, refreshing UI...");

                        self.album_cover_cache.remove(&file_path);
                        self.last_selected_file = None;

                        if let Ok(mut player) = self.player.lock() {
                            if let Some(state) = player.as_mut() {
                                if state.current_file == file_path {
                                    state.title = title;
                                    state.artist = Some(artist).filter(|a| !a.is_empty());
                                    state.album_cover = None;
                                }
                            }
                        }

                        close_editor = true;
                    }
                }
            }

            if close_editor {
                self.metadata_editor = None;
            }
        }

        if self.search_open {
            let window_response = egui::Window::new("")
                .anchor(egui::Align2::CENTER_CENTER, [0.0, 0.0])
                .collapsible(false)
                .resizable(false)
                .title_bar(false)
                .fixed_size([700.0, 500.0])
                .frame(egui::Frame {
                    fill: egui::Color32::TRANSPARENT,
                    stroke: egui::Stroke::NONE,
                    ..Default::default()
                })
                .show(ctx, |ui| {
                    egui::Frame {
                        fill: egui::Color32::BLACK,
                        stroke: egui::Stroke::new(1.0, egui::Color32::from_rgb(80, 80, 80)),
                        inner_margin: egui::Margin::same(12.0),
                        ..Default::default()
                    }
                    .show(ui, |ui| {
                        let response = ui.add_sized(
                            [ui.available_width(), 20.0],
                            egui::TextEdit::singleline(&mut self.search_query)
                                .font(egui::TextStyle::Monospace)
                                .hint_text("search...")
                                .frame(false)
                        );
                        response.request_focus();

                        if response.changed() {
                            self.perform_search();
                        }

                        if ui.input(|i| i.key_pressed(egui::Key::Escape)) {
                            self.search_open = false;
                            self.search_query.clear();
                            self.search_results.clear();
                        }
                    });

                    ui.add_space(10.0);

                    if !self.search_results.is_empty() {
                        if ui.input(|i| i.key_pressed(egui::Key::ArrowDown) || i.key_pressed(egui::Key::J)) {
                            if self.search_selected < self.search_results.len().saturating_sub(1) {
                                self.search_selected += 1;
                            }
                        }
                        if ui.input(|i| i.key_pressed(egui::Key::ArrowUp) || i.key_pressed(egui::Key::K)) {
                            if self.search_selected > 0 {
                                self.search_selected -= 1;
                            }
                        }

                        let mut selected_result: Option<SearchResult> = None;

                        if ui.input(|i| i.key_pressed(egui::Key::Enter)) {
                            if let Some(result) = self.search_results.get(self.search_selected) {
                                selected_result = Some(SearchResult {
                                    path: result.path.clone(),
                                    filename: result.filename.clone(),
                                    title: result.title.clone(),
                                    artist: result.artist.clone(),
                                    album: result.album.clone(),
                                });
                            }
                        }

                        egui::Frame {
                            fill: egui::Color32::BLACK,
                            stroke: egui::Stroke::new(1.0, egui::Color32::from_rgb(80, 80, 80)),
                            inner_margin: egui::Margin::same(8.0),
                            ..Default::default()
                        }
                        .show(ui, |ui| {
                            egui::ScrollArea::vertical()
                                .max_height(410.0)
                                .show(ui, |ui| {
                                    for (idx, result) in self.search_results.iter().enumerate() {
                                        let is_selected = idx == self.search_selected;

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
                                            egui::vec2(ui.available_width(), 32.0),
                                            egui::Sense::click()
                                        );

                                        if response.clicked() {
                                            selected_result = Some(SearchResult {
                                                path: result.path.clone(),
                                                filename: result.filename.clone(),
                                                title: result.title.clone(),
                                                artist: result.artist.clone(),
                                                album: result.album.clone(),
                                            });
                                        }

                                        if is_selected {
                                            ui.painter().rect_filled(
                                                rect,
                                                0.0,
                                                egui::Color32::WHITE,
                                            );
                                        } else if response.hovered() {
                                            ui.painter().rect_filled(
                                                rect,
                                                0.0,
                                                egui::Color32::from_rgb(30, 30, 30),
                                            );
                                        }

                                        let text_color = if is_selected {
                                            egui::Color32::BLACK
                                        } else {
                                            egui::Color32::from_rgb(200, 200, 200)
                                        };

                                        ui.painter().text(
                                            rect.left_top() + egui::vec2(8.0, 8.0),
                                            egui::Align2::LEFT_TOP,
                                            &full_text,
                                            egui::FontId::monospace(16.0),
                                            text_color,
                                        );
                                    }
                                });
                        });

                        if let Some(result) = selected_result {
                            if let Some(parent) = result.path.parent() {
                                self.current_dir = parent.to_path_buf();
                                self.update_columns();

                                if let Some(idx) = self.columns[0].entries.iter().position(|e| e.path == result.path) {
                                    self.columns[0].selected = idx;
                                }

                                self.play_file(&result.path, ctx);
                            }

                            self.search_open = false;
                            self.search_query.clear();
                            self.search_results.clear();
                        }
                    }
                });

            if let Some(response) = window_response {
                if self.search_just_opened {
                    self.search_just_opened = false;
                } else if ctx.input(|i| i.pointer.primary_released()) {
                    if let Some(pos) = ctx.input(|i| i.pointer.interact_pos()) {
                        if !response.response.rect.contains(pos) {
                            self.search_open = false;
                            self.search_query.clear();
                            self.search_results.clear();
                        }
                    }
                }
            }
        }

        if let Some((path, pos)) = &self.context_menu.clone() {
            let is_dir = path.is_dir();
            if let Some(action) = show_context_menu(ctx, path, *pos, is_dir) {
                match action {
                    ContextMenuAction::Rename => {
                        let name = path.file_name()
                            .unwrap_or_default()
                            .to_string_lossy()
                            .to_string();
                        self.rename_prompt = Some((path.clone(), name));
                    }
                    ContextMenuAction::Delete => {
                        self.delete_confirm_prompt = Some(path.clone());
                    }
                    ContextMenuAction::Copy => {
                        self.clipboard = Some((path.clone(), ClipboardOperation::Copy));
                    }
                    ContextMenuAction::Cut => {
                        self.clipboard = Some((path.clone(), ClipboardOperation::Cut));
                    }
                    ContextMenuAction::ToggleFavorite => {
                        if let Some(idx) = self.favorites.iter().position(|f| f.path == *path) {
                            self.favorites.remove(idx);
                        } else {
                            let name = path.file_name()
                                .unwrap_or_default()
                                .to_string_lossy()
                                .to_string();
                            self.favorites.push(Favorite {
                                path: path.clone(),
                                name,
                                is_dir,
                                timestamp: SystemTime::now(),
                            });
                        }
                        let _ = favorites::save(&self.favorites);
                    }
                    ContextMenuAction::EditMetadata => {
                        let (title, artist, _album, date, _track, _duration) = extract_metadata(path);
                        self.metadata_editor = Some(MetadataEditor {
                            file_path: path.clone(),
                            title,
                            artist: artist.unwrap_or_default(),
                            date: date.unwrap_or_default(),
                            cover_path: None,
                            just_opened: true,
                            error_message: None,
                        });
                    }
                }
                self.context_menu = None;
            }

            if ctx.input(|i| i.pointer.primary_released()) {
                self.context_menu = None;
            }
        }
    }
}
