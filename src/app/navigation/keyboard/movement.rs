use crate::app::WavesApp;
use crate::types::SidebarView;
use eframe::egui;

impl WavesApp {
    pub(super) fn handle_g_key(&mut self, ctx: &egui::Context) {
        let now = std::time::Instant::now();
        if let Some(last_press) = self.last_g_press {
            if now.duration_since(last_press).as_millis() < 500 {
                self.jump_to_first();
                self.last_g_press = None;
                return;
            }
        }

        if ctx.input(|i| i.modifiers.shift) {
            self.jump_to_last();
            self.last_g_press = None;
        } else {
            self.last_g_press = Some(now);
        }
    }

    pub(super) fn jump_to_first(&mut self) {
        let mut moved = false;
        match self.sidebar_view {
            SidebarView::FileBrowser => {
                if !self.columns[0].entries.is_empty() && self.columns[0].selected != 0 {
                    self.columns[0].selected = 0;
                    moved = true;
                }
            }
            SidebarView::Liked => {
                if !self.liked.is_empty() && self.liked_selected != 0 {
                    self.liked_selected = 0;
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
            crate::sound::play_cursor_sound(
                self.config.ui_sounds_enabled,
                self.config.ui_sounds_volume
            );
        }
    }

    pub(super) fn jump_to_last(&mut self) {
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
            SidebarView::Liked => {
                if !self.liked.is_empty() {
                    let last = self.liked.len().saturating_sub(1);
                    if self.liked_selected != last {
                        self.liked_selected = last;
                        moved = true;
                    }
                }
            }
            SidebarView::Settings => {
                let max_items = 13;
                let last = max_items - 1;
                if self.settings_focused_item != last {
                    self.settings_focused_item = last;
                    moved = true;
                }
            }
        }
        if moved {
            self.scroll_to_selection = true;
            crate::sound::play_cursor_sound(
                self.config.ui_sounds_enabled,
                self.config.ui_sounds_volume
            );
        }
    }

    pub(super) fn handle_j_key(&mut self) {
        let mut moved = false;
        match self.sidebar_view {
            SidebarView::FileBrowser => {
                if !self.columns[0].entries.is_empty() {
                    if self.columns[0].selected < self.columns[0].entries.len().saturating_sub(1) {
                        self.columns[0].selected += 1;
                    } else {
                        self.columns[0].selected = 0;
                    }
                    moved = true;
                }
            }
            SidebarView::Liked => {
                if !self.liked.is_empty() {
                    if self.liked_selected < self.liked.len().saturating_sub(1) {
                        self.liked_selected += 1;
                    } else {
                        self.liked_selected = 0;
                    }
                    moved = true;
                }
            }
            SidebarView::Settings => {
                let max_items = 13;
                if self.settings_focused_item < max_items - 1 {
                    self.settings_focused_item += 1;
                } else {
                    self.settings_focused_item = 0;
                }
                moved = true;
            }
        }
        if moved {
            self.scroll_to_selection = true;
            crate::sound::play_cursor_sound(
                self.config.ui_sounds_enabled,
                self.config.ui_sounds_volume
            );
        }
    }

    pub(super) fn handle_k_key(&mut self) {
        let mut moved = false;
        match self.sidebar_view {
            SidebarView::FileBrowser => {
                if !self.columns[0].entries.is_empty() {
                    if self.columns[0].selected > 0 {
                        self.columns[0].selected -= 1;
                    } else {
                        self.columns[0].selected = self.columns[0].entries.len().saturating_sub(1);
                    }
                    moved = true;
                }
            }
            SidebarView::Liked => {
                if !self.liked.is_empty() {
                    if self.liked_selected > 0 {
                        self.liked_selected -= 1;
                    } else {
                        self.liked_selected = self.liked.len().saturating_sub(1);
                    }
                    moved = true;
                }
            }
            SidebarView::Settings => {
                let max_items = 13;
                if self.settings_focused_item > 0 {
                    self.settings_focused_item -= 1;
                } else {
                    self.settings_focused_item = max_items - 1;
                }
                moved = true;
            }
        }
        if moved {
            self.scroll_to_selection = true;
            crate::sound::play_cursor_sound(
                self.config.ui_sounds_enabled,
                self.config.ui_sounds_volume
            );
        }
    }

    pub(super) fn handle_h_key(&mut self, ctx: &egui::Context) {
        let mut changed = false;
        match self.sidebar_view {
            SidebarView::FileBrowser => {
                changed = self.handle_file_browser_back();
            }
            SidebarView::Settings => {
                changed = self.handle_settings_decrease(ctx);
            }
            _ => {}
        }
        if changed {
            crate::sound::play_cursor_sound(
                self.config.ui_sounds_enabled,
                self.config.ui_sounds_volume
            );
        }
    }

    pub(super) fn handle_file_browser_back(&mut self) -> bool {
        use crate::types::{BrowsingMode, GroupedView};
        match self.browsing_mode {
            BrowsingMode::FileStructure => {
                if let Some(parent) = self.current_dir.parent() {
                    if parent >= self.root_dir.as_path() {
                        self.current_dir = parent.to_path_buf();
                        self.update_columns();
                        return true;
                    }
                }
            }
            BrowsingMode::ByArtist | BrowsingMode::ByAlbum => {
                if matches!(self.grouped_view, GroupedView::TrackList(_)) {
                    self.grouped_view = GroupedView::GroupList;
                    self.current_group_tracks.clear();
                    self.update_columns_with_selection(Some(0));
                    return true;
                }
            }
            BrowsingMode::AllSongs => {}
        }
        false
    }
}
