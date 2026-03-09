use eframe::egui;
use rodio::{Decoder, Sink, Source};
use std::collections::VecDeque;
use std::fs::File;
use std::io::BufReader;
use std::path::PathBuf;
use std::sync::{Arc, Mutex};
use std::time::{Duration, Instant};

use super::spectrum::SpectrumCapture;

pub struct PlayerState {
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
    #[allow(dead_code)]
    pub fn is_paused(&self) -> bool {
        self.sink.is_paused()
    }

    #[allow(dead_code)]
    pub fn pause(&mut self) {
        self.sink.pause();
        let elapsed = self.start_time.elapsed();
        self.pause_offset += elapsed;
    }

    #[allow(dead_code)]
    pub fn play(&mut self) {
        self.sink.play();
        self.start_time = Instant::now();
    }

    #[allow(dead_code)]
    pub fn get_current_position(&self) -> Duration {
        if self.sink.is_paused() {
            self.pause_offset
        } else {
            let elapsed = self.start_time.elapsed();
            self.pause_offset + elapsed
        }
    }

    #[allow(dead_code)]
    pub fn set_volume(&mut self, volume: f32) {
        self.sink.set_volume(volume);
    }

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

}
