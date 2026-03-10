use eframe::egui;
use crate::app::WavesApp;
use crate::types::SidebarView;

impl eframe::App for WavesApp {
    fn update(&mut self, ctx: &egui::Context, _frame: &mut eframe::Frame) {
        if self.startup_animation {
            let min_time_elapsed = self.startup_time.elapsed().as_secs_f32() >= 1.2;

            if min_time_elapsed {
                self.startup_animation = false;
            } else {
                crate::ui::startup::render_startup_screen(self, ctx);
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

        poll_file_open_receiver(self, ctx);

        #[cfg(target_os = "macos")]
        poll_menu_action_receiver(self, ctx);

        poll_async_receivers(self, ctx);

        track_mouse_movement(self, ctx);

        let (is_playing, is_empty, is_paused) = get_player_state(self);

        if is_playing {
            handle_playback_state(self, ctx, is_empty, is_paused);
        }

        if self.animation_fullscreen {
            crate::ui::fullscreen::render_fullscreen_animation(self, ctx);
            ctx.request_repaint();
            return;
        }

        let mut keys_to_handle = Vec::new();
        let mut dropped_files = Vec::new();

        let search_has_focus = ctx.memory(|m| m.has_focus(egui::Id::new("main_search_bar")));

        collect_input_events(self, ctx, &mut keys_to_handle, &mut dropped_files, search_has_focus);

        ctx.input_mut(|i| {
            i.consume_key(egui::Modifiers::NONE, egui::Key::Tab);
            i.consume_key(egui::Modifiers::SHIFT, egui::Key::Tab);
        });

        for key in keys_to_handle {
            self.handle_navigation(key, ctx);
        }

        handle_dropped_files(self, ctx, dropped_files);

        check_folder_changes(self);

        crate::ui::status_bar::render_status_bar(self, ctx);

        let current_playing_file = if is_playing {
            let player = self.player.lock().unwrap();
            player.as_ref().map(|state| state.current_file.clone())
        } else {
            None
        };

        let sidebar_response = crate::ui::sidebar::render_sidebar(self, ctx, current_playing_file);

        crate::ui::sidebar::update_sidebar_width(self, ctx, &sidebar_response);
        crate::ui::sidebar::handle_sidebar_events(self, sidebar_response.inner, ctx);

        self.scroll_to_selection = false;

        #[cfg(target_os = "macos")]
        let content_top_margin = 40.0;

        #[cfg(not(target_os = "macos"))]
        let content_top_margin = 10.0;

        egui::CentralPanel::default()
            .frame(egui::Frame::default()
                .fill(egui::Color32::from_rgb(16, 16, 16))
                .inner_margin(egui::Margin { left: 30.0, right: 30.0, top: content_top_margin, bottom: 0.0 }))
            .show(ctx, |ui| {
                crate::ui::search_ui::render_search_bar(self, ui, ctx);
                crate::ui::search_ui::render_search_results(self, ui, ctx, search_has_focus);
                crate::ui::playback_ui::render_playback_controls(self, ui, ctx, is_paused);
            });

        crate::ui::dialogs::handle_new_folder_prompt(self, ctx);
        crate::ui::dialogs::handle_rename_prompt(self, ctx);
        crate::ui::dialogs::handle_delete_confirm_prompt(self, ctx);
        crate::ui::dialogs::handle_metadata_editor(self, ctx);
        crate::ui::dialogs::handle_help_modal(self, ctx);
        crate::ui::dialogs::handle_context_menu(self, ctx);

        self.process_loaded_song();

        crate::ui::dialogs::render_loading_overlay(self, ctx);
    }
}

fn poll_file_open_receiver(app: &mut WavesApp, ctx: &egui::Context) {
    while let Ok(file_path) = app.file_open_receiver.try_recv() {
        eprintln!("WAVES: Handling file open request from receiver: {:?}", file_path);
        if file_path.exists() {
            if file_path.is_file() {
                if let Some(parent) = file_path.parent() {
                    app.current_dir = parent.to_path_buf();
                    app.root_dir = parent.to_path_buf();
                    app.update_columns();

                    if !app.columns.is_empty() {
                        if let Some(idx) = app.columns[0].entries.iter().position(|e| e.path == file_path) {
                            app.columns[0].selected = idx;
                        }
                    }
                }
                app.play_file(&file_path, ctx);
            } else if file_path.is_dir() {
                app.current_dir = file_path.clone();
                app.root_dir = file_path;
                app.update_columns();
            }
        }
    }
}

#[cfg(target_os = "macos")]
fn poll_menu_action_receiver(app: &mut WavesApp, ctx: &egui::Context) {
    while let Ok(action) = app.menu_action_receiver.try_recv() {
        use crate::macos::MenuAction;
        match action {
            MenuAction::OpenFile => {
                if let Some(path) = rfd::FileDialog::new()
                    .add_filter("Audio Files", &["mp3", "wav", "flac", "ogg", "m4a"])
                    .pick_file()
                {
                    if let Some(parent) = path.parent() {
                        app.current_dir = parent.to_path_buf();
                        app.root_dir = parent.to_path_buf();
                        app.update_columns();

                        if !app.columns.is_empty() {
                            if let Some(idx) = app.columns[0].entries.iter().position(|e| e.path == path) {
                                app.columns[0].selected = idx;
                            }
                        }
                    }
                    app.play_file(&path, ctx);
                }
            }
            MenuAction::OpenFolder => {
                if let Some(path) = rfd::FileDialog::new().pick_folder() {
                    app.current_dir = path.clone();
                    app.root_dir = path;
                    app.update_columns();
                }
            }
        }
    }
}

fn poll_async_receivers(app: &mut WavesApp, ctx: &egui::Context) {
    const MAX_CACHE_SIZE: usize = 50;

    while let Ok((path, waveform)) = app.waveform_receiver.try_recv() {
        if app.waveform_cache.len() >= MAX_CACHE_SIZE {
            if let Some(oldest_key) = app.waveform_cache.keys().next().cloned() {
                app.waveform_cache.remove(&oldest_key);
            }
        }

        let should_update_player = {
            if let Ok(player) = app.player.lock() {
                player.as_ref().map(|state| state.current_file == path).unwrap_or(false)
            } else {
                false
            }
        };

        app.waveform_cache.insert(path.clone(), waveform);

        if should_update_player {
            if let Ok(mut player) = app.player.lock() {
                if let Some(state) = player.as_mut() {
                    if let Some(cached) = app.waveform_cache.get(&path) {
                        state.waveform = cached.clone();
                        ctx.request_repaint();
                    }
                }
            }
        }
    }

    while let Ok((path, color_image)) = app.album_cover_receiver.try_recv() {
        if app.album_cover_cache.len() >= MAX_CACHE_SIZE {
            if let Some(oldest_key) = app.album_cover_cache.keys().next().cloned() {
                app.album_cover_cache.remove(&oldest_key);
            }
        }

        let texture = ctx.load_texture(
            format!("album_cover_{}", path.display()),
            color_image,
            egui::TextureOptions::LINEAR
        );

        let should_update_player = {
            if let Ok(player) = app.player.lock() {
                player.as_ref().map(|state| state.current_file == path).unwrap_or(false)
            } else {
                false
            }
        };

        app.album_cover_cache.insert(path.clone(), texture);

        if should_update_player {
            if let Ok(mut player) = app.player.lock() {
                if let Some(state) = player.as_mut() {
                    if let Some(cached) = app.album_cover_cache.get(&path) {
                        state.album_cover = Some(cached.clone());
                        ctx.request_repaint();
                    }
                }
            }
        }
    }

    while let Ok((path, duration)) = app.duration_receiver.try_recv() {
        if app.duration_cache.len() >= MAX_CACHE_SIZE {
            if let Some(oldest_key) = app.duration_cache.keys().next().cloned() {
                app.duration_cache.remove(&oldest_key);
            }
        }
        app.duration_cache.insert(path.clone(), duration);
        app.duration_extraction_in_progress.remove(&path);
        ctx.request_repaint();
    }

    let mut cache_updates = Vec::new();
    if let Some(ref receiver) = app.cache_receiver {
        while let Ok(result) = receiver.try_recv() {
            cache_updates.push(result);
        }
    }

    for result in cache_updates {
        match result {
            crate::app::CacheResult::AudioFiles(files) => {
                app.audio_files_cache = Some(files);
            }
            crate::app::CacheResult::ArtistGroups(groups) => {
                app.artist_groups_cache = Some(groups);
            }
            crate::app::CacheResult::AlbumGroups(groups) => {
                app.album_groups_cache = Some(groups);
            }
        }
        app.update_columns();
        ctx.request_repaint();
    }
}

fn track_mouse_movement(app: &mut WavesApp, ctx: &egui::Context) {
    if ctx.input(|i| i.pointer.is_moving()) {
        app.last_mouse_movement = std::time::Instant::now();
    }
}

fn get_player_state(app: &WavesApp) -> (bool, bool, bool) {
    let player = app.player.lock().unwrap();
    if let Some(state) = player.as_ref() {
        (true, state.sink.empty(), state.sink.is_paused())
    } else {
        (false, false, false)
    }
}

fn handle_playback_state(app: &mut WavesApp, ctx: &egui::Context, is_empty: bool, is_paused: bool) {
    if is_empty {
        if app.loop_enabled {
            let current_file = app.player.lock().unwrap()
                .as_ref()
                .map(|state| state.current_file.clone());
            if let Some(file) = current_file {
                app.play_file(&file, ctx);
            }
        } else {
            match app.playback_context {
                SidebarView::Liked => app.play_next_liked(ctx),
                _ => app.play_next_song(ctx),
            }
        }
    } else if !is_paused {
        let (audio_buffer, sample_rate, channels) = {
            let player = app.player.lock().unwrap();
            if let Some(state) = player.as_ref() {
                (state.audio_buffer.clone(), state.sample_rate, state.channels)
            } else {
                if !app.animation_fullscreen {
                    return;
                } else {
                    (std::sync::Arc::new(std::sync::Mutex::new(std::collections::VecDeque::new())), 44100, 2)
                }
            }
        };

        app.update_spectrum(&audio_buffer, sample_rate, channels);
        ctx.request_repaint();
    } else {
        let audio_buffer = {
            let player = app.player.lock().unwrap();
            if let Some(state) = player.as_ref() {
                Some(state.audio_buffer.clone())
            } else {
                None
            }
        };

        if let Some(buffer) = audio_buffer {
            buffer.lock().unwrap().clear();
        }

        for i in 0..app.spectrum_bars.len() {
            app.spectrum_bars[i] = (app.spectrum_bars[i] - 0.02).max(0.0);
        }
        if app.spectrum_bars.iter().any(|&x| x > 0.0) {
            ctx.request_repaint();
        }
    }
}

fn collect_input_events(
    app: &mut WavesApp,
    ctx: &egui::Context,
    keys_to_handle: &mut Vec<egui::Key>,
    dropped_files: &mut Vec<egui::DroppedFile>,
    search_has_focus: bool,
) {
    ctx.input(|i| {
        for event in &i.events {
            if let egui::Event::Key { key, pressed: true, modifiers, .. } = event {
                if app.metadata_editor.is_some() || app.new_folder_prompt.is_some() ||
                   app.rename_prompt.is_some() || search_has_focus {
                    continue;
                }

                if *key == egui::Key::Slash && modifiers.shift && !app.animation_fullscreen {
                    app.help_modal_open = !app.help_modal_open;
                } else if *key == egui::Key::Slash && !modifiers.command && !modifiers.ctrl && !app.animation_fullscreen {
                    app.search_just_opened = true;
                } else {
                    keys_to_handle.push(*key);
                }
            }
        }

        if !i.raw.dropped_files.is_empty() {
            *dropped_files = i.raw.dropped_files.clone();
        }
    });
}

fn handle_dropped_files(app: &mut WavesApp, ctx: &egui::Context, dropped_files: Vec<egui::DroppedFile>) {
    if !dropped_files.is_empty() {
        for file in &dropped_files {
            if let Some(path) = &file.path {
                eprintln!("WAVES: File dropped: {:?}", path);
                if path.exists() {
                    if path.is_file() {
                        if let Some(parent) = path.parent() {
                            app.current_dir = parent.to_path_buf();
                            app.root_dir = parent.to_path_buf();
                            app.update_columns();

                            if !app.columns.is_empty() {
                                if let Some(idx) = app.columns[0].entries.iter().position(|e| &e.path == path) {
                                    app.columns[0].selected = idx;
                                }
                            }
                        }
                        app.play_file(path, ctx);
                    } else if path.is_dir() {
                        app.current_dir = path.clone();
                        app.root_dir = path.clone();
                        app.update_columns();
                    }
                }
            }
        }
    }
}

fn check_folder_changes(app: &mut WavesApp) {
    let now = std::time::Instant::now();
    if now.duration_since(app.last_folder_check).as_secs() >= 2 {
        app.last_folder_check = now;

        if let Ok(entries) = std::fs::read_dir(&app.current_dir) {
            let current_count = entries.count();
            if current_count != app.last_folder_file_count {
                app.last_folder_file_count = current_count;
                app.update_columns();
            }
        }
    }
}
