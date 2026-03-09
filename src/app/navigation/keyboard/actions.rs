use crate::app::WavesApp;
use crate::types::{SidebarView, BrowsingMode, GroupedView};
use eframe::egui;

impl WavesApp {
    pub(super) fn handle_enter_key(&mut self, ctx: &egui::Context) {
        match self.sidebar_view {
            SidebarView::FileBrowser => self.handle_file_browser_enter(ctx),
            SidebarView::Liked => self.handle_liked_enter(ctx),
            SidebarView::Settings => self.handle_settings_enter(ctx),
        }
    }

    pub(super) fn handle_file_browser_enter(&mut self, ctx: &egui::Context) {
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

    pub(super) fn handle_liked_enter(&mut self, ctx: &egui::Context) {
        if let Some(fav) = self.liked.get(self.liked_selected).cloned() {
            if fav.is_dir {
                self.current_dir = fav.path.clone();
                self.update_columns_with_selection(Some(0));
                self.sidebar_view = SidebarView::FileBrowser;
            } else {
                self.play_file(&fav.path, ctx);
            }
        }
    }

    pub(super) fn handle_settings_enter(&mut self, ctx: &egui::Context) {
        let mut changed = false;
        match self.settings_focused_item {
            0 => changed = self.cycle_primary_color_forward(),
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
                    changed = self.cycle_animation_type_forward();
                }
            }
            5 => {
                changed = self.toggle_sidebar_position();
            }
            6 => {
                self.config.ui_sounds_enabled = !self.config.ui_sounds_enabled;
                let _ = self.config.save();
                changed = true;
            }
            7 => {
                if self.config.ui_sounds_enabled {
                    changed = self.increase_ui_volume();
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
            crate::sound::play_cursor_sound(
                self.config.ui_sounds_enabled,
                self.config.ui_sounds_volume
            );
        }
    }
}
