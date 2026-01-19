use crate::app::WavesApp;
use crate::types::{FileEntry, ClipboardOperation, Favorite, SidebarView, BrowsingMode, GroupedView};
use crate::metadata::extract_metadata;
use crate::ui::input::MetadataEditor;
use eframe::egui;

impl WavesApp {
    /// Handles keyboard navigation for file browsing and playback control.
    ///
    /// Processes vim-style keybindings (h/j/k/l) for navigation and media controls.
    /// # Arguments
    /// * `key` - The keyboard key that was pressed
    /// * `ctx` - egui context for UI operations
    pub fn handle_navigation(&mut self, key: egui::Key, ctx: &egui::Context) {
        if self.columns.is_empty() {
            return;
        }

        if self.new_folder_prompt.is_some() || self.rename_prompt.is_some() || self.delete_confirm_prompt.is_some() || self.search_open {
            return;
        }

        match key {
            // vim-style: 'g' followed by 'g' within 500ms to go to top
            egui::Key::G => {
                // Check if this is 'gg' (two g presses within 500ms)
                let now = std::time::Instant::now();
                if let Some(last_press) = self.last_g_press {
                    if now.duration_since(last_press).as_millis() < 500 {
                        // This is 'gg' - go to top
                        let mut moved = false;
                        match self.sidebar_view {
                            SidebarView::FileBrowser => {
                                if !self.columns[0].entries.is_empty() && self.columns[0].selected != 0 {
                                    self.columns[0].selected = 0;
                                    moved = true;
                                }
                            }
                            SidebarView::Favorites => {
                                if !self.favorites.is_empty() && self.favorites_selected != 0 {
                                    self.favorites_selected = 0;
                                    moved = true;
                                }
                            }
                            SidebarView::Settings => {
                                if self.settings_focused_item != 0 {
                                    self.settings_focused_item = 0;
                                    moved = true;
                                }
                            }
                        }
                        if moved {
                            self.scroll_to_selection = true;
                            crate::cursor_sound::play_cursor_sound(
                                self.config.ui_sounds_enabled,
                                self.config.ui_sounds_volume
                            );
                        }
                        self.last_g_press = None;
                        return;
                    }
                }
                // First 'g' press or Shift+G (Go to bottom)
                // Check if Shift is held for 'G' (go to bottom)
                if ctx.input(|i| i.modifiers.shift) {
                    // Shift+G - go to bottom
                    let mut moved = false;
                    match self.sidebar_view {
                        SidebarView::FileBrowser => {
                            if !self.columns[0].entries.is_empty() {
                                let last = self.columns[0].entries.len().saturating_sub(1);
                                if self.columns[0].selected != last {
                                    self.columns[0].selected = last;
                                    moved = true;
                                }
                            }
                        }
                        SidebarView::Favorites => {
                            if !self.favorites.is_empty() {
                                let last = self.favorites.len().saturating_sub(1);
                                if self.favorites_selected != last {
                                    self.favorites_selected = last;
                                    moved = true;
                                }
                            }
                        }
                        SidebarView::Settings => {
                            let max_items = 11;
                            let last = max_items - 1;
                            if self.settings_focused_item != last {
                                self.settings_focused_item = last;
                                moved = true;
                            }
                        }
                    }
                    if moved {
                        self.scroll_to_selection = true;
                        crate::cursor_sound::play_cursor_sound(
                            self.config.ui_sounds_enabled,
                            self.config.ui_sounds_volume
                        );
                    }
                    self.last_g_press = None;
                } else {
                    // Just 'g' - remember this press for potential 'gg'
                    self.last_g_press = Some(now);
                }
            }
            egui::Key::J => {
                let mut moved = false;
                match self.sidebar_view {
                    SidebarView::FileBrowser => {
                        if !self.columns[0].entries.is_empty() {
                            if self.columns[0].selected < self.columns[0].entries.len().saturating_sub(1) {
                                self.columns[0].selected += 1;
                            } else {
                                // Wrap around to top
                                self.columns[0].selected = 0;
                            }
                            moved = true;
                        }
                    }
                    SidebarView::Favorites => {
                        if !self.favorites.is_empty() {
                            if self.favorites_selected < self.favorites.len().saturating_sub(1) {
                                self.favorites_selected += 1;
                            } else {
                                // Wrap around to top
                                self.favorites_selected = 0;
                            }
                            moved = true;
                        }
                    }
                    SidebarView::Settings => {
                        let max_items = 11;
                        if self.settings_focused_item < max_items - 1 {
                            self.settings_focused_item += 1;
                        } else {
                            // Wrap around to top
                            self.settings_focused_item = 0;
                        }
                        moved = true;
                    }
                }
                if moved {
                    self.scroll_to_selection = true;
                    crate::cursor_sound::play_cursor_sound(
                        self.config.ui_sounds_enabled,
                        self.config.ui_sounds_volume
                    );
                }
            }
            egui::Key::K => {
                let mut moved = false;
                match self.sidebar_view {
                    SidebarView::FileBrowser => {
                        if !self.columns[0].entries.is_empty() {
                            if self.columns[0].selected > 0 {
                                self.columns[0].selected -= 1;
                            } else {
                                // Wrap around to bottom
                                self.columns[0].selected = self.columns[0].entries.len().saturating_sub(1);
                            }
                            moved = true;
                        }
                    }
                    SidebarView::Favorites => {
                        if !self.favorites.is_empty() {
                            if self.favorites_selected > 0 {
                                self.favorites_selected -= 1;
                            } else {
                                // Wrap around to bottom
                                self.favorites_selected = self.favorites.len().saturating_sub(1);
                            }
                            moved = true;
                        }
                    }
                    SidebarView::Settings => {
                        let max_items = 12;
                        if self.settings_focused_item > 0 {
                            self.settings_focused_item -= 1;
                        } else {
                            // Wrap around to bottom
                            self.settings_focused_item = max_items - 1;
                        }
                        moved = true;
                    }
                }
                if moved {
                    self.scroll_to_selection = true;
                    crate::cursor_sound::play_cursor_sound(
                        self.config.ui_sounds_enabled,
                        self.config.ui_sounds_volume
                    );
                }
            }
            egui::Key::L | egui::Key::Enter => {
                match self.sidebar_view {
                    SidebarView::FileBrowser => {
                        if let Some(entry) = self.columns[0].entries.get(self.columns[0].selected).cloned() {
                            if entry.is_dir {
                                match self.browsing_mode {
                                    BrowsingMode::FileStructure => {
                                        self.current_dir = entry.path.clone();
                                        self.update_columns_with_selection(Some(0));
                                    }
                                    BrowsingMode::ByArtist | BrowsingMode::ByAlbum => {
                                        if matches!(self.grouped_view, GroupedView::GroupList) {
                                            let files = self.get_files_for_group(&entry.name);
                                            if !files.is_empty() {
                                                self.current_group_tracks = files.clone();
                                                let group_name = entry.name.clone();
                                                self.grouped_view = GroupedView::TrackList(group_name);
                                                self.update_columns_with_selection(Some(0));
                                                let first_track = self.current_group_tracks[0].clone();
                                                self.play_file(&first_track, ctx);
                                            }
                                        }
                                    }
                                    BrowsingMode::AllSongs => {}
                                }
                            } else {
                                self.play_file(&entry.path, ctx);
                            }
                        }
                    }
                    SidebarView::Favorites => {
                        if let Some(fav) = self.favorites.get(self.favorites_selected).cloned() {
                            if fav.is_dir {
                                self.current_dir = fav.path.clone();
                                self.update_columns_with_selection(Some(0));
                                self.sidebar_view = SidebarView::FileBrowser;
                            } else {
                                self.play_file(&fav.path, ctx);
                            }
                        }
                    }
                    SidebarView::Settings => {
                        let mut changed = false;
                        match self.settings_focused_item {
                            0 => {
                                let preset_colors = vec![
                                    "#9664FF", "#4A90E2", "#50E3C2",
                                    "#FF6B9D", "#FF8A00", "#FF4444"
                                ];
                                if let Some(current_idx) = preset_colors.iter().position(|c| c.to_lowercase() == self.config.primary_color.to_lowercase()) {
                                    let next_idx = (current_idx + 1) % preset_colors.len();
                                    self.config.primary_color = preset_colors[next_idx].to_string();
                                    let _ = self.config.save();
                                    changed = true;
                                }
                            }
                            1 => {
                                self.config.show_status_bar = !self.config.show_status_bar;
                                let _ = self.config.save();
                                changed = true;
                            }
                            2 => {
                                self.config.decorations = !self.config.decorations;
                                let _ = self.config.save();
                                ctx.send_viewport_cmd(egui::ViewportCommand::Decorations(self.config.decorations));
                                changed = true;
                            }
                            3 => {
                                self.config.animation = !self.config.animation;
                                let _ = self.config.save();
                                changed = true;
                            }
                            4 => {
                                if self.config.animation {
                                    use crate::config::AnimationType;
                                    self.config.animation_type = match self.config.animation_type {
                                        AnimationType::Spectrum => AnimationType::WaveformPulse,
                                        AnimationType::WaveformPulse => AnimationType::CircleSpectrum,
                                        AnimationType::CircleSpectrum => AnimationType::Agbe,
                                        AnimationType::Agbe => AnimationType::Dots,
                                        AnimationType::Dots => AnimationType::Spectrum,
                                    };
                                    let _ = self.config.save();
                                    changed = true;
                                }
                            }
                            5 => {
                                use crate::config::SidebarPosition;
                                self.config.sidebar_position = match self.config.sidebar_position {
                                    SidebarPosition::Left => SidebarPosition::Right,
                                    SidebarPosition::Right => SidebarPosition::Left,
                                };
                                let _ = self.config.save();
                                changed = true;
                            }
                            6 => {
                                self.config.ui_sounds_enabled = !self.config.ui_sounds_enabled;
                                let _ = self.config.save();
                                changed = true;
                            }
                            7 => {
                                if self.config.ui_sounds_enabled {
                                    let old_volume = self.config.ui_sounds_volume;
                                    self.config.ui_sounds_volume = (self.config.ui_sounds_volume + 0.05).min(1.0);
                                    if (self.config.ui_sounds_volume - old_volume).abs() > 0.001 {
                                        let _ = self.config.save();
                                        changed = true;
                                    }
                                }
                            }
                            8 => {
                                self.config.startup_sound_enabled = !self.config.startup_sound_enabled;
                                let _ = self.config.save();
                                changed = true;
                            }
                            _ => {}
                        }
                        if changed {
                            crate::cursor_sound::play_cursor_sound(
                                self.config.ui_sounds_enabled,
                                self.config.ui_sounds_volume
                            );
                        }
                    }
                }
            }
            egui::Key::H => {
                let mut changed = false;
                match self.sidebar_view {
                    SidebarView::FileBrowser => {
                        match self.browsing_mode {
                            BrowsingMode::FileStructure => {
                                if let Some(parent) = self.current_dir.parent() {
                                    if parent >= self.root_dir.as_path() {
                                        self.current_dir = parent.to_path_buf();
                                        self.update_columns();
                                        changed = true;
                                    }
                                }
                            }
                            BrowsingMode::ByArtist | BrowsingMode::ByAlbum => {
                                if matches!(self.grouped_view, GroupedView::TrackList(_)) {
                                    self.grouped_view = GroupedView::GroupList;
                                    self.current_group_tracks.clear();
                                    self.update_columns_with_selection(Some(0));
                                    changed = true;
                                }
                            }
                            BrowsingMode::AllSongs => {}
                        }
                    }
                    SidebarView::Settings => {
                        match self.settings_focused_item {
                            0 => {
                                let preset_colors = vec![
                                    "#9664FF", "#4A90E2", "#50E3C2",
                                    "#FF6B9D", "#FF8A00", "#FF4444"
                                ];
                                if let Some(current_idx) = preset_colors.iter().position(|c| c.to_lowercase() == self.config.primary_color.to_lowercase()) {
                                    let prev_idx = if current_idx == 0 {
                                        preset_colors.len() - 1
                                    } else {
                                        current_idx - 1
                                    };
                                    self.config.primary_color = preset_colors[prev_idx].to_string();
                                    let _ = self.config.save();
                                    changed = true;
                                }
                            }
                            1 => {
                                if self.config.show_status_bar {
                                    self.config.show_status_bar = false;
                                    let _ = self.config.save();
                                    changed = true;
                                }
                            }
                            2 => {
                                if self.config.decorations {
                                    self.config.decorations = false;
                                    let _ = self.config.save();
                                    ctx.send_viewport_cmd(egui::ViewportCommand::Decorations(self.config.decorations));
                                    changed = true;
                                }
                            }
                            3 => {
                                if self.config.animation {
                                    self.config.animation = false;
                                    let _ = self.config.save();
                                    changed = true;
                                }
                            }
                            4 => {
                                if self.config.animation {
                                    use crate::config::AnimationType;
                                    self.config.animation_type = match self.config.animation_type {
                                        AnimationType::Spectrum => AnimationType::Dots,
                                        AnimationType::Dots => AnimationType::Agbe,
                                        AnimationType::Agbe => AnimationType::CircleSpectrum,
                                        AnimationType::CircleSpectrum => AnimationType::WaveformPulse,
                                        AnimationType::WaveformPulse => AnimationType::Spectrum,
                                    };
                                    let _ = self.config.save();
                                    changed = true;
                                }
                            }
                            5 => {
                                use crate::config::SidebarPosition;
                                self.config.sidebar_position = match self.config.sidebar_position {
                                    SidebarPosition::Left => SidebarPosition::Right,
                                    SidebarPosition::Right => SidebarPosition::Left,
                                };
                                let _ = self.config.save();
                                changed = true;
                            }
                            6 => {
                                if self.config.ui_sounds_enabled {
                                    self.config.ui_sounds_enabled = false;
                                    let _ = self.config.save();
                                    changed = true;
                                }
                            }
                            7 => {
                                if self.config.ui_sounds_enabled {
                                    let old_volume = self.config.ui_sounds_volume;
                                    self.config.ui_sounds_volume = (self.config.ui_sounds_volume - 0.05).max(0.0);
                                    if (old_volume - self.config.ui_sounds_volume).abs() > 0.001 {
                                        let _ = self.config.save();
                                        changed = true;
                                    }
                                }
                            }
                            8 => {
                                if self.config.startup_sound_enabled {
                                    self.config.startup_sound_enabled = false;
                                    let _ = self.config.save();
                                    changed = true;
                                }
                            }
                            _ => {}
                        }
                    }
                    _ => {}
                }
                if changed {
                    crate::cursor_sound::play_cursor_sound(
                        self.config.ui_sounds_enabled,
                        self.config.ui_sounds_volume
                    );
                }
            }
            egui::Key::ArrowLeft => {
                match self.sidebar_view {
                    SidebarView::Settings => {
                        let mut changed = false;
                        match self.settings_focused_item {
                            0 => {
                                let preset_colors = vec![
                                    "#9664FF", "#4A90E2", "#50E3C2",
                                    "#FF6B9D", "#FF8A00", "#FF4444"
                                ];
                                if let Some(current_idx) = preset_colors.iter().position(|c| c.to_lowercase() == self.config.primary_color.to_lowercase()) {
                                    let prev_idx = if current_idx == 0 {
                                        preset_colors.len() - 1
                                    } else {
                                        current_idx - 1
                                    };
                                    self.config.primary_color = preset_colors[prev_idx].to_string();
                                    let _ = self.config.save();
                                    changed = true;
                                }
                            }
                            7 => {
                                if self.config.ui_sounds_enabled {
                                    let old_volume = self.config.ui_sounds_volume;
                                    self.config.ui_sounds_volume = (self.config.ui_sounds_volume - 0.05).max(0.0);
                                    if (old_volume - self.config.ui_sounds_volume).abs() > 0.001 {
                                        let _ = self.config.save();
                                        changed = true;
                                    }
                                }
                            }
                            _ => {}
                        }
                        if changed {
                            crate::cursor_sound::play_cursor_sound(
                                self.config.ui_sounds_enabled,
                                self.config.ui_sounds_volume
                            );
                        }
                    }
                    SidebarView::Favorites => {
                        if self.favorites_selected > 0 {
                            self.favorites_selected -= 1;
                            if let Some(fav) = self.favorites.get(self.favorites_selected).cloned() {
                                if !fav.is_dir {
                                    self.play_file(&fav.path, ctx);
                                }
                            }
                        }
                    }
                    SidebarView::FileBrowser => {
                        match self.playback_context {
                            SidebarView::Favorites => self.play_previous_favorite(ctx),
                            _ => self.play_previous_song(ctx),
                        }
                    }
                }
            }
            egui::Key::ArrowRight => {
                match self.sidebar_view {
                    SidebarView::Settings => {
                        let mut changed = false;
                        match self.settings_focused_item {
                            0 => {
                                let preset_colors = vec![
                                    "#9664FF", "#4A90E2", "#50E3C2",
                                    "#FF6B9D", "#FF8A00", "#FF4444"
                                ];
                                if let Some(current_idx) = preset_colors.iter().position(|c| c.to_lowercase() == self.config.primary_color.to_lowercase()) {
                                    let next_idx = (current_idx + 1) % preset_colors.len();
                                    self.config.primary_color = preset_colors[next_idx].to_string();
                                    let _ = self.config.save();
                                    changed = true;
                                }
                            }
                            7 => {
                                if self.config.ui_sounds_enabled {
                                    let old_volume = self.config.ui_sounds_volume;
                                    self.config.ui_sounds_volume = (self.config.ui_sounds_volume + 0.05).min(1.0);
                                    if (self.config.ui_sounds_volume - old_volume).abs() > 0.001 {
                                        let _ = self.config.save();
                                        changed = true;
                                    }
                                }
                            }
                            _ => {}
                        }
                        if changed {
                            crate::cursor_sound::play_cursor_sound(
                                self.config.ui_sounds_enabled,
                                self.config.ui_sounds_volume
                            );
                        }
                    }
                    SidebarView::Favorites => {
                        if self.favorites_selected < self.favorites.len().saturating_sub(1) {
                            self.favorites_selected += 1;
                            if let Some(fav) = self.favorites.get(self.favorites_selected).cloned() {
                                if !fav.is_dir {
                                    self.play_file(&fav.path, ctx);
                                }
                            }
                        }
                    }
                    SidebarView::FileBrowser => {
                        match self.playback_context {
                            SidebarView::Favorites => self.play_next_favorite(ctx),
                            _ => self.play_next_song(ctx),
                        }
                    }
                }
            }
            egui::Key::Space => {
                self.toggle_pause();
            }
            egui::Key::ArrowUp => {
                crate::cursor_sound::play_cursor_sound(
                    self.config.ui_sounds_enabled,
                    self.config.ui_sounds_volume
                );
                self.volume = (self.volume + 0.05).min(1.0);
                if let Ok(player) = self.player.lock() {
                    if let Some(state) = player.as_ref() {
                        state.sink.set_volume(self.volume);
                    }
                }
            }
            egui::Key::ArrowDown => {
                crate::cursor_sound::play_cursor_sound(
                    self.config.ui_sounds_enabled,
                    self.config.ui_sounds_volume
                );
                self.volume = (self.volume - 0.05).max(0.0);
                if let Ok(player) = self.player.lock() {
                    if let Some(state) = player.as_ref() {
                        state.sink.set_volume(self.volume);
                    }
                }
            }
            egui::Key::N => {
                if matches!(self.sidebar_view, SidebarView::FileBrowser) {
                    crate::cursor_sound::play_cursor_sound(
                        self.config.ui_sounds_enabled,
                        self.config.ui_sounds_volume
                    );
                    self.new_folder_prompt = Some(String::new());
                }
            }
            egui::Key::R => {
                if matches!(self.sidebar_view, SidebarView::FileBrowser) {
                    if let Some(entry) = self.columns[0].entries.get(self.columns[0].selected).cloned() {
                        crate::cursor_sound::play_cursor_sound(
                            self.config.ui_sounds_enabled,
                            self.config.ui_sounds_volume
                        );
                        self.rename_prompt = Some((entry.path.clone(), entry.name.clone()));
                    }
                }
            }
            egui::Key::Y => {
                if matches!(self.sidebar_view, SidebarView::FileBrowser) {
                    if let Some(entry) = self.columns[0].entries.get(self.columns[0].selected).cloned() {
                        crate::cursor_sound::play_cursor_sound(
                            self.config.ui_sounds_enabled,
                            self.config.ui_sounds_volume
                        );
                        if let Some((ref path, ClipboardOperation::Copy)) = self.clipboard {
                            if path == &entry.path {
                                self.clipboard = None;
                            } else {
                                self.clipboard = Some((entry.path.clone(), ClipboardOperation::Copy));
                            }
                        } else {
                            self.clipboard = Some((entry.path.clone(), ClipboardOperation::Copy));
                        }
                    }
                }
            }
            egui::Key::X => {
                if matches!(self.sidebar_view, SidebarView::FileBrowser) {
                    if let Some(entry) = self.columns[0].entries.get(self.columns[0].selected).cloned() {
                        crate::cursor_sound::play_cursor_sound(
                            self.config.ui_sounds_enabled,
                            self.config.ui_sounds_volume
                        );
                        if let Some((ref path, ClipboardOperation::Cut)) = self.clipboard {
                            if path == &entry.path {
                                self.clipboard = None;
                            } else {
                                self.clipboard = Some((entry.path.clone(), ClipboardOperation::Cut));
                            }
                        } else {
                            self.clipboard = Some((entry.path.clone(), ClipboardOperation::Cut));
                        }
                    }
                }
            }
            egui::Key::P => {
                if matches!(self.sidebar_view, SidebarView::FileBrowser) {
                    crate::cursor_sound::play_cursor_sound(
                        self.config.ui_sounds_enabled,
                        self.config.ui_sounds_volume
                    );
                    self.paste_clipboard();
                }
            }
            egui::Key::D => {
                if matches!(self.sidebar_view, SidebarView::FileBrowser) {
                    if let Some(entry) = self.columns[0].entries.get(self.columns[0].selected).cloned() {
                        crate::cursor_sound::play_cursor_sound(
                            self.config.ui_sounds_enabled,
                            self.config.ui_sounds_volume
                        );
                        self.delete_confirm_prompt = Some(entry.path.clone());
                    }
                }
            }
            egui::Key::F => {
                crate::cursor_sound::play_cursor_sound(
                    self.config.ui_sounds_enabled,
                    self.config.ui_sounds_volume
                );
                self.toggle_favorite();
            }
            egui::Key::M => {
                crate::cursor_sound::play_cursor_sound(
                    self.config.ui_sounds_enabled,
                    self.config.ui_sounds_volume
                );
                match self.sidebar_view {
                    SidebarView::FileBrowser => {
                        if let Some(entry) = self.columns[0].entries.get(self.columns[0].selected).cloned() {
                            if !entry.is_dir {
                                let (title, artist, _album, date, _track, _duration) = extract_metadata(&entry.path);
                                let existing_cover_data = crate::album_cover::extract_album_cover(&entry.path);
                                let has_existing_cover = existing_cover_data.is_some();
                                self.metadata_editor = Some(MetadataEditor {
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
                                });
                            }
                        }
                    }
                    SidebarView::Favorites => {
                        if let Some(fav) = self.favorites.get(self.favorites_selected).cloned() {
                            if !fav.is_dir {
                                let (title, artist, _album, date, _track, _duration) = extract_metadata(&fav.path);
                                let existing_cover_data = crate::album_cover::extract_album_cover(&fav.path);
                                let has_existing_cover = existing_cover_data.is_some();
                                self.metadata_editor = Some(MetadataEditor {
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
                                });
                            }
                        }
                    }
                    SidebarView::Settings => {}
                }
            }
            egui::Key::Tab => {
                crate::cursor_sound::play_cursor_sound(
                    self.config.ui_sounds_enabled,
                    self.config.ui_sounds_volume
                );
                if ctx.input(|i| i.modifiers.shift) {
                    self.sidebar_view = match self.sidebar_view {
                        SidebarView::FileBrowser => SidebarView::Settings,
                        SidebarView::Favorites => SidebarView::FileBrowser,
                        SidebarView::Settings => {
                            // Ensure favorites_selected is valid when switching to Favorites
                            if self.favorites_selected >= self.favorites.len() && !self.favorites.is_empty() {
                                self.favorites_selected = self.favorites.len() - 1;
                            } else if self.favorites.is_empty() {
                                self.favorites_selected = 0;
                            }
                            SidebarView::Favorites
                        },
                    };
                } else {
                    self.sidebar_view = match self.sidebar_view {
                        SidebarView::FileBrowser => {
                            // Ensure favorites_selected is valid when switching to Favorites
                            if self.favorites_selected >= self.favorites.len() && !self.favorites.is_empty() {
                                self.favorites_selected = self.favorites.len() - 1;
                            } else if self.favorites.is_empty() {
                                self.favorites_selected = 0;
                            }
                            SidebarView::Favorites
                        },
                        SidebarView::Favorites => SidebarView::Settings,
                        SidebarView::Settings => SidebarView::FileBrowser,
                    };
                }
            }
            egui::Key::B => {
                crate::cursor_sound::play_cursor_sound(
                    self.config.ui_sounds_enabled,
                    self.config.ui_sounds_volume
                );
                match self.sidebar_view {
                    SidebarView::FileBrowser => {
                        self.browsing_mode = self.browsing_mode.next();
                        self.grouped_view = GroupedView::GroupList;
                        self.current_group_tracks.clear();
                        self.update_columns_with_selection(Some(0));
                    }
                    _ => {}
                }
            }
            egui::Key::Escape => {
                crate::cursor_sound::play_cursor_sound(
                    self.config.ui_sounds_enabled,
                    self.config.ui_sounds_volume
                );
                self.clipboard = None;
            }
            _ => {}
        }
    }

    /// Toggles the favorite status of the currently selected file.
    ///
    /// Adds or removes the selected file from the favorites list and persists changes.
    pub fn toggle_favorite(&mut self) {
        let entry = match self.sidebar_view {
            SidebarView::FileBrowser => {
                if self.columns.is_empty() || self.columns[0].entries.is_empty() {
                    return;
                }
                self.columns[0].entries.get(self.columns[0].selected).cloned()
            }
            SidebarView::Favorites => {
                if self.favorites.is_empty() || self.favorites_selected >= self.favorites.len() {
                    return;
                }
                let fav = &self.favorites[self.favorites_selected];
                Some(FileEntry {
                    name: fav.name.clone(),
                    path: fav.path.clone(),
                    is_dir: fav.is_dir,
                })
            }
            SidebarView::Settings => {
                return;
            }
        };

        if let Some(entry) = entry {
            if entry.is_dir {
                return;
            }

            if let Some(pos) = self.favorites.iter().position(|f| f.path == entry.path) {
                self.favorites.remove(pos);
                if self.favorites_selected >= self.favorites.len() && self.favorites_selected > 0 {
                    self.favorites_selected = self.favorites.len() - 1;
                }
            } else {
                self.favorites.insert(0, Favorite {
                    path: entry.path.clone(),
                    name: entry.name.clone(),
                    is_dir: entry.is_dir,
                    timestamp: std::time::SystemTime::now(),
                });
                self.favorites_selected = 0;
            }
            crate::favorites::save(&self.favorites);
        }
    }

    /// Pastes the file or folder from clipboard to the selected destination.
    ///
    /// Performs copy or move operation depending on clipboard operation type.
    /// Handles both files and directories recursively.
    pub fn paste_clipboard(&mut self) {
        if let Some((source_path, operation)) = &self.clipboard {
            let source_name = source_path.file_name()
                .and_then(|n| n.to_str())
                .unwrap_or("unknown");

            let dest_dir = if !self.columns.is_empty() && !self.columns[0].entries.is_empty() {
                if let Some(selected) = self.columns[0].entries.get(self.columns[0].selected) {
                    if selected.is_dir {
                        selected.path.clone()
                    } else {
                        self.current_dir.clone()
                    }
                } else {
                    self.current_dir.clone()
                }
            } else {
                self.current_dir.clone()
            };

            let dest_path = dest_dir.join(source_name);

            if !source_path.exists() {
                eprintln!("Source path no longer exists: {:?}", source_path);
                self.clipboard = None;
                return;
            }

            if dest_path.exists() {
                eprintln!("Destination already exists: {:?}", dest_path);
                return;
            }

            match operation {
                ClipboardOperation::Copy => {
                    let result = if source_path.is_dir() {
                        std::process::Command::new("cp")
                            .arg("-r")
                            .arg(source_path)
                            .arg(&dest_path)
                            .status()
                    } else {
                        std::process::Command::new("cp")
                            .arg(source_path)
                            .arg(&dest_path)
                            .status()
                    };

                    if let Err(e) = result {
                        eprintln!("Failed to copy: {}", e);
                    } else {
                        self.update_columns();
                    }
                }
                ClipboardOperation::Cut => {
                    let result = std::process::Command::new("mv")
                        .arg(source_path)
                        .arg(&dest_path)
                        .status();

                    if let Err(e) = result {
                        eprintln!("Failed to move: {}", e);
                    } else {
                        self.update_columns();
                        self.clipboard = None;
                    }
                }
            }
        }
    }
}
