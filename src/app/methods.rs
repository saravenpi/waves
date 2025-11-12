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

        if self.cache_root_dir.as_ref() != Some(&self.root_dir) {
            self.audio_files_cache = None;
            self.artist_groups_cache = None;
            self.album_groups_cache = None;
            self.cache_root_dir = Some(self.root_dir.clone());
        }

        let current_entries = match self.browsing_mode {
            BrowsingMode::FileStructure => {
                read_directory(&self.current_dir)
            }
            BrowsingMode::ByArtist => {
                if self.audio_files_cache.is_none() {
                    self.audio_files_cache = Some(collect_all_audio_files(&self.root_dir));
                }

                if self.artist_groups_cache.is_none() {
                    if let Some(ref audio_files) = self.audio_files_cache {
                        self.artist_groups_cache = Some(group_by_artist(audio_files));
                    }
                }

                if let Some(ref artists) = self.artist_groups_cache {
                    artists.iter().map(|(artist, _)| {
                        FileEntry {
                            name: format!("🎤 {}", artist),
                            path: self.root_dir.join(artist),
                            is_dir: true,
                        }
                    }).collect()
                } else {
                    Vec::new()
                }
            }
            BrowsingMode::ByAlbum => {
                if self.audio_files_cache.is_none() {
                    self.audio_files_cache = Some(collect_all_audio_files(&self.root_dir));
                }

                if self.album_groups_cache.is_none() {
                    if let Some(ref audio_files) = self.audio_files_cache {
                        self.album_groups_cache = Some(group_by_album(audio_files));
                    }
                }

                if let Some(ref albums) = self.album_groups_cache {
                    albums.iter().map(|(album, _)| {
                        FileEntry {
                            name: format!("💿 {}", album),
                            path: self.root_dir.join(album),
                            is_dir: true,
                        }
                    }).collect()
                } else {
                    Vec::new()
                }
            }
            BrowsingMode::AllSongs => {
                if self.audio_files_cache.is_none() {
                    self.audio_files_cache = Some(collect_all_audio_files(&self.root_dir));
                }

                if let Some(ref audio_files) = self.audio_files_cache {
                    audio_files.iter().map(|path| {
                        let name = path.file_name()
                            .unwrap_or_default()
                            .to_string_lossy()
                            .to_string();
                        FileEntry {
                            name: format!("🎵 {}", name),
                            path: path.clone(),
                            is_dir: false,
                        }
                    }).collect()
                } else {
                    Vec::new()
                }
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
        match self.browsing_mode {
            BrowsingMode::ByArtist => {
                if let Some(ref artists) = self.artist_groups_cache {
                    let clean_name = group_name.trim_start_matches("🎤 ");
                    for (artist, files) in artists {
                        if artist == clean_name {
                            return files.clone();
                        }
                    }
                }
                Vec::new()
            }
            BrowsingMode::ByAlbum => {
                if let Some(ref albums) = self.album_groups_cache {
                    let clean_name = group_name.trim_start_matches("💿 ");
                    for (album, files) in albums {
                        if album == clean_name {
                            return files.clone();
                        }
                    }
                }
                Vec::new()
            }
            _ => Vec::new()
        }
    }
}
