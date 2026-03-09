use crate::app::WavesApp;
use crate::types::{SidebarView, GroupedView};

impl WavesApp {
    pub(super) fn handle_browsing_mode_toggle(&mut self) {
        crate::sound::play_cursor_sound(
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

    pub(super) fn handle_escape(&mut self) {
        crate::sound::play_cursor_sound(
            self.config.ui_sounds_enabled,
            self.config.ui_sounds_volume
        );
        self.clipboard = None;
    }
}
