use crate::app::WavesApp;
use crate::types::SidebarView;
use eframe::egui;

impl WavesApp {
    pub(super) fn handle_arrow_left(&mut self, ctx: &egui::Context) {
        match self.sidebar_view {
            SidebarView::Settings => self.handle_settings_arrow_left(),
            SidebarView::Liked => self.handle_liked_arrow_left(ctx),
            SidebarView::FileBrowser => {
                match self.playback_context {
                    SidebarView::Liked => self.play_previous_liked(ctx),
                    _ => self.play_previous_song(ctx),
                }
            }
        }
    }

    pub(super) fn handle_settings_arrow_left(&mut self) {
        let mut changed = false;
        match self.settings_focused_item {
            0 => changed = self.cycle_primary_color_backward(),
            7 => {
                if self.config.ui_sounds_enabled {
                    changed = self.decrease_ui_volume();
                }
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

    pub(super) fn handle_liked_arrow_left(&mut self, ctx: &egui::Context) {
        if self.liked_selected > 0 {
            self.liked_selected -= 1;
            if let Some(fav) = self.liked.get(self.liked_selected).cloned() {
                if !fav.is_dir {
                    self.play_file(&fav.path, ctx);
                }
            }
        }
    }

    pub(super) fn handle_arrow_right(&mut self, ctx: &egui::Context) {
        match self.sidebar_view {
            SidebarView::Settings => self.handle_settings_arrow_right(),
            SidebarView::Liked => self.handle_liked_arrow_right(ctx),
            SidebarView::FileBrowser => {
                match self.playback_context {
                    SidebarView::Liked => self.play_next_liked(ctx),
                    _ => self.play_next_song(ctx),
                }
            }
        }
    }

    pub(super) fn handle_settings_arrow_right(&mut self) {
        let mut changed = false;
        match self.settings_focused_item {
            0 => changed = self.cycle_primary_color_forward(),
            7 => {
                if self.config.ui_sounds_enabled {
                    changed = self.increase_ui_volume();
                }
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

    pub(super) fn handle_liked_arrow_right(&mut self, ctx: &egui::Context) {
        if self.liked_selected < self.liked.len().saturating_sub(1) {
            self.liked_selected += 1;
            if let Some(fav) = self.liked.get(self.liked_selected).cloned() {
                if !fav.is_dir {
                    self.play_file(&fav.path, ctx);
                }
            }
        }
    }
}
