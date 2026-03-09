use crate::app::WavesApp;
use crate::types::SidebarView;
use crate::metadata::extract_metadata;
use crate::ui::input::MetadataEditor;

impl WavesApp {
    pub(super) fn handle_favorite(&mut self) {
        crate::sound::play_cursor_sound(
            self.config.ui_sounds_enabled,
            self.config.ui_sounds_volume
        );
        self.toggle_favorite();
    }

    pub(super) fn handle_metadata_editor(&mut self) {
        crate::sound::play_cursor_sound(
            self.config.ui_sounds_enabled,
            self.config.ui_sounds_volume
        );
        match self.sidebar_view {
            SidebarView::FileBrowser => self.open_metadata_editor_for_browser(),
            SidebarView::Liked => self.open_metadata_editor_for_liked(),
            SidebarView::Settings => {}
        }
    }

    pub(super) fn open_metadata_editor_for_browser(&mut self) {
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

    pub(super) fn open_metadata_editor_for_liked(&mut self) {
        if let Some(fav) = self.liked.get(self.liked_selected).cloned() {
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
}
