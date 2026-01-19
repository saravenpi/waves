use eframe::egui;
use rodio::{Decoder, OutputStream, Sink, Source};
use std::collections::VecDeque;
use std::fs::File;
use std::io::BufReader;
use std::path::PathBuf;
use std::sync::{Arc, Mutex};
use std::time::{Duration, Instant};

use super::spectrum::SpectrumCapture;

pub struct PlayerState {
    pub _stream: OutputStream,
    pub sink: Sink,
    pub current_file: PathBuf,
    pub waveform: Vec<f32>,
    pub audio_buffer: Arc<Mutex<VecDeque<f32>>>,
    pub sample_rate: u32,
    pub channels: u16,
    pub duration: Duration,
    pub start_time: Instant,
    pub pause_offset: Duration,
    pub title: String,
    pub artist: Option<String>,
    pub album_cover: Option<egui::TextureHandle>,
}

impl PlayerState {
    /// Checks if playback is currently paused.
    #[allow(dead_code)]
    pub fn is_paused(&self) -> bool {
        self.sink.is_paused()
    }

    /// Pauses playback and updates timing offset.
    #[allow(dead_code)]
    pub fn pause(&mut self) {
        self.sink.pause();
        let elapsed = self.start_time.elapsed();
        self.pause_offset += elapsed;
    }

    /// Resumes playback and resets timing.
    #[allow(dead_code)]
    pub fn play(&mut self) {
        self.sink.play();
        self.start_time = Instant::now();
    }

    /// Returns the current playback position accounting for pause state.
    #[allow(dead_code)]
    pub fn get_current_position(&self) -> Duration {
        if self.sink.is_paused() {
            self.pause_offset
        } else {
            let elapsed = self.start_time.elapsed();
            self.pause_offset + elapsed
        }
    }

    /// Sets the playback volume.
    ///
    /// # Arguments
    /// * `volume` - Volume level from 0.0 to 1.0
    #[allow(dead_code)]
    pub fn set_volume(&mut self, volume: f32) {
        self.sink.set_volume(volume);
    }

    /// Attempts to seek to a specific position using fast seeking.
    ///
    /// # Arguments
    /// * `target_duration` - Target position in the track
    /// # Returns
    /// Ok if seek succeeded, Err if fast seeking is not supported
    #[allow(dead_code)]
    pub fn seek(&mut self, target_duration: Duration) -> Result<(), ()> {
        let was_paused = self.sink.is_paused();

        if self.sink.try_seek(target_duration).is_ok() {
            self.start_time = Instant::now();
            self.pause_offset = target_duration;

            if was_paused {
                self.sink.pause();
            }
            Ok(())
        } else {
            Err(())
        }
    }

    /// Fallback seek method that reloads the file and skips to position.
    ///
    /// Used when fast seeking is not supported by the audio format.
    /// # Arguments
    /// * `target_duration` - Target position in the track
    /// # Returns
    /// Result with error message if operation fails
    #[allow(dead_code)]
    pub fn seek_fallback(&mut self, target_duration: Duration) -> Result<(), String> {
        let current_path = self.current_file.clone();
        let audio_buffer = self.audio_buffer.clone();
        let was_paused = self.sink.is_paused();
        let volume = self.sink.volume();

        audio_buffer.lock().unwrap().clear();

        let new_sink = Sink::connect_new(self._stream.mixer());

        self.sink.stop();
        self.sink = new_sink;
        self.sink.set_volume(volume);

        let file = File::open(&current_path)
            .map_err(|e| format!("Failed to open file: {}", e))?;

        let buf_reader = BufReader::new(file);
        let ext = current_path.extension()
            .and_then(|s| s.to_str())
            .unwrap_or("")
            .to_lowercase();
        let source = match ext.as_str() {
            "m4a" | "mp4" | "aac" => Decoder::new_mp4(buf_reader),
            _ => Decoder::new(buf_reader),
        }.map_err(|e| format!("Failed to decode file: {}", e))?;

        let source_with_skip = source.skip_duration(target_duration);
        let captured_source = SpectrumCapture::new(
            source_with_skip,
            audio_buffer.clone()
        );

        self.sink.append(captured_source);
        self.start_time = Instant::now();
        self.pause_offset = target_duration;

        if was_paused {
            self.sink.pause();
        }

        Ok(())
    }
}
