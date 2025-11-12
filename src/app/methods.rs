use crate::app::WavesApp;
use crate::types::{Column, FileEntry, BrowsingMode};
use crate::file_operations::browser::{read_directory, collect_all_audio_files, group_by_artist, group_by_album};

impl WavesApp {
    /// Updates the file browser columns using the current selection.
    pub fn update_columns(&mut self) {
        self.update_columns_with_selection(None);
    }

    /// Updates the file browser columns with a specified selection index.
    ///
    /// # Arguments
    /// * `selection` - Optional index to select in the updated column
    pub fn update_columns_with_selection(&mut self, selection: Option<usize>) {
        let current_selection = if let Some(sel) = selection {
            sel
        } else if !self.columns.is_empty() {
            self.columns[0].selected
        } else {
            0
        };

        self.columns.clear();

        let current_entries = match self.browsing_mode {
            BrowsingMode::FileStructure => {
                read_directory(&self.current_dir)
            }
            BrowsingMode::ByArtist => {
                let audio_files = collect_all_audio_files(&self.root_dir);
                let artists = group_by_artist(&audio_files);
                artists.into_iter().map(|(artist, _)| {
                    FileEntry {
                        name: format!("🎤 {}", artist),
                        path: self.root_dir.join(&artist),
                        is_dir: true,
                    }
                }).collect()
            }
            BrowsingMode::ByAlbum => {
                let audio_files = collect_all_audio_files(&self.root_dir);
                let albums = group_by_album(&audio_files);
                albums.into_iter().map(|(album, _)| {
                    FileEntry {
                        name: format!("💿 {}", album),
                        path: self.root_dir.join(&album),
                        is_dir: true,
                    }
                }).collect()
            }
            BrowsingMode::AllSongs => {
                let audio_files = collect_all_audio_files(&self.root_dir);
                audio_files.into_iter().map(|path| {
                    let name = path.file_name()
                        .unwrap_or_default()
                        .to_string_lossy()
                        .to_string();
                    FileEntry {
                        name: format!("🎵 {}", name),
                        path,
                        is_dir: false,
                    }
                }).collect()
            }
        };

        let selected = if current_entries.is_empty() {
            0
        } else {
            current_selection.min(current_entries.len().saturating_sub(1))
        };

        let current_column = Column {
            entries: current_entries,
            selected,
        };
        self.columns.push(current_column);
    }

    pub fn get_files_for_group(&self, group_name: &str) -> Vec<std::path::PathBuf> {
        let audio_files = collect_all_audio_files(&self.root_dir);

        match self.browsing_mode {
            BrowsingMode::ByArtist => {
                let artists = group_by_artist(&audio_files);
                let clean_name = group_name.trim_start_matches("🎤 ");
                for (artist, files) in artists {
                    if artist == clean_name {
                        return files;
                    }
                }
                Vec::new()
            }
            BrowsingMode::ByAlbum => {
                let albums = group_by_album(&audio_files);
                let clean_name = group_name.trim_start_matches("💿 ");
                for (album, files) in albums {
                    if album == clean_name {
                        return files;
                    }
                }
                Vec::new()
            }
            _ => Vec::new()
        }
    }
}
