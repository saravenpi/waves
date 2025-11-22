use eframe::egui;
use std::fs;
use std::path::PathBuf;
use std::time::{Duration, SystemTime};

use crate::app::WavesApp;
use crate::config::SidebarPosition;
use crate::types::{FileEntry, Favorite, ClipboardOperation, SidebarView};
use crate::ui::helpers::{ContextMenuAction, show_text_prompt, show_context_menu};
use crate::ui::input::MetadataEditor;
use crate::ui::components::{ConfirmDialog, IconButton, Select};
use crate::utils::{format_duration, truncate_text};
use crate::metadata::{extract_metadata, save_audio_metadata};
use crate::favorites;
use crate::file_operations::SearchResult;

impl eframe::App for WavesApp {
    fn update(&mut self, ctx: &egui::Context, _frame: &mut eframe::Frame) {
        if self.startup_animation {
            let min_time_elapsed = self.startup_time.elapsed().as_secs_f32() >= 1.2;

            if min_time_elapsed {
                self.startup_animation = false;
            } else {
                let opacity = (self.config.window_opacity / 100.0 * 255.0) as u8;
                let bg_color = egui::Color32::from_rgba_unmultiplied(0, 0, 0, opacity);

                egui::CentralPanel::default()
                    .frame(egui::Frame::none().fill(bg_color))
                    .show(ctx, |ui| {
                        ui.vertical_centered(|ui| {
                            let available_height = ui.available_height();
                            ui.add_space(available_height * 0.35);

                            ui.label(
                                egui::RichText::new("Waves")
                                    .size(72.0)
                                    .color(self.primary_color())
                                    .strong()
                            );

                            ui.add_space(40.0);

                            crate::ui::spinner::square_spinner(ui, 60.0, self.primary_color());

                            ui.add_space(20.0);

                            ui.label(
                                egui::RichText::new("Music Player")
                                    .size(18.0)
                                    .color(egui::Color32::from_gray(180))
                            );
                        });
                    });

                ctx.request_repaint();
                return;
            }
        }

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

        let mut cache_updates = Vec::new();
        if let Some(ref receiver) = self.cache_receiver {
            while let Ok(result) = receiver.try_recv() {
                cache_updates.push(result);
            }
        }

        for result in cache_updates {
            match result {
                crate::app::CacheResult::AudioFiles(files) => {
                    self.audio_files_cache = Some(files);
                }
                crate::app::CacheResult::ArtistGroups(groups) => {
                    self.artist_groups_cache = Some(groups);
                }
                crate::app::CacheResult::AlbumGroups(groups) => {
                    self.album_groups_cache = Some(groups);
                }
            }
            self.update_columns();
            ctx.request_repaint();
        }

        while let Ok(status) = self.update_checker.status_receiver.try_recv() {
            match &status {
                crate::update::UpdateStatus::Checking => {
                    self.update_in_progress = true;
                }
                crate::update::UpdateStatus::UpdateAvailable(version) => {
                    self.update_available_version = Some(version.clone());
                    self.show_update_dialog = true;
                    self.update_in_progress = false;
                }
                crate::update::UpdateStatus::NoUpdateAvailable => {
                    self.update_in_progress = false;
                }
                crate::update::UpdateStatus::Downloading(_) => {
                    self.update_in_progress = true;
                }
                crate::update::UpdateStatus::Installing => {
                    self.update_in_progress = true;
                }
                crate::update::UpdateStatus::Success => {
                    self.update_in_progress = false;
                }
                crate::update::UpdateStatus::Error(_) => {
                    self.update_in_progress = false;
                }
            }
            self.current_update_status = Some(status);
            ctx.request_repaint();
        }

        if ctx.input(|i| i.pointer.is_moving()) {
            self.last_mouse_movement = std::time::Instant::now();
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
                        if !self.animation_fullscreen {
                            return;
                        } else {
                            (std::sync::Arc::new(std::sync::Mutex::new(std::collections::VecDeque::new())), 44100, 2)
                        }
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

        let mut keys_to_handle = Vec::new();
        let mut dropped_files = Vec::new();

        // Check if search bar has focus
        let search_has_focus = ctx.memory(|m| m.has_focus(egui::Id::new("main_search_bar")));

        ctx.input(|i| {
            for event in &i.events {
                if let egui::Event::Key { key, pressed: true, modifiers, .. } = event {
                    if self.metadata_editor.is_some() || self.new_folder_prompt.is_some() ||
                       self.rename_prompt.is_some() || search_has_focus {
                        continue;
                    }

                    // Handle help modal with ? key (Shift+Slash)
                    if *key == egui::Key::Slash && modifiers.shift && !self.animation_fullscreen {
                        self.help_modal_open = !self.help_modal_open;
                    } else if *key == egui::Key::Slash && !modifiers.command && !modifiers.ctrl && !self.animation_fullscreen {
                        self.search_just_opened = true;
                    } else {
                        keys_to_handle.push(*key);
                    }
                }
            }

            if !i.raw.dropped_files.is_empty() {
                dropped_files = i.raw.dropped_files.clone();
            }
        });

        // Consume Tab key to prevent egui from using it for focus navigation
        ctx.input_mut(|i| {
            i.consume_key(egui::Modifiers::NONE, egui::Key::Tab);
            i.consume_key(egui::Modifiers::SHIFT, egui::Key::Tab);
        });

        for key in keys_to_handle {
            self.handle_navigation(key, ctx);
        }

            if !dropped_files.is_empty() {
                for file in &dropped_files {
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

        if self.config.show_status_bar {
            let status_opacity = ((self.config.window_opacity / 100.0) * 255.0).min(255.0) as u8;

            egui::TopBottomPanel::bottom("status")
                .frame(egui::Frame::default().fill(egui::Color32::from_black_alpha(status_opacity)))
                .show(ctx, |ui| {
                    ui.separator();
                    let volume_percent = (self.volume * 100.0) as i32;
                    let status_text = format!(" h/j/k/l: navigate | ENTER: select/play | SPACE: pause | ←/→: prev/next | TAB: view | ↑/↓: vol ({}%) | ?: help", volume_percent);
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

                        let expand_response = IconButton::new(expand_arrow).size(20.0).show(ui);
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

                ui.add_space(10.0);
                ui.allocate_ui_with_layout(
                    egui::vec2(ui.available_width(), 35.0),
                    egui::Layout::left_to_right(egui::Align::Center),
                    |ui| {
                        ui.heading(egui::RichText::new("Waves").size(32.0).color(egui::Color32::WHITE).strong());

                        ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
                            let collapse_response = IconButton::new(collapse_arrow).size(20.0).show(ui);
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
                    }
                );
                ui.add_space(10.0);

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
                        use crate::types::GroupedView;

                        let mut clicked_sidebar_view = None;

                        let sidebar_options = vec![
                            ("📁".to_string(), "Browser".to_string()),
                            ("⭐".to_string(), "Favorites".to_string()),
                            ("⚙".to_string(), "Settings".to_string()),
                        ];

                        let sidebar_index = match self.sidebar_view {
                            SidebarView::FileBrowser => 0,
                            SidebarView::Favorites => 1,
                            SidebarView::Settings => 2,
                        };

                        let primary_color = self.primary_color();

                        ui.allocate_ui_with_layout(
                            egui::vec2(ui.available_width(), 30.0),
                            egui::Layout::left_to_right(egui::Align::Center),
                            |ui| {
                                ui.add_space(2.0);

                                let (_, clicked) = Select::new(sidebar_options, sidebar_index)
                                    .show(ui, primary_color);
                                clicked_sidebar_view = clicked;
                            });

                        if let Some(idx) = clicked_sidebar_view {
                            match idx {
                                0 => self.sidebar_view = SidebarView::FileBrowser,
                                1 => self.sidebar_view = SidebarView::Favorites,
                                2 => self.sidebar_view = SidebarView::Settings,
                                _ => {}
                            };
                        }
                        ui.add_space(5.0);
                        ui.separator();
                        ui.add_space(5.0);

                        let mut header_height = 75.0;

                        match self.sidebar_view {
                            SidebarView::FileBrowser => {
                                use crate::types::BrowsingMode;

                                let folder_name = match self.browsing_mode {
                                    BrowsingMode::ByArtist | BrowsingMode::ByAlbum => {
                                        match &self.grouped_view {
                                            GroupedView::TrackList(group_name) => {
                                                group_name.trim_start_matches("🎤 ").trim_start_matches("💿 ").to_string()
                                            }
                                            GroupedView::GroupList => {
                                                if self.browsing_mode == BrowsingMode::ByArtist {
                                                    "All Artists".to_string()
                                                } else {
                                                    "All Albums".to_string()
                                                }
                                            }
                                        }
                                    }
                                    BrowsingMode::AllSongs => "All Songs".to_string(),
                                    BrowsingMode::FileStructure => {
                                        self.current_dir.file_name()
                                            .and_then(|n| n.to_str())
                                            .unwrap_or("/")
                                            .to_string()
                                    }
                                };

                                let show_back = match self.browsing_mode {
                                    BrowsingMode::FileStructure => self.current_dir != self.root_dir,
                                    BrowsingMode::ByArtist | BrowsingMode::ByAlbum => {
                                        matches!(self.grouped_view, GroupedView::TrackList(_))
                                    }
                                    BrowsingMode::AllSongs => false,
                                };

                                ui.horizontal(|ui| {
                                    ui.add_space(2.0);

                                    if show_back {
                                        let text_color = egui::Color32::from_rgb(150, 150, 150);
                                        let back_response = IconButton::new("<").size(14.0).color(text_color).show(ui);

                                        if back_response.clicked() {
                                            back_button_clicked = true;
                                        }

                                        if back_response.hovered() {
                                            ui.painter().rect_stroke(
                                                back_response.rect,
                                                0.0,
                                                egui::Stroke::new(1.0, egui::Color32::WHITE),
                                            );
                                        }

                                        ui.add_space(5.0);
                                    }

                                    ui.label(
                                        egui::RichText::new(folder_name)
                                            .size(14.0)
                                            .color(egui::Color32::from_rgb(150, 150, 150))
                                    );
                                });

                                ui.add_space(5.0);
                                ui.separator();
                                ui.add_space(5.0);

                                let browsing_options = vec![
                                    ("📂".to_string(), "Folders".to_string()),
                                    ("🎤".to_string(), "Artists".to_string()),
                                    ("💿".to_string(), "Albums".to_string()),
                                    ("🎵".to_string(), "All Songs".to_string()),
                                ];

                                let browsing_index = match self.browsing_mode {
                                    BrowsingMode::FileStructure => 0,
                                    BrowsingMode::ByArtist => 1,
                                    BrowsingMode::ByAlbum => 2,
                                    BrowsingMode::AllSongs => 3,
                                };

                                let (_, clicked_browsing) = Select::new(browsing_options, browsing_index)
                                    .show(ui, self.primary_color());

                                if let Some(idx) = clicked_browsing {
                                    let new_mode = match idx {
                                        0 => BrowsingMode::FileStructure,
                                        1 => BrowsingMode::ByArtist,
                                        2 => BrowsingMode::ByAlbum,
                                        3 => BrowsingMode::AllSongs,
                                        _ => self.browsing_mode,
                                    };
                                    if new_mode != self.browsing_mode {
                                        self.browsing_mode = new_mode;
                                        self.grouped_view = crate::types::GroupedView::GroupList;
                                        self.current_group_tracks.clear();
                                        self.update_columns_with_selection(Some(0));
                                    }
                                }

                                ui.add_space(5.0);
                                header_height = 110.0;
                            }
                            _ => {}
                        }

                        let list_height = browser_height - header_height;

                        match self.sidebar_view {
                            SidebarView::FileBrowser => {
                        if !self.columns.is_empty() {
                            let column = &self.columns[0];

                            let scroll_area = egui::ScrollArea::vertical()
                                .auto_shrink([false, false])
                                .min_scrolled_height(list_height)
                                .max_height(list_height)
                                .scroll_bar_visibility(egui::scroll_area::ScrollBarVisibility::VisibleWhenNeeded)
                                .id_salt("file_browser_scroll");

                            scroll_area.show(ui, |ui| {
                                    let available_width = ui.available_width();
                                    let max_chars = ((available_width - 10.0) / 10.5) as usize;

                                    if !column.entries.is_empty()
                                        && (column.entries[0].name.starts_with("Loading ")) {
                                        let loading_text = &column.entries[0].name;
                                        ui.add_space(50.0);
                                        crate::ui::spinner::square_spinner_with_text(ui, loading_text, self.primary_color());
                                        return;
                                    }

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

                                        // Auto-scroll to selected item on keyboard navigation
                                        if is_selected {
                                            ui.scroll_to_rect(rect, Some(egui::Align::Center));
                                        }

                                        if response.clicked() {
                                            clicked_entry = Some((idx, entry.clone()));
                                        }

                                        if response.secondary_clicked() {
                                            if let Some(pos) = response.interact_pointer_pos() {
                                                context_menu_event = Some((entry.path.clone(), pos));
                                            }
                                        }

                                        let is_hovered = response.hovered();

                                        let name_has_emoji = entry.name.starts_with("🎤 ")
                                            || entry.name.starts_with("💿 ")
                                            || entry.name.starts_with("🎵 ");

                                        let icon = if name_has_emoji {
                                            ""
                                        } else if entry.is_dir {
                                            "📁"
                                        } else {
                                            "🎵"
                                        };

                                        let display_name = truncate_text(&entry.name, max_chars.saturating_sub(4));
                                        let is_favorited = self.favorites.iter().any(|f| f.path == entry.path);
                                        let star = if is_favorited { "* " } else { "" };
                                        let display_text = if icon.is_empty() {
                                            format!(" {}{}", star, display_name)
                                        } else {
                                            format!(" {}{} {}", star, icon, display_name)
                                        };

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

                                if favorites_clone.is_empty() {
                                    ui.vertical_centered(|ui| {
                                        ui.add_space(50.0);
                                        ui.label(
                                            egui::RichText::new("No favorites yet")
                                                .size(16.0)
                                                .color(egui::Color32::from_rgb(150, 150, 150))
                                        );
                                        ui.add_space(10.0);
                                        ui.label(
                                            egui::RichText::new("Press 'f' to add audio files to favorites")
                                                .size(14.0)
                                                .color(egui::Color32::from_rgb(120, 120, 120))
                                        );
                                    });
                                } else {
                                    egui::ScrollArea::vertical()
                                        .auto_shrink([false, false])
                                        .min_scrolled_height(list_height)
                                        .max_height(list_height)
                                        .scroll_bar_visibility(egui::scroll_area::ScrollBarVisibility::VisibleWhenNeeded)
                                        .id_salt("favorites_scroll")
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

                                            // Auto-scroll to selected item on keyboard navigation
                                            if is_selected {
                                                ui.scroll_to_rect(rect, Some(egui::Align::Center));
                                            }

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
                            }
                            SidebarView::Settings => {
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
                                                ui.ctx().send_viewport_cmd(egui::ViewportCommand::Transparent(opacity < 100.0));
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
                                                ui.label(egui::RichText::new("Show Title Bar").size(16.0).color(egui::Color32::WHITE));

                                                ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
                                                    let toggle_width = 50.0;
                                                    let toggle_height = 25.0;
                                                    let (rect, response) = ui.allocate_exact_size(
                                                        egui::vec2(toggle_width, toggle_height),
                                                        egui::Sense::click()
                                                    );

                                                    if response.clicked() {
                                                        self.config.decorations = !self.config.decorations;
                                                        let _ = self.config.save();
                                                        ui.ctx().send_viewport_cmd(egui::ViewportCommand::Decorations(self.config.decorations));
                                                    }

                                                    let bg_color = if self.config.decorations {
                                                        self.primary_color()
                                                    } else {
                                                        egui::Color32::from_rgb(60, 60, 60)
                                                    };

                                                    ui.painter().rect_filled(rect, 0.0, bg_color);

                                                    let square_size = 20.0;
                                                    let square_x = if self.config.decorations {
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
                                                            .stroke(egui::Stroke::NONE)
                                                            .rounding(0.0);

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
                                                    .stroke(egui::Stroke::NONE)
                                                    .rounding(0.0);

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
                                                    .stroke(egui::Stroke::NONE)
                                                    .rounding(0.0);

                                                if ui.add(right_button).clicked() {
                                                    self.config.sidebar_position = SidebarPosition::Right;
                                                    let _ = self.config.save();
                                                }
                                            });
                                        });
                                        ui.add_space(15.0);

                                        ui.separator();
                                        ui.add_space(15.0);

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

                                        let is_focused = self.settings_focused_item == 9;
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
                                                ui.label(egui::RichText::new("Startup Sound").size(16.0).color(egui::Color32::WHITE));

                                                ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
                                                    let toggle_width = 50.0;
                                                    let toggle_height = 25.0;
                                                    let (rect, response) = ui.allocate_exact_size(
                                                        egui::vec2(toggle_width, toggle_height),
                                                        egui::Sense::click()
                                                    );

                                                    if response.clicked() {
                                                        self.config.startup_sound_enabled = !self.config.startup_sound_enabled;
                                                        let _ = self.config.save();
                                                    }

                                                    let bg_color = if self.config.startup_sound_enabled {
                                                        self.primary_color()
                                                    } else {
                                                        egui::Color32::from_rgb(60, 60, 60)
                                                    };

                                                    ui.painter().rect_filled(rect, 0.0, bg_color);

                                                    let square_size = 20.0;
                                                    let square_x = if self.config.startup_sound_enabled {
                                                        rect.max.x - square_size - 2.5
                                                    } else {
                                                        rect.min.x + 2.5
                                                    };
                                                    let square_y = rect.min.y + (rect.height() - square_size) / 2.0;

                                                    let square_rect = egui::Rect::from_min_size(
                                                        egui::pos2(square_x, square_y),
                                                        egui::vec2(square_size, square_size),
                                                    );

                                                    ui.painter().rect_filled(
                                                        square_rect,
                                                        0.0,
                                                        egui::Color32::WHITE,
                                                    );
                                                });
                                            });
                                        });

                                        ui.add_space(15.0);

                                        ui.separator();
                                        ui.add_space(15.0);

                                        let is_focused = self.settings_focused_item == 10;
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
                                            ui.add_space(3.0);
                                            ui.label(egui::RichText::new("The folder that opens when launching WAVES").size(12.0).color(egui::Color32::from_rgb(140, 140, 140)));
                                            ui.add_space(10.0);

                                            let has_custom_folder = !self.default_folder_input.is_empty();

                                            egui::Frame::default()
                                                .fill(egui::Color32::from_rgb(30, 30, 30))
                                                .inner_margin(egui::Margin::same(10.0))
                                                .rounding(4.0)
                                                .show(ui, |ui| {
                                                    ui.horizontal(|ui| {
                                                        ui.label(egui::RichText::new("📁").size(16.0));
                                                        ui.add_space(8.0);
                                                        if has_custom_folder {
                                                            ui.label(egui::RichText::new(&self.default_folder_input).size(14.0).color(egui::Color32::WHITE));
                                                        } else {
                                                            let default_path = dirs::audio_dir()
                                                                .map(|p| p.to_string_lossy().to_string())
                                                                .unwrap_or_else(|| "~/Music".to_string());
                                                            ui.label(egui::RichText::new(format!("{} (system default)", default_path)).size(14.0).color(egui::Color32::from_rgb(140, 140, 140)));
                                                        }
                                                    });
                                                });

                                            ui.add_space(10.0);
                                            ui.horizontal(|ui| {
                                                if ui.button("Choose Folder").clicked() {
                                                    if let Some(path) = rfd::FileDialog::new().pick_folder() {
                                                        let path_str = path.to_string_lossy().to_string();
                                                        #[cfg(not(target_os = "windows"))]
                                                        let path_str = {
                                                            if let Some(home) = dirs::home_dir() {
                                                                let home_str = home.to_string_lossy();
                                                                if path_str.starts_with(&*home_str) {
                                                                    path_str.replacen(&*home_str, "~", 1)
                                                                } else {
                                                                    path_str
                                                                }
                                                            } else {
                                                                path_str
                                                            }
                                                        };
                                                        self.default_folder_input = path_str.clone();
                                                        self.config.default_folder = Some(path_str);
                                                        let _ = self.config.save();
                                                    }
                                                }
                                                if has_custom_folder {
                                                    if ui.button("Reset to Default").clicked() {
                                                        self.default_folder_input.clear();
                                                        self.config.default_folder = None;
                                                        let _ = self.config.save();
                                                    }
                                                }
                                            });
                                        });

                                        ui.separator();
                                        ui.add_space(15.0);

                                        let is_focused = self.settings_focused_item == 11;
                                        let frame = if is_focused {
                                            egui::Frame::default()
                                                .stroke(egui::Stroke::new(2.0, egui::Color32::WHITE))
                                                .inner_margin(egui::Margin::same(8.0))
                                                .rounding(0.0)
                                        } else {
                                            egui::Frame::default().inner_margin(egui::Margin::same(8.0))
                                        };
                                        frame.show(ui, |ui| {
                                            ui.label(egui::RichText::new("Updates").size(16.0).color(egui::Color32::WHITE));
                                            ui.add_space(5.0);
                                            ui.horizontal(|ui| {
                                                ui.add_enabled_ui(!self.update_in_progress, |ui| {
                                                    if ui.button("Check for Updates").clicked() {
                                                        self.update_checker.check_for_updates();
                                                    }
                                                });

                                                if let Some(ref status) = self.current_update_status {
                                                    match status {
                                                        crate::update::UpdateStatus::Checking => {
                                                            ui.label(egui::RichText::new("Checking...").size(14.0).color(egui::Color32::from_rgb(200, 200, 200)));
                                                        }
                                                        crate::update::UpdateStatus::UpdateAvailable(version) => {
                                                            ui.label(egui::RichText::new(format!("Update available: v{}", version)).size(14.0).color(egui::Color32::from_rgb(100, 200, 100)));
                                                        }
                                                        crate::update::UpdateStatus::NoUpdateAvailable => {
                                                            ui.label(egui::RichText::new("You're up to date!").size(14.0).color(egui::Color32::from_rgb(100, 200, 100)));
                                                        }
                                                        crate::update::UpdateStatus::Downloading(progress) => {
                                                            ui.label(egui::RichText::new(format!("Downloading... {}%", progress)).size(14.0).color(egui::Color32::from_rgb(200, 200, 100)));
                                                        }
                                                        crate::update::UpdateStatus::Installing => {
                                                            ui.label(egui::RichText::new("Installing...").size(14.0).color(egui::Color32::from_rgb(200, 200, 100)));
                                                        }
                                                        crate::update::UpdateStatus::Success => {
                                                            ui.label(egui::RichText::new("Update successful! Please restart.").size(14.0).color(egui::Color32::from_rgb(100, 200, 100)));
                                                        }
                                                        crate::update::UpdateStatus::Error(msg) => {
                                                            ui.label(egui::RichText::new(format!("Error: {}", msg)).size(14.0).color(egui::Color32::from_rgb(200, 100, 100)));
                                                        }
                                                    }
                                                }
                                            });
                                            ui.label(egui::RichText::new(format!("Current version: v{}", env!("CARGO_PKG_VERSION"))).size(12.0).color(egui::Color32::from_rgb(140, 140, 140)));
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
                    use crate::types::{BrowsingMode, GroupedView};

                    match self.browsing_mode {
                        BrowsingMode::FileStructure => {
                            if let Some(parent) = self.current_dir.parent() {
                                if parent >= self.root_dir.as_path() {
                                    self.current_dir = parent.to_path_buf();
                                    self.update_columns_with_selection(Some(0));
                                }
                            }
                        }
                        BrowsingMode::ByArtist | BrowsingMode::ByAlbum => {
                            if matches!(self.grouped_view, GroupedView::TrackList(_)) {
                                self.grouped_view = GroupedView::GroupList;
                                self.current_group_tracks.clear();
                                self.update_columns_with_selection(Some(0));
                            }
                        }
                        BrowsingMode::AllSongs => {}
                    }
                }

            });

        if !self.sidebar_collapsed {
            // Only update width when user is actively resizing (dragging)
            if ctx.input(|i| i.pointer.any_down()) {
                let new_width = sidebar_response.response.rect.width().max(250.0).min(800.0);
                if (new_width - self.config.sidebar_width).abs() > 1.0 {
                    self.config.sidebar_width = new_width;
                    let _ = self.config.save();
                }
            }
        }

        let central_opacity = ((self.config.window_opacity / 100.0) * 255.0).min(255.0) as u8;

        egui::CentralPanel::default()
            .frame(egui::Frame::default().fill(egui::Color32::from_black_alpha(central_opacity)))
            .show(ctx, |ui| {
                let primary_color = self.primary_color();

                if !self.animation_fullscreen {
                    ui.add_space(10.0);
                    ui.horizontal(|ui| {
                        ui.add_space(10.0);

                        // Calculate proper width for search bar with right padding
                        let search_bar_width = ui.available_width() - 10.0;

                        // Custom styled search bar with border
                        let search_frame = egui::Frame {
                            fill: egui::Color32::from_rgb(20, 20, 20),
                            stroke: egui::Stroke::new(1.0, primary_color),
                            inner_margin: egui::Margin::symmetric(8.0, 6.0),
                            rounding: egui::Rounding::same(0.0),
                            ..Default::default()
                        };

                        search_frame.show(ui, |ui| {
                            let search_response = ui.add_sized(
                                [search_bar_width - 16.0, 18.0], // Subtract inner margin
                                egui::TextEdit::singleline(&mut self.search_query)
                                    .hint_text("🔍 Search files...")
                                    .frame(false)
                                    .id(egui::Id::new("main_search_bar"))
                            );

                            // Handle focus when / was pressed
                            if self.search_just_opened {
                                search_response.request_focus();
                                self.search_just_opened = false;
                            }

                            // Remove / character if it was typed when opening search
                            if self.search_query.starts_with('/') {
                                self.search_query = self.search_query[1..].to_string();
                            }

                            if !self.search_query.is_empty() {
                                self.perform_search();
                            } else {
                                self.search_results.clear();
                                self.search_selected = 0;
                            }
                        });

                        ui.add_space(10.0);
                    });

                    // Display search results inline below the search bar
                    if !self.search_results.is_empty() {
                        ui.add_space(5.0);

                        // Handle keyboard navigation when search has focus or results are showing
                        if search_has_focus || !self.search_results.is_empty() {
                            if ui.input(|i| i.key_pressed(egui::Key::ArrowDown)) {
                                let max_display = self.search_results.len().min(5).saturating_sub(1);
                                if self.search_selected < max_display {
                                    self.search_selected += 1;
                                }
                            }
                            if ui.input(|i| i.key_pressed(egui::Key::ArrowUp)) {
                                if self.search_selected > 0 {
                                    self.search_selected -= 1;
                                }
                            }
                            if ui.input(|i| i.key_pressed(egui::Key::Enter)) {
                                if let Some(result) = self.search_results.get(self.search_selected) {
                                    let path = result.path.clone();
                                    self.play_file(&path, ctx);
                                    self.search_query.clear();
                                    self.search_results.clear();
                                    self.search_selected = 0;
                                }
                            }
                            if ui.input(|i| i.key_pressed(egui::Key::Escape)) {
                                self.search_query.clear();
                                self.search_results.clear();
                                self.search_selected = 0;
                            }
                        }

                        ui.horizontal(|ui| {
                            ui.add_space(10.0);

                            let mut clicked_result: Option<PathBuf> = None;

                            egui::Frame {
                                fill: egui::Color32::from_rgb(15, 15, 15),
                                stroke: egui::Stroke::new(1.0, egui::Color32::from_rgb(80, 80, 80)),
                                inner_margin: egui::Margin::same(8.0),
                                ..Default::default()
                            }
                            .show(ui, |ui| {
                                ui.style_mut().spacing.item_spacing = egui::vec2(0.0, 0.0);

                                // Only show top 5 results
                                let display_results = self.search_results.iter().take(5);

                                for (idx, result) in display_results.enumerate() {
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
                                        egui::vec2(ui.available_width() - 8.0, 28.0),
                                        egui::Sense::click()
                                    );

                                    if response.clicked() {
                                        clicked_result = Some(result.path.clone());
                                    }

                                    if is_selected {
                                        ui.painter().rect_filled(
                                            rect,
                                            0.0,
                                            egui::Color32::WHITE
                                        );
                                    }

                                    let text_color = if is_selected {
                                        egui::Color32::BLACK
                                    } else {
                                        egui::Color32::from_rgb(200, 200, 200)
                                    };

                                    ui.painter().text(
                                        rect.left_top() + egui::vec2(8.0, 6.0),
                                        egui::Align2::LEFT_TOP,
                                        &full_text,
                                        egui::FontId::proportional(13.0),
                                        text_color
                                    );
                                }
                            });

                            if let Some(path) = clicked_result {
                                self.play_file(&path, ctx);
                                self.search_query.clear();
                                self.search_results.clear();
                                self.search_selected = 0;
                            }

                            ui.add_space(10.0);
                        });

                        ui.add_space(5.0);
                    } else {
                        ui.add_space(10.0);
                    }
                }

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
                    let spectrum_height = total_height - bottom_panel_height - 45.0;

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

                                    let next_response = IconButton::new("⏭").show(ui);
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
                                    let play_pause_response = IconButton::new(pause_play_text).show(ui);
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
                                    let loop_response = IconButton::new("🔁").size(24.0).color(loop_color).show(ui);
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

                                    let prev_response = IconButton::new("⏮").show(ui);
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

            let (confirmed, cancelled, new_selected) = ConfirmDialog::new("Confirm Delete", &message)
                .confirm_text("Delete")
                .cancel_text("Cancel")
                .selected(self.delete_confirm_selected)
                .show(ctx, self.primary_color());

            self.delete_confirm_selected = new_selected;

            if confirmed {
                let result = if is_dir {
                    fs::remove_dir_all(&delete_path_clone)
                } else {
                    fs::remove_file(&delete_path_clone)
                };

                if let Err(e) = result {
                    eprintln!("Failed to delete: {}", e);
                } else {
                    crate::delete_sound::play_delete_sound();

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
                self.delete_confirm_selected = 1;
            }

            if cancelled {
                self.delete_confirm_prompt = None;
                self.delete_confirm_selected = 1;
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

            if save_metadata {
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

                match save_audio_metadata(&file_path, &title, &artist, &date, cover_path_to_use.as_deref()) {
                    Err(e) => {
                        eprintln!("Failed to save metadata: {}", e);
                        editor.error_message = Some(format!("Error: {}", e));
                    }
                    Ok(()) => {
                        eprintln!("Metadata saved successfully, refreshing UI...");

                        if let Some(temp_path) = cover_path_to_use {
                            if temp_path.contains("waves_temp_cover") {
                                let _ = std::fs::remove_file(&temp_path);
                            }
                        }

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

        if self.help_modal_open {
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
                        fill: egui::Color32::BLACK,
                        stroke: egui::Stroke::new(1.0, egui::Color32::from_rgb(80, 80, 80)),
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

                            ui.add_space(15.0);

                            egui::ScrollArea::vertical()
                                .max_height(500.0)
                                .show(ui, |ui| {
                                    let keybind = |ui: &mut egui::Ui, key: &str, desc: &str| {
                                        ui.horizontal(|ui| {
                                            ui.label(
                                                egui::RichText::new(key)
                                                    .size(14.0)
                                                    .color(self.primary_color())
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
                                    keybind(ui, "TAB", "Cycle views (Files → Favorites → Settings)");
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
                                    keybind(ui, "f", "Toggle favorite for selected item");
                                    keybind(ui, "m", "Edit metadata (audio files only)");
                                    keybind(ui, "/", "Search files");

                                    ui.add_space(10.0);
                                    ui.label(egui::RichText::new("View").size(16.0).color(egui::Color32::WHITE).strong());
                                    ui.add_space(5.0);
                                    keybind(ui, "b", "Toggle browse mode");
                                    keybind(ui, "?", "Show/hide this help");
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
                self.help_modal_open = false;
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
                            let max_display = self.search_results.len().min(5).saturating_sub(1);
                            if self.search_selected < max_display {
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
                                    relevance: result.relevance,
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
                            ui.style_mut().spacing.item_spacing = egui::vec2(0.0, 0.0);

                            // Only show top 5 results
                            let display_results = self.search_results.iter().take(5);

                            for (idx, result) in display_results.enumerate() {
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
                                    egui::vec2(ui.available_width() - 8.0, 32.0),
                                    egui::Sense::click()
                                );

                                if response.clicked() {
                                    selected_result = Some(SearchResult {
                                        path: result.path.clone(),
                                        filename: result.filename.clone(),
                                        title: result.title.clone(),
                                        artist: result.artist.clone(),
                                        album: result.album.clone(),
                                        relevance: result.relevance,
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
                        // Only allow favoriting audio files, not directories
                        if is_dir {
                            // Do nothing for directories
                        } else if let Some(idx) = self.favorites.iter().position(|f| f.path == *path) {
                            self.favorites.remove(idx);
                            let _ = favorites::save(&self.favorites);
                        } else {
                            let name = path.file_name()
                                .unwrap_or_default()
                                .to_string_lossy()
                                .to_string();
                            self.favorites.push(Favorite {
                                path: path.clone(),
                                name,
                                is_dir: false,
                                timestamp: SystemTime::now(),
                            });
                            let _ = favorites::save(&self.favorites);
                        }
                    }
                    ContextMenuAction::EditMetadata => {
                        let (title, artist, _album, date, _track, _duration) = extract_metadata(path);
                        let existing_cover_data = crate::album_cover::extract_album_cover(path);
                        let has_existing_cover = existing_cover_data.is_some();
                        self.metadata_editor = Some(MetadataEditor {
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
                self.context_menu = None;
            }

            if ctx.input(|i| i.pointer.primary_released()) {
                self.context_menu = None;
            }
        }

        if self.show_update_dialog {
            egui::Window::new("Update Available")
                .collapsible(false)
                .resizable(false)
                .anchor(egui::Align2::CENTER_CENTER, [0.0, 0.0])
                .show(ctx, |ui| {
                    ui.set_min_width(400.0);

                    if let Some(ref version) = self.update_available_version {
                        ui.label(
                            egui::RichText::new(format!("A new version of WAVES is available: v{}", version))
                                .size(16.0)
                                .color(egui::Color32::WHITE)
                        );
                        ui.add_space(10.0);
                        ui.label(
                            egui::RichText::new(format!("Current version: v{}", env!("CARGO_PKG_VERSION")))
                                .size(14.0)
                                .color(egui::Color32::from_gray(180))
                        );
                        ui.add_space(20.0);

                        if let Some(ref status) = self.current_update_status {
                            match status {
                                crate::update::UpdateStatus::Downloading(progress) => {
                                    ui.label(
                                        egui::RichText::new(format!("Downloading... {}%", progress))
                                            .size(14.0)
                                            .color(egui::Color32::from_rgb(200, 200, 100))
                                    );
                                    ui.add_space(10.0);
                                }
                                crate::update::UpdateStatus::Installing => {
                                    ui.label(
                                        egui::RichText::new("Installing update...")
                                            .size(14.0)
                                            .color(egui::Color32::from_rgb(200, 200, 100))
                                    );
                                    ui.add_space(10.0);
                                }
                                crate::update::UpdateStatus::Success => {
                                    ui.label(
                                        egui::RichText::new("Update successful! Please restart WAVES.")
                                            .size(14.0)
                                            .color(egui::Color32::from_rgb(100, 200, 100))
                                    );
                                    ui.add_space(10.0);
                                }
                                crate::update::UpdateStatus::Error(msg) => {
                                    ui.label(
                                        egui::RichText::new(format!("Error: {}", msg))
                                            .size(14.0)
                                            .color(egui::Color32::from_rgb(200, 100, 100))
                                    );
                                    ui.add_space(10.0);
                                }
                                _ => {}
                            }
                        }

                        ui.horizontal(|ui| {
                            match &self.current_update_status {
                                Some(crate::update::UpdateStatus::Success) => {
                                    if ui.button("Close").clicked() {
                                        self.show_update_dialog = false;
                                    }
                                }
                                Some(crate::update::UpdateStatus::Error(_)) => {
                                    if ui.button("Close").clicked() {
                                        self.show_update_dialog = false;
                                        self.current_update_status = None;
                                    }
                                }
                                Some(crate::update::UpdateStatus::Downloading(_)) |
                                Some(crate::update::UpdateStatus::Installing) => {
                                }
                                _ => {
                                    ui.add_enabled_ui(!self.update_in_progress, |ui| {
                                        if ui.button("Update Now").clicked() {
                                            self.update_checker.perform_update();
                                        }
                                    });
                                    if ui.button("Later").clicked() {
                                        self.show_update_dialog = false;
                                    }
                                }
                            }
                        });
                    }
                });
        }

        self.process_loaded_song();

        if self.song_loading {
            let screen_rect = ctx.screen_rect();
            egui::Area::new(egui::Id::new("loading_overlay"))
                .fixed_pos(screen_rect.min)
                .show(ctx, |ui| {
                    ui.allocate_ui(screen_rect.size(), |ui| {
                        ui.painter().rect_filled(
                            screen_rect,
                            0.0,
                            egui::Color32::from_black_alpha(180)
                        );

                        ui.with_layout(
                            egui::Layout::centered_and_justified(egui::Direction::TopDown),
                            |ui| {
                                ui.vertical_centered(|ui| {
                                    let center_y = screen_rect.height() * 0.45;
                                    ui.add_space(center_y);

                                    crate::ui::spinner::square_spinner(ui, 50.0, self.primary_color());

                                    ui.add_space(15.0);

                                    ui.label(
                                        egui::RichText::new("Loading...")
                                            .size(16.0)
                                            .color(egui::Color32::from_gray(200))
                                    );
                                });
                            }
                        );
                    });
                });

            ctx.request_repaint();
        }
    }
}
