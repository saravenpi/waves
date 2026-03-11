use crate::audio::{PlayerState, SpectrumCapture, create_placeholder_waveform};
use crate::album_cover::extract_album_cover;
use crate::metadata::extract_metadata;
use crate::app::state::SongLoadData;
use crate::WavesApp;
use eframe::egui;
use rodio::{Decoder, Sink, Source};
use std::collections::VecDeque;
use std::io::Cursor;
use std::path::Path;
use std::sync::{Arc, Mutex};
use std::time::{Duration, Instant};

impl WavesApp {
    pub fn play_file(&mut self, path: &Path, _ctx: &egui::Context) {
        let ext = path.extension().and_then(|s| s.to_str()).unwrap_or("");
        if !matches!(ext, "mp3" | "wav" | "flac" | "ogg" | "m4a") {
            return;
        }

        if self.song_loading {
            return;
        }

        while self.song_data_receiver.try_recv().is_ok() {}

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

        let stream = match &self.audio_stream {
            Some(s) => s,
            None => {
                eprintln!("No audio stream available");
                self.song_loading = false;
                self.song_loading_started = None;
                return;
            }
        };

        let sink = Sink::connect_new(stream.mixer());
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

            self.dots_initialized = false;

            if !self.waveform_cache.contains(&data.path) {
                let path_clone = data.path.clone();
                let sender = self.waveform_sender.clone();
                std::thread::spawn(move || {
                    let result = std::panic::catch_unwind(|| {
                        crate::audio::extract_waveform(&path_clone)
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
}
