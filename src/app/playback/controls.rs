use crate::WavesApp;
use eframe::egui;
use std::path::PathBuf;
use std::time::{Duration, Instant};

impl WavesApp {
    pub fn toggle_pause(&mut self) {
        if let Ok(mut player) = self.player.lock() {
            if let Some(state) = player.as_mut() {
                if state.sink.is_paused() {
                    state.sink.play();
                    state.start_time = Instant::now();
                } else {
                    state.sink.pause();
                    let elapsed = state.start_time.elapsed();
                    state.pause_offset += elapsed;
                }
            }
        }
    }

    pub fn get_current_position(&self) -> Option<Duration> {
        if let Ok(player) = self.player.lock() {
            if let Some(state) = player.as_ref() {
                let current_pos = if state.sink.is_paused() {
                    state.pause_offset
                } else {
                    let elapsed = state.start_time.elapsed();
                    state.pause_offset + elapsed
                };

                return Some(current_pos.min(state.duration));
            }
        }
        None
    }

    pub fn play_next_song(&mut self, ctx: &egui::Context) {
        use crate::types::{BrowsingMode, GroupedView};

        if self.columns.is_empty() || self.columns[0].entries.is_empty() {
            return;
        }

        let current_file = self.player.lock().unwrap()
            .as_ref()
            .map(|state| state.current_file.clone());

        if let Some(current) = current_file {
            match self.browsing_mode {
                BrowsingMode::ByArtist | BrowsingMode::ByAlbum => {
                    if matches!(self.grouped_view, GroupedView::TrackList(_)) && !self.current_group_tracks.is_empty() {
                        if let Some(pos) = self.current_group_tracks.iter().position(|p| p == &current) {
                            let next_pos = (pos + 1) % self.current_group_tracks.len();
                            self.columns[0].selected = next_pos;
                            let next_track = self.current_group_tracks[next_pos].clone();
                            self.play_file(&next_track, ctx);
                            return;
                        }
                    }
                }
                _ => {}
            }

            let audio_files: Vec<(usize, PathBuf)> = self.columns[0].entries.iter().enumerate()
                .filter_map(|(idx, entry)| {
                    if entry.is_dir {
                        return None;
                    }
                    let ext = entry.path.extension().and_then(|s| s.to_str()).unwrap_or("");
                    if matches!(ext, "mp3" | "wav" | "flac" | "ogg" | "m4a") {
                        Some((idx, entry.path.clone()))
                    } else {
                        None
                    }
                })
                .collect();

            if audio_files.is_empty() {
                return;
            }

            let current_pos = audio_files.iter().position(|(_, path)| path == &current);

            if let Some(pos) = current_pos {
                let next_pos = (pos + 1) % audio_files.len();
                let (idx, path) = &audio_files[next_pos];
                self.columns[0].selected = *idx;
                self.play_file(path, ctx);
            } else {
                let (idx, path) = &audio_files[0];
                self.columns[0].selected = *idx;
                self.play_file(path, ctx);
            }
        }
    }

    pub fn play_previous_song(&mut self, ctx: &egui::Context) {
        use crate::types::{BrowsingMode, GroupedView};

        if self.columns.is_empty() || self.columns[0].entries.is_empty() {
            return;
        }

        let current_file = self.player.lock().unwrap()
            .as_ref()
            .map(|state| state.current_file.clone());

        if let Some(current) = current_file {
            match self.browsing_mode {
                BrowsingMode::ByArtist | BrowsingMode::ByAlbum => {
                    if matches!(self.grouped_view, GroupedView::TrackList(_)) && !self.current_group_tracks.is_empty() {
                        if let Some(pos) = self.current_group_tracks.iter().position(|p| p == &current) {
                            let prev_pos = if pos == 0 {
                                self.current_group_tracks.len() - 1
                            } else {
                                pos - 1
                            };
                            self.columns[0].selected = prev_pos;
                            let prev_track = self.current_group_tracks[prev_pos].clone();
                            self.play_file(&prev_track, ctx);
                            return;
                        }
                    }
                }
                _ => {}
            }

            let audio_files: Vec<(usize, PathBuf)> = self.columns[0].entries.iter().enumerate()
                .filter_map(|(idx, entry)| {
                    if entry.is_dir {
                        return None;
                    }
                    let ext = entry.path.extension().and_then(|s| s.to_str()).unwrap_or("");
                    if matches!(ext, "mp3" | "wav" | "flac" | "ogg" | "m4a") {
                        Some((idx, entry.path.clone()))
                    } else {
                        None
                    }
                })
                .collect();

            if audio_files.is_empty() {
                return;
            }

            let current_pos = audio_files.iter().position(|(_, path)| path == &current);

            if let Some(pos) = current_pos {
                let prev_pos = if pos == 0 {
                    audio_files.len() - 1
                } else {
                    pos - 1
                };
                let (idx, path) = &audio_files[prev_pos];
                self.columns[0].selected = *idx;
                self.play_file(path, ctx);
            } else {
                let (idx, path) = &audio_files[audio_files.len() - 1];
                self.columns[0].selected = *idx;
                self.play_file(path, ctx);
            }
        }
    }

    pub fn play_next_liked(&mut self, ctx: &egui::Context) {
        if self.liked.is_empty() {
            return;
        }

        let current_file = self.player.lock().unwrap()
            .as_ref()
            .map(|state| state.current_file.clone());

        let audio_liked: Vec<(usize, PathBuf)> = self.liked.iter().enumerate()
            .filter_map(|(idx, fav)| {
                if fav.is_dir {
                    return None;
                }
                let ext = fav.path.extension().and_then(|s| s.to_str()).unwrap_or("");
                if matches!(ext, "mp3" | "wav" | "flac" | "ogg" | "m4a") {
                    Some((idx, fav.path.clone()))
                } else {
                    None
                }
            })
            .collect();

        if audio_liked.is_empty() {
            return;
        }

        if let Some(current) = current_file {
            let current_pos = audio_liked.iter().position(|(_, path)| path == &current);

            if let Some(pos) = current_pos {
                let next_pos = (pos + 1) % audio_liked.len();
                let (idx, path) = &audio_liked[next_pos];
                self.liked_selected = *idx;
                self.play_file(path, ctx);
            } else {
                let (idx, path) = &audio_liked[0];
                self.liked_selected = *idx;
                self.play_file(path, ctx);
            }
        } else {
            let (idx, path) = &audio_liked[0];
            self.liked_selected = *idx;
            self.play_file(path, ctx);
        }
    }

    pub fn play_previous_liked(&mut self, ctx: &egui::Context) {
        if self.liked.is_empty() {
            return;
        }

        let current_file = self.player.lock().unwrap()
            .as_ref()
            .map(|state| state.current_file.clone());

        let audio_liked: Vec<(usize, PathBuf)> = self.liked.iter().enumerate()
            .filter_map(|(idx, fav)| {
                if fav.is_dir {
                    return None;
                }
                let ext = fav.path.extension().and_then(|s| s.to_str()).unwrap_or("");
                if matches!(ext, "mp3" | "wav" | "flac" | "ogg" | "m4a") {
                    Some((idx, fav.path.clone()))
                } else {
                    None
                }
            })
            .collect();

        if audio_liked.is_empty() {
            return;
        }

        if let Some(current) = current_file {
            let current_pos = audio_liked.iter().position(|(_, path)| path == &current);

            if let Some(pos) = current_pos {
                let prev_pos = if pos == 0 {
                    audio_liked.len() - 1
                } else {
                    pos - 1
                };
                let (idx, path) = &audio_liked[prev_pos];
                self.liked_selected = *idx;
                self.play_file(path, ctx);
            } else {
                let (idx, path) = &audio_liked[audio_liked.len() - 1];
                self.liked_selected = *idx;
                self.play_file(path, ctx);
            }
        } else {
            let (idx, path) = &audio_liked[audio_liked.len() - 1];
            self.liked_selected = *idx;
            self.play_file(path, ctx);
        }
    }
}
