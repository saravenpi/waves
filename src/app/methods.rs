use crate::app::{WavesApp, CacheResult};
use crate::types::{Column, FileEntry, BrowsingMode, GroupedView};
use crate::file_operations::browser::{read_directory, collect_all_audio_files, group_by_artist, group_by_album};
use std::sync::mpsc::channel;

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
            self.cache_loading = false;
            self.cache_receiver = None;
        }

        let current_entries = match self.browsing_mode {
            BrowsingMode::FileStructure => {
                read_directory(&self.current_dir)
            }
            BrowsingMode::ByArtist => {
                if self.audio_files_cache.is_none() && !self.cache_loading {
                    self.cache_loading = true;
                    let root_dir = self.root_dir.clone();
                    let (sender, receiver) = channel();
                    self.cache_receiver = Some(receiver);

                    std::thread::spawn(move || {
                        let audio_files = collect_all_audio_files(&root_dir);
                        let _ = sender.send(CacheResult::AudioFiles(audio_files.clone()));
                        let artists = group_by_artist(&audio_files);
                        let _ = sender.send(CacheResult::ArtistGroups(artists));
                        let albums = group_by_album(&audio_files);
                        let _ = sender.send(CacheResult::AlbumGroups(albums));
                    });
                }

                match &self.grouped_view {
                    GroupedView::GroupList => {
                        if let Some(ref artists) = self.artist_groups_cache {
                            artists.iter().map(|(artist, _)| {
                                FileEntry {
                                    name: format!("🎤 {}", artist),
                                    path: self.root_dir.join(artist),
                                    is_dir: true,
                                }
                            }).collect()
                        } else {
                            vec![FileEntry {
                                name: "Loading artists...".to_string(),
                                path: self.root_dir.clone(),
                                is_dir: true,
                            }]
                        }
                    }
                    GroupedView::TrackList(_) => {
                        self.current_group_tracks.iter().enumerate().map(|(idx, path)| {
                            let name = path.file_name()
                                .unwrap_or_default()
                                .to_string_lossy()
                                .to_string();
                            FileEntry {
                                name: format!("{}. {}", idx + 1, name),
                                path: path.clone(),
                                is_dir: false,
                            }
                        }).collect()
                    }
                }
            }
            BrowsingMode::ByAlbum => {
                if self.audio_files_cache.is_none() && !self.cache_loading {
                    self.cache_loading = true;
                    let root_dir = self.root_dir.clone();
                    let (sender, receiver) = channel();
                    self.cache_receiver = Some(receiver);

                    std::thread::spawn(move || {
                        let audio_files = collect_all_audio_files(&root_dir);
                        let _ = sender.send(CacheResult::AudioFiles(audio_files.clone()));
                        let artists = group_by_artist(&audio_files);
                        let _ = sender.send(CacheResult::ArtistGroups(artists));
                        let albums = group_by_album(&audio_files);
                        let _ = sender.send(CacheResult::AlbumGroups(albums));
                    });
                }

                match &self.grouped_view {
                    GroupedView::GroupList => {
                        if let Some(ref albums) = self.album_groups_cache {
                            albums.iter().map(|(album, _)| {
                                FileEntry {
                                    name: format!("💿 {}", album),
                                    path: self.root_dir.join(album),
                                    is_dir: true,
                                }
                            }).collect()
                        } else {
                            vec![FileEntry {
                                name: "Loading albums...".to_string(),
                                path: self.root_dir.clone(),
                                is_dir: true,
                            }]
                        }
                    }
                    GroupedView::TrackList(_) => {
                        self.current_group_tracks.iter().enumerate().map(|(idx, path)| {
                            let name = path.file_name()
                                .unwrap_or_default()
                                .to_string_lossy()
                                .to_string();
                            FileEntry {
                                name: format!("{}. {}", idx + 1, name),
                                path: path.clone(),
                                is_dir: false,
                            }
                        }).collect()
                    }
                }
            }
            BrowsingMode::AllSongs => {
                if self.audio_files_cache.is_none() && !self.cache_loading {
                    self.cache_loading = true;
                    let root_dir = self.root_dir.clone();
                    let (sender, receiver) = channel();
                    self.cache_receiver = Some(receiver);

                    std::thread::spawn(move || {
                        let audio_files = collect_all_audio_files(&root_dir);
                        let _ = sender.send(CacheResult::AudioFiles(audio_files.clone()));
                        let artists = group_by_artist(&audio_files);
                        let _ = sender.send(CacheResult::ArtistGroups(artists));
                        let albums = group_by_album(&audio_files);
                        let _ = sender.send(CacheResult::AlbumGroups(albums));
                    });
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
                    vec![FileEntry {
                        name: "Loading songs...".to_string(),
                        path: self.root_dir.clone(),
                        is_dir: true,
                    }]
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
