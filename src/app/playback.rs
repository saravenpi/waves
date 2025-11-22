use crate::audio::{PlayerState, SpectrumCapture, create_placeholder_waveform, extract_waveform};
use crate::album_cover::extract_album_cover;
use crate::metadata::extract_metadata;
use crate::app::state::SongLoadData;
use crate::WavesApp;
use eframe::egui;
use rodio::{Decoder, OutputStreamBuilder, Sink, Source};
use rustfft::num_complex::Complex;
use std::collections::VecDeque;
use std::fs::File;
use std::io::{BufReader, Cursor};
use std::path::{Path, PathBuf};
use std::sync::{Arc, Mutex};
use std::time::{Duration, Instant};

impl WavesApp {
    /// Queues an audio file to be played in the background.
    ///
    /// Spawns a background thread to load file data and metadata while the UI shows a spinner.
    /// # Arguments
    /// * `path` - Path to the audio file to play
    /// * `_ctx` - egui context (currently unused but kept for future use)
    pub fn play_file(&mut self, path: &Path, _ctx: &egui::Context) {
        let ext = path.extension().and_then(|s| s.to_str()).unwrap_or("");
        if !matches!(ext, "mp3" | "wav" | "flac" | "ogg" | "m4a") {
            return;
        }

        self.song_loading = true;
        self.song_loading_started = Some(std::time::Instant::now());

        let path_clone = path.to_path_buf();
        let sender = self.song_data_sender.clone();
        let cached_waveform = self.waveform_cache.get(path).cloned();

        std::thread::spawn(move || {
            let result = std::panic::catch_unwind(std::panic::AssertUnwindSafe(|| {
                let file_bytes = match std::fs::read(&path_clone) {
                    Ok(bytes) => bytes,
                    Err(e) => {
                        eprintln!("Failed to read audio file {:?}: {}", path_clone, e);
                        return None;
                    }
                };

                let (title, artist, _album, _date, _track, duration) = extract_metadata(&path_clone);
                let waveform = cached_waveform.unwrap_or_else(create_placeholder_waveform);

                Some(SongLoadData {
                    path: path_clone,
                    file_bytes,
                    title,
                    artist,
                    duration,
                    waveform,
                })
            }));

            if let Ok(Some(data)) = result {
                let _ = sender.send(data);
            }
        });
    }

    /// Processes loaded song data and starts playback.
    ///
    /// Called from the render loop to check for loaded song data.
    pub fn process_loaded_song(&mut self) {
        if let Some(started) = self.song_loading_started {
            if started.elapsed() > Duration::from_secs(30) {
                eprintln!("Song loading timeout - resetting loading state");
                self.song_loading = false;
                self.song_loading_started = None;
                return;
            }
        }

        let data = match self.song_data_receiver.try_recv() {
            Ok(d) => d,
            Err(_) => return,
        };

        match OutputStreamBuilder::open_default_stream() {
            Ok(_stream) => {
                let sink = Sink::connect_new(_stream.mixer());
                {
                        let cursor = Cursor::new(data.file_bytes);
                        let ext = data.path.extension()
                            .and_then(|s| s.to_str())
                            .unwrap_or("")
                            .to_lowercase();
                        let decoder_result = std::panic::catch_unwind(
                            std::panic::AssertUnwindSafe(|| {
                                match ext.as_str() {
                                    "m4a" | "mp4" | "aac" => Decoder::new_mp4(cursor),
                                    _ => Decoder::new(cursor),
                                }
                            })
                        );
                        let source = match decoder_result {
                            Ok(Ok(s)) => s,
                            Ok(Err(e)) => {
                                eprintln!("Failed to decode audio file {:?}: {}", data.path, e);
                                self.song_loading = false;
                                self.song_loading_started = None;
                                return;
                            }
                            Err(_) => {
                                eprintln!("Panic while decoding audio file {:?}", data.path);
                                self.song_loading = false;
                                self.song_loading_started = None;
                                return;
                            }
                        };

                        let sample_rate = source.sample_rate();
                        let channels = source.channels();
                        let audio_buffer = Arc::new(Mutex::new(VecDeque::new()));

                        let captured_source = SpectrumCapture::new(
                            source,
                            audio_buffer.clone()
                        );

                        sink.append(captured_source);
                        sink.set_volume(self.volume);

                        let mut player = self.player.lock().unwrap();
                        *player = Some(PlayerState {
                            _stream,
                            sink,
                            current_file: data.path.clone(),
                            waveform: data.waveform,
                            audio_buffer,
                            sample_rate,
                            channels,
                            duration: data.duration,
                            start_time: Instant::now(),
                            pause_offset: Duration::from_secs(0),
                            title: data.title,
                            artist: data.artist,
                            album_cover: None,
                        });

                        self.song_loading = false;
                        self.song_loading_started = None;

                        if !self.waveform_cache.contains_key(&data.path) {
                            let path_clone = data.path.clone();
                            let sender = self.waveform_sender.clone();
                            std::thread::spawn(move || {
                                let result = std::panic::catch_unwind(|| {
                                    extract_waveform(&path_clone)
                                });
                                if let Ok(waveform) = result {
                                    let _ = sender.send((path_clone, waveform));
                                } else {
                                    eprintln!("Panic while extracting waveform for {:?}", path_clone);
                                }
                            });
                        }

                        let path_clone = data.path.clone();
                        let sender = self.album_cover_sender.clone();
                        std::thread::spawn(move || {
                            let result = std::panic::catch_unwind(|| {
                                extract_album_cover(&path_clone)
                            });

                            if let Ok(Some(cover_data)) = result {
                                if let Ok(img) = image::load_from_memory(&cover_data) {
                                    let size = [img.width() as usize, img.height() as usize];
                                    let rgba = img.to_rgba8();
                                    let pixels = rgba.as_flat_samples();
                                    if let Ok(color_image) = std::panic::catch_unwind(|| {
                                        egui::ColorImage::from_rgba_unmultiplied(
                                            size,
                                            pixels.as_slice()
                                        )
                                    }) {
                                        let _ = sender.send((path_clone, color_image));
                                    }
                                }
                            }
                        });
                }
            }
            Err(_) => {
                self.song_loading = false;
                self.song_loading_started = None;
            },
        }
    }

    /// Toggles pause/play state of the currently playing track.
    ///
    /// Updates playback timing state to maintain accurate position tracking.
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

    /// Returns the current playback position accounting for pause state.
    ///
    /// # Returns
    /// Current position as Duration, or None if no track is playing
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

    /// Plays the next audio file in the current directory.
    ///
    /// Wraps around to the first track when reaching the end of the playlist.
    /// # Arguments
    /// * `ctx` - egui context for UI updates
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

    /// Plays the previous audio file in the current directory.
    ///
    /// Wraps around to the last track when at the beginning of the playlist.
    /// # Arguments
    /// * `ctx` - egui context for UI updates
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

    /// Updates the spectrum analyzer bars using FFT on the audio buffer.
    ///
    /// Performs 4096-sample FFT with Hann window and logarithmic frequency bands.
    /// Applies smoothing and gravity effects for natural bar movement.
    /// # Arguments
    /// * `audio_buffer` - Circular buffer containing recent audio samples
    /// * `sample_rate` - Audio sample rate in Hz
    /// * `channels` - Number of audio channels
    pub fn update_spectrum(&mut self, audio_buffer: &Arc<Mutex<VecDeque<f32>>>, sample_rate: u32, channels: u16) {
        const NUM_BARS: usize = 64;
        const FFT_SIZE: usize = 4096;

        let samples: Vec<f32> = {
            let buffer = audio_buffer.lock().unwrap();
            if buffer.len() < FFT_SIZE {
                return;
            }

            let mono_samples: Vec<f32> = if channels == 2 {
                buffer.iter()
                    .copied()
                    .collect::<Vec<f32>>()
                    .chunks(2)
                    .map(|chunk| (chunk[0] + chunk.get(1).unwrap_or(&0.0)) / 2.0)
                    .collect()
            } else {
                buffer.iter().copied().collect()
            };

            if mono_samples.len() < FFT_SIZE {
                return;
            }

            mono_samples[mono_samples.len() - FFT_SIZE..].to_vec()
        };

        let mut buffer: Vec<Complex<f32>> = samples
            .iter()
            .enumerate()
            .map(|(i, &sample)| {
                let window = 0.5 - 0.5 * ((2.0 * std::f32::consts::PI * i as f32) / FFT_SIZE as f32).cos();
                Complex::new(sample * window, 0.0)
            })
            .collect();

        let fft = self.fft_planner.plan_fft_forward(FFT_SIZE);
        fft.process(&mut buffer);

        let nyquist = sample_rate as f32 / 2.0;
        let freq_per_bin = nyquist / (FFT_SIZE as f32 / 2.0);

        let freq_bands: Vec<(f32, f32)> = (0..NUM_BARS)
            .map(|i| {
                let freq_min = 20.0 * (20000.0_f32 / 20.0).powf(i as f32 / NUM_BARS as f32);
                let freq_max = 20.0 * (20000.0_f32 / 20.0).powf((i + 1) as f32 / NUM_BARS as f32);
                (freq_min, freq_max)
            })
            .collect();

        let mut last_bin_end = 0;

        for (i, &(freq_min, freq_max)) in freq_bands.iter().enumerate() {
            let mut bin_start = (freq_min / freq_per_bin) as usize;
            let mut bin_end = ((freq_max / freq_per_bin) as usize).min(FFT_SIZE / 2);

            if bin_start < last_bin_end {
                bin_start = last_bin_end;
            }

            if bin_end <= bin_start {
                bin_end = bin_start + 1;
            }

            last_bin_end = bin_end;

            let normalized = if bin_start >= FFT_SIZE / 2 || bin_end > FFT_SIZE / 2 {
                0.0
            } else {
                let bin_count = bin_end - bin_start;
                let mut magnitude_sum = 0.0_f32;
                for j in bin_start..bin_end {
                    if j < buffer.len() {
                        let magnitude = (buffer[j].re * buffer[j].re + buffer[j].im * buffer[j].im).sqrt();
                        magnitude_sum += magnitude;
                    }
                }

                if magnitude_sum.is_finite() && magnitude_sum > 0.0 {
                    let avg_magnitude = magnitude_sum / bin_count as f32;
                    let db = 20.0 * (avg_magnitude + 1e-10).log10();
                    ((db + 110.0) / 160.0).clamp(0.0, 1.0)
                } else {
                    0.0
                }
            };

            let smoothing_up = 0.6;
            let smoothing_down = 0.85;

            let new_value = if normalized > self.spectrum_bars[i] {
                self.spectrum_bars[i] * (1.0 - smoothing_up) + normalized * smoothing_up
            } else {
                self.spectrum_bars[i] * (1.0 - smoothing_down) + normalized * smoothing_down
            };

            let gravity = if new_value < 0.05 {
                0.01
            } else {
                0.005
            };

            self.spectrum_bars[i] = (new_value - gravity).max(0.0);

            if self.spectrum_bars[i] < 0.001 {
                self.spectrum_bars[i] = 0.0;
            }
        }
    }

    /// Plays the next audio file in the favorites list.
    ///
    /// Wraps around to the first favorite when reaching the end.
    /// # Arguments
    /// * `ctx` - egui context for UI updates
    pub fn play_next_favorite(&mut self, ctx: &egui::Context) {
        if self.favorites.is_empty() {
            return;
        }

        let current_file = self.player.lock().unwrap()
            .as_ref()
            .map(|state| state.current_file.clone());

        let audio_favorites: Vec<(usize, PathBuf)> = self.favorites.iter().enumerate()
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

        if audio_favorites.is_empty() {
            return;
        }

        if let Some(current) = current_file {
            let current_pos = audio_favorites.iter().position(|(_, path)| path == &current);

            if let Some(pos) = current_pos {
                let next_pos = (pos + 1) % audio_favorites.len();
                let (idx, path) = &audio_favorites[next_pos];
                self.favorites_selected = *idx;
                self.play_file(path, ctx);
            } else {
                let (idx, path) = &audio_favorites[0];
                self.favorites_selected = *idx;
                self.play_file(path, ctx);
            }
        } else {
            let (idx, path) = &audio_favorites[0];
            self.favorites_selected = *idx;
            self.play_file(path, ctx);
        }
    }

    /// Plays the previous audio file in the favorites list.
    ///
    /// Wraps around to the last favorite when at the beginning.
    /// # Arguments
    /// * `ctx` - egui context for UI updates
    pub fn play_previous_favorite(&mut self, ctx: &egui::Context) {
        if self.favorites.is_empty() {
            return;
        }

        let current_file = self.player.lock().unwrap()
            .as_ref()
            .map(|state| state.current_file.clone());

        let audio_favorites: Vec<(usize, PathBuf)> = self.favorites.iter().enumerate()
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

        if audio_favorites.is_empty() {
            return;
        }

        if let Some(current) = current_file {
            let current_pos = audio_favorites.iter().position(|(_, path)| path == &current);

            if let Some(pos) = current_pos {
                let prev_pos = if pos == 0 {
                    audio_favorites.len() - 1
                } else {
                    pos - 1
                };
                let (idx, path) = &audio_favorites[prev_pos];
                self.favorites_selected = *idx;
                self.play_file(path, ctx);
            } else {
                let (idx, path) = &audio_favorites[audio_favorites.len() - 1];
                self.favorites_selected = *idx;
                self.play_file(path, ctx);
            }
        } else {
            let (idx, path) = &audio_favorites[audio_favorites.len() - 1];
            self.favorites_selected = *idx;
            self.play_file(path, ctx);
        }
    }

    /// Seeks to a specific position in the currently playing track.
    ///
    /// Attempts fast seeking first, falls back to reloading and skipping if unsupported.
    /// # Arguments
    /// * `progress` - Target position as normalized value between 0.0 and 1.0
    pub fn seek_to_position(&mut self, progress: f32) {
        let target_duration = {
            let player = self.player.lock().unwrap();
            if let Some(state) = player.as_ref() {
                Duration::from_secs_f32(state.duration.as_secs_f32() * progress)
            } else {
                return;
            }
        };

        if let Ok(mut player) = self.player.lock() {
            if let Some(state) = player.as_mut() {
                let was_paused = state.sink.is_paused();

                if state.sink.try_seek(target_duration).is_ok() {
                    state.start_time = Instant::now();
                    state.pause_offset = target_duration;

                    if was_paused {
                        state.sink.pause();
                    }
                } else {
                    let current_path = state.current_file.clone();
                    let audio_buffer = state.audio_buffer.clone();
                    drop(player);

                    match File::open(&current_path) {
                        Ok(file) => {
                            if let Ok(mut player) = self.player.lock() {
                                if let Some(state) = player.as_mut() {
                                    state.sink.stop();
                                    audio_buffer.lock().unwrap().clear();

                                    let buf_reader = BufReader::new(file);
                                    let ext = current_path.extension()
                                        .and_then(|s| s.to_str())
                                        .unwrap_or("")
                                        .to_lowercase();
                                    let decoder_result = match ext.as_str() {
                                        "m4a" | "mp4" | "aac" => Decoder::new_mp4(buf_reader),
                                        _ => Decoder::new(buf_reader),
                                    };
                                    if let Ok(source) = decoder_result {
                                        let source_with_skip = source.skip_duration(target_duration);
                                        let captured_source = SpectrumCapture::new(
                                            source_with_skip,
                                            audio_buffer.clone()
                                        );

                                        state.sink.append(captured_source);
                                        state.start_time = Instant::now();
                                        state.pause_offset = target_duration;

                                        if was_paused {
                                            state.sink.pause();
                                        }
                                    }
                                }
                            }
                        }
                        Err(_) => {},
                    }
                }
            }
        }
    }
}
