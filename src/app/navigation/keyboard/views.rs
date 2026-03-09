use crate::app::WavesApp;
use crate::types::SidebarView;
use eframe::egui;

impl WavesApp {
    pub(super) fn handle_tab(&mut self, ctx: &egui::Context) {
        crate::sound::play_cursor_sound(
            self.config.ui_sounds_enabled,
            self.config.ui_sounds_volume
        );
        if ctx.input(|i| i.modifiers.shift) {
            self.cycle_view_backward();
        } else {
            self.cycle_view_forward();
        }
    }

    pub(super) fn cycle_view_forward(&mut self) {
        self.sidebar_view = match self.sidebar_view {
            SidebarView::FileBrowser => {
                if self.liked_selected >= self.liked.len() && !self.liked.is_empty() {
                    self.liked_selected = self.liked.len() - 1;
                } else if self.liked.is_empty() {
                    self.liked_selected = 0;
                }
                SidebarView::Liked
            },
            SidebarView::Liked => SidebarView::Settings,
            SidebarView::Settings => SidebarView::FileBrowser,
        };
    }

    pub(super) fn cycle_view_backward(&mut self) {
        self.sidebar_view = match self.sidebar_view {
            SidebarView::FileBrowser => SidebarView::Settings,
            SidebarView::Liked => SidebarView::FileBrowser,
            SidebarView::Settings => {
                if self.liked_selected >= self.liked.len() && !self.liked.is_empty() {
                    self.liked_selected = self.liked.len() - 1;
                } else if self.liked.is_empty() {
                    self.liked_selected = 0;
                }
                SidebarView::Liked
            },
        };
    }
}
