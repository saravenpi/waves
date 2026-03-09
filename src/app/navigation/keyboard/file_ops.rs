use crate::app::WavesApp;
use crate::types::SidebarView;

impl WavesApp {
    pub(super) fn handle_new_folder(&mut self) {
        if matches!(self.sidebar_view, SidebarView::FileBrowser) {
            crate::sound::play_cursor_sound(
                self.config.ui_sounds_enabled,
                self.config.ui_sounds_volume
            );
            self.new_folder_prompt = Some(String::new());
        }
    }

    pub(super) fn handle_rename(&mut self) {
        if matches!(self.sidebar_view, SidebarView::FileBrowser) {
            if let Some(entry) = self.columns[0].entries.get(self.columns[0].selected).cloned() {
                crate::sound::play_cursor_sound(
                    self.config.ui_sounds_enabled,
                    self.config.ui_sounds_volume
                );
                self.rename_prompt = Some((entry.path.clone(), entry.name.clone()));
            }
        }
    }

    pub(super) fn handle_yank(&mut self) {
        if matches!(self.sidebar_view, SidebarView::FileBrowser) {
            if let Some(entry) = self.columns[0].entries.get(self.columns[0].selected).cloned() {
                crate::sound::play_cursor_sound(
                    self.config.ui_sounds_enabled,
                    self.config.ui_sounds_volume
                );
                if let Some((ref path, crate::types::ClipboardOperation::Copy)) = self.clipboard {
                    if path == &entry.path {
                        self.clipboard = None;
                    } else {
                        self.clipboard = Some((entry.path.clone(), crate::types::ClipboardOperation::Copy));
                    }
                } else {
                    self.clipboard = Some((entry.path.clone(), crate::types::ClipboardOperation::Copy));
                }
            }
        }
    }

    pub(super) fn handle_cut(&mut self) {
        if matches!(self.sidebar_view, SidebarView::FileBrowser) {
            if let Some(entry) = self.columns[0].entries.get(self.columns[0].selected).cloned() {
                crate::sound::play_cursor_sound(
                    self.config.ui_sounds_enabled,
                    self.config.ui_sounds_volume
                );
                if let Some((ref path, crate::types::ClipboardOperation::Cut)) = self.clipboard {
                    if path == &entry.path {
                        self.clipboard = None;
                    } else {
                        self.clipboard = Some((entry.path.clone(), crate::types::ClipboardOperation::Cut));
                    }
                } else {
                    self.clipboard = Some((entry.path.clone(), crate::types::ClipboardOperation::Cut));
                }
            }
        }
    }

    pub(super) fn handle_paste(&mut self) {
        if matches!(self.sidebar_view, SidebarView::FileBrowser) {
            crate::sound::play_cursor_sound(
                self.config.ui_sounds_enabled,
                self.config.ui_sounds_volume
            );
            self.paste_clipboard();
        }
    }

    pub(super) fn handle_delete(&mut self) {
        if matches!(self.sidebar_view, SidebarView::FileBrowser) {
            if let Some(entry) = self.columns[0].entries.get(self.columns[0].selected).cloned() {
                crate::sound::play_cursor_sound(
                    self.config.ui_sounds_enabled,
                    self.config.ui_sounds_volume
                );
                self.delete_confirm_prompt = Some(entry.path.clone());
            }
        }
    }
}
