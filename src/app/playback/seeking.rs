use crate::audio::SpectrumCapture;
use crate::WavesApp;
use rodio::{Decoder, Sink, Source};
use std::fs::File;
use std::io::BufReader;
use std::time::{Duration, Instant};

impl WavesApp {
    pub fn seek_to_position(&mut self, progress: f32) {
        let target_duration = {
            let player = self.player.lock().unwrap();
            if let Some(state) = player.as_ref() {
                Duration::from_secs_f32(state.duration.as_secs_f32() * progress)
            } else {
                return;
            }
        };

        if progress < 0.01 {
            self.dots_initialized = false;
        }

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
                    let volume = state.sink.volume();
                    drop(player);

                    let stream = match &self.audio_stream {
                        Some(s) => s,
                        None => {
                            eprintln!("No audio stream available for seeking");
                            return;
                        }
                    };

                    match File::open(&current_path) {
                        Ok(file) => {
                            if let Ok(mut player) = self.player.lock() {
                                if let Some(state) = player.as_mut() {
                                    audio_buffer.lock().unwrap().clear();

                                    let new_sink = Sink::connect_new(stream.mixer());
                                    state.sink.stop();
                                    state.sink = new_sink;
                                    state.sink.set_volume(volume);

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
